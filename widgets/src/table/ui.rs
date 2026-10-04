//! The table, drawn.
//!
//! Rewritten for the component model, and smaller for it. The table used to
//! own its column widths - a `TableLayout` in a hook slot, each width an
//! `Rc<Cell<u64>>` it wrote to during a drag - which made two tables of the
//! same shape in two windows share a slot or not depending on where the slot
//! landed, and made the widths unreachable to anything that wanted to save
//! them.
//!
//! Now it owns nothing. Widths come in as data, a drag goes out as
//! [`Resized`], and what happens next is the page's business - which, in an
//! Elm backend, means a field on the page and a line in its `update`.

use super::*;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::ops::Range;
use std::rc::Rc;

use guinea_mark::Mark;
use windows_reactor::{
    Border, Callback, Color, Component, ComponentContext, CornerRadius, Grid, GridLength,
    HorizontalAlignment, IntoPayloadCallback, ItemsRepeater, KeyedView, PointerEventInfo,
    Rectangle, ScrollBarVisibility, ScrollViewer, TextBlock, Thickness, VerticalAlignment, View,
    ViewContext, VirtualSource, keyed,
};

use crate::resize::{RESIZE_HANDLE_WIDTH, resize_handle};

const MIN_COLUMN_WIDTH: f64 = 24.0;

/// Horizontal inset shared by header and body cells so column content never
/// sits flush against the resize handle or the row edge. Opt out per column
/// with [`ColumnSpec::flush`].
const CELL_HORIZONTAL_PADDING: f64 = 12.0;
/// Extra vertical room for the header row - it can carry two-line content
/// (label + aggregate value) where body rows stay a single fixed height.
const HEADER_VERTICAL_PADDING: f64 = 8.0;
const TRANSPARENT: Color = Color {
    a: 0,
    r: 0,
    g: 0,
    b: 0,
};

/// The automation id of the body's empty part, below the last row. A click
/// there reports `None` to [`Table::selection`].
pub const EMPTY_AREA: &str = "guinea-widgets.table.empty";

/// What each column is currently wide, by the name of its mark.
///
/// Plain data, and the whole point of it: the table draws with this and never
/// writes to it. A drag is reported and the owner decides. By name rather than
/// by the mark itself, so what is saved is a string that outlives the enum.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnWidths(BTreeMap<&'static str, f64>);

impl ColumnWidths {
    /// What this column is wide, or `None` when it has never been dragged.
    ///
    /// `None` rather than a guess: the answer for a column nobody has touched
    /// is what that column declared, and only the column knows it. A default
    /// `ColumnWidths` is therefore a complete answer - a page that never
    /// persists widths starts with one and needs no `init`.
    pub fn get(&self, id: &str) -> Option<f64> {
        self.0.get(id).copied()
    }

    /// Applies a drag. What a page's `update` calls when [`Resized`] arrives.
    pub fn apply(&mut self, drag: Resized) {
        self.0.insert(drag.column, drag.width);
    }
}

/// A column boundary dragged to a new width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resized {
    /// The name of the column's mark.
    pub column: &'static str,
    pub width: f64,
}

/// The order the columns are drawn in, by the names of their marks.
///
/// Plain data, like [`ColumnWidths`]: the table draws with it, and a header
/// dragged along the row is reported as [`Reordered`] for the owner to apply.
/// The first column the table was given stays first whatever this says; a
/// column this does not name keeps its place after the ones it does.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnOrder(Vec<&'static str>);

impl ColumnOrder {
    /// An order read back from wherever it was kept.
    pub fn new(names: Vec<&'static str>) -> Self {
        Self(names)
    }

    pub fn names(&self) -> &[&'static str] {
        &self.0
    }

    /// Applies a move. What a page's `update` calls when [`Reordered`]
    /// arrives.
    pub fn apply(&mut self, moved: Reordered) {
        self.0 = moved.order;
    }
}

/// A header dragged past its neighbour: every column, by the name of its
/// mark, in the order they now go.
#[derive(Clone, Debug, PartialEq)]
pub struct Reordered {
    pub order: Vec<&'static str>,
}

/// How the table paints what is not content: the rows' height, the plate
/// under a row that is pointed at or selected, and the header's lines.
///
/// Colours rather than theme brushes. The reactor has no token for a subtle
/// fill or a divider, and the colour scheme is observed once per window, by
/// the application - so the table cannot know which of the two it is drawn in,
/// and whoever does passes the colours for it. The defaults are a grey that
/// reads in both.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// The least a row is tall. Cells may make it taller, never shorter.
    pub row_height: f64,
    /// The plate under the row the pointer is over.
    pub hovered: Color,
    /// The plate under the selected row. It wins over `hovered`.
    pub selected: Color,
    /// How far the plate stands off the row's edges, across and down.
    pub inset: (f64, f64),
    pub radius: f64,
    /// The lines between columns in the header.
    pub separator: Color,
    /// The line under the header.
    pub rule: Color,
}

impl Default for Look {
    fn default() -> Self {
        let grey = |a| Color {
            a,
            r: 128,
            g: 128,
            b: 128,
        };

        Self {
            row_height: 32.0,
            hovered: grey(24),
            selected: grey(40),
            inset: (4.0, 1.0),
            radius: 4.0,
            separator: grey(48),
            rule: grey(48),
        }
    }
}

/// One column: what it is called, how it heads the table, and what it shows
/// for a row.
///
/// `C` is the application's enum of columns. A column's mark is also its sort
/// key and, by name, its key in [`ColumnWidths`] - and it is put on its header
/// cell and on its cell in every row, where a test finds them.
pub struct ColumnSpec<T, C> {
    pub id: C,
    pub header: Rc<dyn Fn() -> View>,
    pub initial_width: f64,
    pub min_width: f64,
    pub sortable: bool,
    pub flush: bool,
    pub fill: bool,
    pub cell: Rc<dyn Fn(&T) -> View>,
}

impl<T, C: Mark> ColumnSpec<T, C> {
    pub fn new(
        id: C,
        header: impl Into<String>,
        initial_width: f64,
        cell: impl Fn(&T) -> View + 'static,
    ) -> Self {
        let header = header.into();
        Self {
            id,
            header: Rc::new(move || TextBlock::new().text(header.clone()).into()),
            initial_width,
            min_width: MIN_COLUMN_WIDTH,
            sortable: false,
            flush: false,
            fill: false,
            cell: Rc::new(cell),
        }
    }

    /// Use an arbitrary view as the column header instead of plain text. The
    /// factory is called on every draw, so it may depend on state.
    pub fn new_with_header(
        id: C,
        header: impl Fn() -> View + 'static,
        initial_width: f64,
        cell: impl Fn(&T) -> View + 'static,
    ) -> Self {
        Self {
            id,
            header: Rc::new(header),
            initial_width,
            min_width: MIN_COLUMN_WIDTH,
            sortable: false,
            flush: false,
            fill: false,
            cell: Rc::new(cell),
        }
    }

    /// Sets the minimum width the resize handle enforces.
    pub fn min_width(mut self, min_width: f64) -> Self {
        self.min_width = min_width;
        self
    }

    /// Makes the header clickable and shows the sort indicator when the table
    /// is given a matching [`SortState`]. The sort id is the column `id`.
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Drops [`CELL_HORIZONTAL_PADDING`] for this column, header and body
    /// alike, so its content starts at the column's own left edge.
    ///
    /// For a column that reserves its own left gutter - a tree column with a
    /// chevron slot, say - the shared inset is a second gutter on top of the
    /// first, and the two of them push the content visibly off the table's
    /// edge.
    pub fn flush(mut self) -> Self {
        self.flush = true;
        self
    }

    /// Gives this column whatever width the table has left over, so there is
    /// no empty band after the last column. Several such columns share it.
    ///
    /// It never goes below [`min_width`](Self::min_width): a table too narrow
    /// for that overflows, as a table of fixed columns does. And it has no
    /// resize handle - its width is what the others leave, and the table is
    /// never told what that came to, so a drag would have nothing to start
    /// from. Its neighbours' handles move it instead.
    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }
}

/// A table being described. Nothing is drawn until [`build`](Table::build).
pub struct Table<T, C> {
    rows: Vec<T>,
    columns: Vec<ColumnSpec<T, C>>,
    widths: ColumnWidths,
    on_resize: Option<Callback<Resized>>,
    order: ColumnOrder,
    on_reorder: Option<Callback<Reordered>>,
    sort: Option<(SortState<C>, Callback<C>)>,
    selection: Option<(Option<usize>, Callback<Option<usize>>)>,
    span: Option<Range<usize>>,
    sort_indicator: Option<Rc<dyn Fn(bool) -> View>>,
    look: Look,
    corner_radius: f64,
}

/// A table of `rows`, one per line, in the order given.
///
/// Rows have no identity of their own: the row drawn at a line is whatever
/// sits at that index now. A list re-sorted every tick then changes what the
/// visible lines show and nothing else, where rows keyed by an id made the
/// reactor reset the whole collection - blanking the body and losing the
/// scroll position each time.
pub fn table<T: 'static, C: Mark + Clone + PartialEq>(
    rows: Vec<T>,
    columns: Vec<ColumnSpec<T, C>>,
) -> Table<T, C> {
    Table {
        widths: ColumnWidths::default(),
        rows,
        columns,
        on_resize: None,
        order: ColumnOrder::default(),
        on_reorder: None,
        sort: None,
        selection: None,
        span: None,
        sort_indicator: None,
        look: Look::default(),
        corner_radius: 0.0,
    }
}

impl<T: 'static, C: Mark + Clone + PartialEq> Table<T, C> {
    /// The widths to draw with. Without this the table uses what the columns
    /// declared, which is right for a table nobody can resize.
    pub fn widths(mut self, widths: &ColumnWidths) -> Self {
        self.widths = widths.clone();
        self
    }

    /// What to paint around the content with. See [`Look`].
    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    /// The radius of the corners of whatever the table sits in. A header
    /// cell lit under the pointer rounds the table's corners it touches to
    /// it: the first cell its top left, the last its top right when a column
    /// fills the table to its right edge. Square by default.
    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Where a drag goes. Without it the handles are not drawn at all - a
    /// handle that reported to nobody would move and snap back. A column's
    /// handle carries the automation id `{mark name}/resize`.
    ///
    /// Takes what every reactor widget takes: a plain closure, or a `Callback`
    /// a segment already made. The second is the usual one - `cx.on(..)` seals
    /// the drag as one of the page's own messages, and passing it straight in
    /// beats wrapping it in a closure that calls it and throws the answer away.
    pub fn on_resize(mut self, on_resize: impl IntoPayloadCallback<Resized>) -> Self {
        self.on_resize = Some(on_resize.into_payload_callback());
        self
    }

    /// The order to draw the columns in. Without it they go in the order they
    /// were given.
    pub fn order(mut self, order: &ColumnOrder) -> Self {
        self.order = order.clone();
        self
    }

    /// Where a header dragged along the row goes. With it, a header held down
    /// and moved past the middle of its neighbour trades places with it, and
    /// a press that moved is not a click, so it does not sort. The first
    /// column does not move, and neither does one that fills - its width is
    /// what the others leave, so there is no middle to pass.
    pub fn on_reorder(mut self, on_reorder: impl IntoPayloadCallback<Reordered>) -> Self {
        self.on_reorder = Some(on_reorder.into_payload_callback());
        self
    }

    /// Which column the rows are sorted by, and where a click on a sortable
    /// header goes - with that column's mark.
    pub fn sort(mut self, state: SortState<C>, on_sort: impl IntoPayloadCallback<C>) -> Self {
        self.sort = Some((state, on_sort.into_payload_callback()));
        self
    }

    pub fn selection(
        mut self,
        at: Option<usize>,
        on_select: impl IntoPayloadCallback<Option<usize>>,
    ) -> Self {
        self.selection = Some((at, on_select.into_payload_callback()));
        self
    }

    /// Paints a run of rows as selected together, as one block: the corners
    /// round at its ends only, and the plates meet between its rows.
    ///
    /// Painting, not selecting. It takes the place of the row
    /// [`selection`](Self::selection) would paint, and a click still reports
    /// the one row it landed on - what that row stands for, a group or a
    /// member of one, is the caller's to decide.
    pub fn selected_span(mut self, rows: Range<usize>) -> Self {
        self.span = Some(rows);
        self
    }

    /// Renders the sort direction indicator, instead of the plain glyph.
    ///
    /// guinea depends on no icon set and cannot bundle one, so this is how a
    /// themed icon gets in. The closure is handed `descending`.
    pub fn sort_indicator(mut self, render: impl Fn(bool) -> View + 'static) -> Self {
        self.sort_indicator = Some(Rc::new(render));
        self
    }

    pub fn build(self) -> View {
        let Self {
            rows,
            columns,
            widths,
            on_resize,
            order,
            on_reorder,
            sort,
            selection,
            span,
            sort_indicator,
            look,
            corner_radius,
        } = self;

        let (sort_state, on_sort) = match sort {
            Some((state, callback)) => (Some(state), Some(callback)),
            None => (None, None),
        };

        let placed = placed(&columns, &order);
        let slots = slots(&placed);

        // Keyed by column rather than positional, so a column is still itself
        // when the handles come and go with `on_resize`. And kept in the order
        // the columns were given, whatever order they are drawn in: a move
        // changes which grid column a cell stands in, never which child it
        // is, so the header being dragged is not rebuilt under the pointer.
        let last = columns.len().saturating_sub(1);
        let reaches_right = columns.iter().any(|column| column.fill);
        let mut header_cells: Vec<KeyedView> = Vec::with_capacity(columns.len() * 2);
        let mut handles: Vec<KeyedView> = Vec::with_capacity(columns.len());
        for (at, column) in columns.iter().enumerate() {
            let slot = slots[at];
            let rounded = (
                if slot == 0 { corner_radius } else { 0.0 },
                if slot == last && reaches_right {
                    corner_radius
                } else {
                    0.0
                },
            );
            let railed = on_resize.is_some() && slot != last && !column.fill;

            let moving = on_reorder
                .as_ref()
                .filter(|_| slot != 0 && !column.fill)
                .map(|on_reorder| {
                    let beside = |slot: usize| {
                        let neighbour = &columns[placed[slot]];
                        (slot != 0 && !neighbour.fill).then(|| width_of(&widths, neighbour))
                    };
                    Moving {
                        order: placed.iter().map(|&at| columns[at].id.name()).collect(),
                        at: slot,
                        left: beside(slot - 1),
                        right: (slot < last).then(|| beside(slot + 1)).flatten(),
                        on_reorder: on_reorder.clone(),
                    }
                });

            let cell = header_cell(
                column,
                sort_state.as_ref(),
                on_sort.as_ref(),
                sort_indicator.as_ref(),
                moving,
                look.hovered,
                rounded,
                railed,
            );

            header_cells.push(keyed(
                column.id.name().to_string(),
                Border::new().grid_column(slot as i32).content(cell),
            ));

            // Over the column's right edge, not beside it. A handle standing
            // in the row between two header cells took its width from the row,
            // and the body has no handles - so every header cell stood one
            // handle further right of its column than the last, and the values
            // stopped sitting under their headings.
            if let Some(on_resize) = &on_resize
                && railed
            {
                let key = format!("{}/resize", column.id.name());
                handles.push(keyed(
                    key.clone(),
                    Border::new()
                        .automation_id(key)
                        .grid_column(slot as i32)
                        .width(RESIZE_HANDLE_WIDTH)
                        .horizontal_alignment(HorizontalAlignment::Right)
                        .margin(Thickness::new(
                            0.0,
                            0.0,
                            0.5 - RESIZE_HANDLE_WIDTH / 2.0,
                            0.0,
                        ))
                        .content(handle(
                            column,
                            width_of(&widths, column),
                            look.separator,
                            on_resize.clone(),
                        )),
                ));
            }
        }
        header_cells.extend(handles);

        let (lengths, least) = lengths(&columns, &placed, &widths);
        let header = Grid::new()
            .columns(lengths.clone())
            .min_width(least)
            .grid_row(0)
            .keyed_children(header_cells);

        let separator = Rectangle::new().fill(look.rule).height(1.0).grid_row(1);

        // No horizontal scrolling. Allowed to scroll across, the viewer
        // measures the rows as wide as they like, and a column that fills
        // then has no leftover to fill and shrinks to its content, row by row.
        // Nothing is lost: the header stands outside the viewer, so scrolling
        // the rows across only ever slid them out from under their headings.
        // A table too narrow for its columns is clipped instead, header and
        // rows alike.
        let on_deselect = selection.as_ref().map(|(_, on_select)| on_select.clone());

        let laid = Laid {
            slots,
            lengths,
            least,
        };
        let lines = ItemsRepeater::new()
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .virtual_source(rows_source(rows, columns, laid, selection, span, look));

        let content: View = match on_deselect {
            Some(on_deselect) => Grid::new()
                .children((
                    Border::new()
                        .automation_id(EMPTY_AREA)
                        .background(TRANSPARENT)
                        .horizontal_alignment(HorizontalAlignment::Stretch)
                        .vertical_alignment(VerticalAlignment::Stretch)
                        .on_pointer_released(Callback::new(move |_: PointerEventInfo| {
                            on_deselect.call(None);
                        })),
                    lines,
                ))
                .into(),
            None => lines.into(),
        };

        let body = ScrollViewer::new()
            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
            .grid_row(2)
            .content(content);

        Grid::new()
            .rows([GridLength::Auto, GridLength::Auto, GridLength::Star(1.0)])
            .children((header, separator, body))
            .into()
    }
}

/// How many rows the source claims at a time.
///
/// The count is rounded up to this, and the rows past the data are empty and
/// have no height. See [`rows_source`] for why.
const BUCKET: usize = 16;

/// The rows, built only for what is on screen.
///
/// Keyed by index, so a re-sort leaves the keys alone: the row drawn at a
/// line is whatever sits at that index now, and the line itself never moves.
///
/// The count is rounded up to [`BUCKET`], and the rows past the data are
/// empty ones of no height. The reason is how the reactor treats a change of
/// keys: any change at all retires every realized row and resets the
/// collection - there is no path that moves a container - so a list whose
/// length is its key set resets every time a process starts or exits, which
/// on a process list is most seconds. Rounded, the key set survives the
/// comings and goings within a bucket, and what the rows show is updated in
/// place.
///
/// What the padding costs is scroll past the end. The repeater estimates the
/// part of the list it has not realized from the height of the part it has,
/// and near the top that estimate counts the empty rows as full ones - so the
/// list scrolls a little beyond its last row, and the slack shrinks but never
/// quite closes as the empty rows realize. That is what sets the bucket: at
/// 64 the slack was a whole viewport of nothing, at 16 it is a few rows.
fn rows_source<T: 'static, C: Mark>(
    rows: Vec<T>,
    columns: Vec<ColumnSpec<T, C>>,
    laid: Laid,
    selection: Option<(Option<usize>, Callback<Option<usize>>)>,
    span: Option<Range<usize>>,
    look: Look,
) -> VirtualSource {
    let len = rows.len();
    let claimed = len.next_multiple_of(BUCKET);
    let rows = Rc::new(rows);
    let columns = Rc::new(columns);
    let laid = Rc::new(laid);
    let (selected, on_select) = match selection {
        Some((at, callback)) => (at, Some(callback)),
        None => (None, None),
    };

    // One row selected is a run of one: the same plate, rounded at both ends.
    let painted = span.or_else(|| selected.map(|at| at..at + 1));

    VirtualSource::new(
        claimed as u64,
        claimed,
        |index| index,
        move |index| match rows.get(index) {
            Some(row) => View::component::<Pointed>(Line {
                cells: row_view(row, &columns, &laid),
                index,
                selected: painted
                    .as_ref()
                    .filter(|run| run.contains(&index))
                    .map(|run| Run {
                        above: index > run.start,
                        below: index + 1 < run.end,
                    }),
                on_select: on_select.clone(),
                look,
            }),
            None => Border::new().into(),
        },
    )
}

/// What makes a row a row rather than a strip of cells: its height, the
/// plate under it, and the click that selects it.
///
/// Drawn here because the rows no longer sit in a `ListView`, which used to
/// do all three. A component for the one thing only the row knows - whether
/// the pointer is over it. That is not the page's business: a message to the
/// page for every row the pointer crosses would rebuild the whole page to
/// move a highlight.
#[derive(Clone, PartialEq)]
struct Line {
    cells: View,
    index: usize,
    /// Where the row sits in the run painted as selected, if it is in one.
    selected: Option<Run>,
    on_select: Option<Callback<Option<usize>>>,
    look: Look,
}

/// Whether a row's plate meets the plates of the rows above and below it.
#[derive(Clone, Copy, PartialEq)]
struct Run {
    above: bool,
    below: bool,
}

impl Run {
    const ALONE: Run = Run {
        above: false,
        below: false,
    };
}

enum Pointer {
    Entered,
    Exited,
}

/// Whether the pointer is over the row.
struct Pointed(bool);

impl Component for Pointed {
    type Input = Line;
    type Message = Pointer;

    fn create(_input: &Line, _cx: &ComponentContext<Self>) -> Self {
        Self(false)
    }

    fn update(&mut self, message: Pointer, _cx: &ComponentContext<Self>) {
        self.0 = matches!(message, Pointer::Entered);
    }

    fn input_changed(&mut self, _input: &Line, _cx: &ComponentContext<Self>) {}

    fn view(&self, line: &Line, cx: &mut ViewContext<Self>) -> View {
        let look = line.look;

        let (plate, run) = match line.selected {
            Some(run) => (look.selected, run),
            None if self.0 => (look.hovered, Run::ALONE),
            None => (TRANSPARENT, Run::ALONE),
        };

        // Where the plate meets a neighbour's, it runs to the row's edge and
        // stays square, so a run of selected rows reads as one block.
        let (across, down) = look.inset;
        let top = if run.above { 0.0 } else { down };
        let bottom = if run.below { 0.0 } else { down };
        let upper = if run.above { 0.0 } else { look.radius };
        let lower = if run.below { 0.0 } else { look.radius };

        // Under the cells rather than behind the row: inset, so it reads as a
        // plate and not a stripe, with the cells still where the header's
        // columns are.
        let layered = Grid::new().children((
            Border::new()
                .margin(Thickness::new(across, top, across, bottom))
                .corner_radius(CornerRadius::new(upper, upper, lower, lower))
                .background(plate),
            line.cells.clone(),
        ));

        // The background is always set, transparent at rest, so the whole
        // row takes the pointer - the gaps between cells and around the
        // plate included, or the highlight would flicker between rows.
        let row = Border::new()
            .min_height(look.row_height)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .background(TRANSPARENT)
            .on_pointer_entered(cx.callback(|_: PointerEventInfo| Pointer::Entered))
            .on_pointer_exited(cx.callback(|_: PointerEventInfo| Pointer::Exited));

        match &line.on_select {
            Some(on_select) => {
                let on_select = on_select.clone();
                let index = line.index;

                row.on_pointer_released(Callback::new(move |_: PointerEventInfo| {
                    on_select.call(Some(index));
                }))
                .content(layered)
                .into()
            }
            None => row.content(layered).into(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn header_cell<T, C: Mark + Clone + PartialEq>(
    column: &ColumnSpec<T, C>,
    sort_state: Option<&SortState<C>>,
    on_sort: Option<&Callback<C>>,
    sort_indicator: Option<&Rc<dyn Fn(bool) -> View>>,
    moving: Option<Moving>,
    hovered: Color,
    rounded: (f64, f64),
    railed: bool,
) -> View {
    let active = sort_state.filter(|s| column.sortable && s.field_id.as_ref() == Some(&column.id));

    let base = (column.header)();
    let content = match active {
        Some(state) => {
            let indicator = match sort_indicator {
                Some(render) => render(state.descending),
                None => TextBlock::new()
                    .text(if state.descending { "▼" } else { "▲" })
                    .into(),
            };
            // A grid rather than a horizontal stack: a stack gives the header
            // only what it asks for, so a header that aligns its own content
            // - right, or across two lines - was squeezed to its text and
            // drifted left in the one column that was sorted. The first
            // column takes the cell, the indicator what it needs, and an
            // empty indicator nothing.
            Grid::new()
                .columns([GridLength::Star(1.0), GridLength::Auto])
                .children((
                    Border::new().grid_column(0).content(base),
                    Border::new().grid_column(1).content(indicator),
                ))
                .into()
        }
        None => base,
    };

    // Kept as a `Border` rather than collapsed to a bare view: padding is a
    // capability of the widget, and an erased node has none. The width is the
    // grid column's.
    let padding = Thickness::xy(
        if column.flush {
            0.0
        } else {
            CELL_HORIZONTAL_PADDING
        },
        HEADER_VERTICAL_PADDING,
    );

    let sorts = on_sort.filter(|_| column.sortable).cloned();
    if sorts.is_none() && moving.is_none() {
        return Border::new()
            .automation_id(column.id.name())
            .padding(padding)
            .content(content)
            .into();
    }

    View::component::<PointedHeading<C>>(Heading {
        content: Border::new().padding(padding).content(content).into(),
        column: column.id.clone(),
        on_sort: sorts,
        moving,
        hovered,
        rounded,
        railed,
    })
}

/// How far a pressed header has to travel before the press is a drag and not
/// a click.
const DRAG_THRESHOLD: f64 = 4.0;

#[derive(Clone, PartialEq)]
struct Heading<C> {
    content: View,
    column: C,
    on_sort: Option<Callback<C>>,
    moving: Option<Moving>,
    hovered: Color,
    rounded: (f64, f64),
    railed: bool,
}

/// What a header needs to be dragged along the row: every column in the order
/// drawn, its own slot among them, how wide the neighbours it may trade
/// places with are - `None` for one it may not pass - and where a move goes.
#[derive(Clone, PartialEq)]
struct Moving {
    order: Vec<&'static str>,
    at: usize,
    left: Option<f64>,
    right: Option<f64>,
    on_reorder: Callback<Reordered>,
}

impl Moving {
    /// The move `delta` from where the drag started calls for, if any: the
    /// order after it, and how far the header's own place moved with it.
    fn step(&self, delta: f64) -> Option<(Vec<&'static str>, f64)> {
        let (to, shift) = match (self.left, self.right) {
            (_, Some(right)) if delta > right / 2.0 => (self.at + 1, right),
            (Some(left), _) if delta < -left / 2.0 => (self.at - 1, -left),
            _ => return None,
        };

        let mut order = self.order.clone();
        order.swap(self.at, to);
        Some((order, shift))
    }
}

/// A drag in progress, shared with the pointer callbacks, which run between
/// renders and have to see what the one before them did.
#[derive(Default)]
struct Drag {
    /// Where in the window the header's place began when pressed, moved on
    /// by every trade since. Gone once the capture is lost: the drag stops
    /// trading there, while the release that follows still counts.
    anchor: Cell<Option<f64>>,
    /// The press has travelled far enough to be a drag, so its release does
    /// not sort.
    moved: Cell<bool>,
    /// The slot a trade was last reported from. Until the table draws the
    /// header in its new slot the pointer keeps moving against the old one,
    /// and would report the same trade again.
    sent_from: Cell<Option<usize>>,
}

struct PointedHeading<C> {
    hovered: bool,
    drag: Rc<Drag>,
    _column: std::marker::PhantomData<fn() -> C>,
}

impl<C: Mark + Clone + PartialEq + 'static> Component for PointedHeading<C> {
    type Input = Heading<C>;
    type Message = Pointer;

    fn create(_input: &Heading<C>, _cx: &ComponentContext<Self>) -> Self {
        Self {
            hovered: false,
            drag: Rc::default(),
            _column: std::marker::PhantomData,
        }
    }

    fn update(&mut self, message: Pointer, _cx: &ComponentContext<Self>) {
        self.hovered = matches!(message, Pointer::Entered);
    }

    fn view(&self, heading: &Heading<C>, cx: &mut ViewContext<Self>) -> View {
        let plate = if self.hovered {
            heading.hovered
        } else {
            TRANSPARENT
        };
        let (left, right) = heading.rounded;
        let rail = if heading.railed { 1.0 } else { 0.0 };

        let layered = Grid::new().children((
            Border::new()
                .margin(Thickness::new(0.0, 0.0, rail, 0.0))
                .corner_radius(CornerRadius::new(left, right, 0.0, 0.0))
                .background(plate),
            heading.content.clone(),
        ));

        let column = heading.column.clone();
        let on_sort = heading.on_sort.clone();
        let released = self.drag.clone();

        let cell = Border::new()
            .automation_id(heading.column.name())
            .background(TRANSPARENT)
            .on_pointer_entered(cx.callback(|_: PointerEventInfo| Pointer::Entered))
            .on_pointer_exited(cx.callback(|_: PointerEventInfo| Pointer::Exited))
            .on_pointer_released(Callback::new(move |_: PointerEventInfo| {
                released.anchor.set(None);
                if released.moved.replace(false) {
                    return;
                }
                if let Some(on_sort) = &on_sort {
                    on_sort.call(column.clone());
                }
            }));

        let Some(moving) = heading.moving.clone() else {
            return cell.content(layered).into();
        };

        let pressed = self.drag.clone();
        let dragged = self.drag.clone();
        let lost = self.drag.clone();
        cell.capture_pointer_on_press(true)
            .on_pointer_capture_lost(move || lost.anchor.set(None))
            .on_pointer_pressed(Callback::new(move |info: PointerEventInfo| {
                pressed.anchor.set(Some(info.window_x));
                pressed.moved.set(false);
                pressed.sent_from.set(None);
            }))
            .on_pointer_moved(Callback::new(move |info: PointerEventInfo| {
                let Some(anchor) = dragged.anchor.get().filter(|_| info.is_left_button_pressed)
                else {
                    return;
                };
                let delta = info.window_x - anchor;
                if delta.abs() > DRAG_THRESHOLD {
                    dragged.moved.set(true);
                }
                if !dragged.moved.get() || dragged.sent_from.get() == Some(moving.at) {
                    return;
                }

                if let Some((order, shift)) = moving.step(delta) {
                    moving.on_reorder.call(Reordered { order });
                    dragged.anchor.set(Some(anchor + shift));
                    dragged.sent_from.set(Some(moving.at));
                }
            }))
            .content(layered)
            .into()
    }
}

/// What this column is drawn at: what it was dragged to, or what it declared.
fn width_of<T, C: Mark>(widths: &ColumnWidths, column: &ColumnSpec<T, C>) -> f64 {
    widths
        .get(column.id.name())
        .unwrap_or(column.initial_width)
        .max(column.min_width)
}

/// The columns as the header and every row lay them out, and the least a
/// row can be wide.
///
/// The header and the rows are separate grids, each finding its own share for
/// the columns that [`fill`](ColumnSpec::fill); they agree because they are
/// handed the same width. The least width stands in for a per-column minimum,
/// which a reactor grid column cannot have: narrower than that, the row is
/// laid out at it and overflows, the way a row of fixed columns always did.
fn lengths<T, C: Mark>(
    columns: &[ColumnSpec<T, C>],
    placed: &[usize],
    widths: &ColumnWidths,
) -> (Vec<GridLength>, f64) {
    let lengths = placed
        .iter()
        .map(|&at| &columns[at])
        .map(|column| {
            if column.fill {
                GridLength::Star(1.0)
            } else {
                GridLength::Pixel(width_of(widths, column))
            }
        })
        .collect();

    let least = columns
        .iter()
        .map(|column| {
            if column.fill {
                column.min_width
            } else {
                width_of(widths, column)
            }
        })
        .sum();

    (lengths, least)
}

fn handle<T, C: Mark>(
    column: &ColumnSpec<T, C>,
    width: f64,
    rail: Color,
    on_resize: Callback<Resized>,
) -> View {
    let id = column.id.name();
    // The one place a closure is still the right shape: the handle reports a
    // width and the table turns it into a `Resized` for the column it belongs
    // to, which is a mapping rather than a hand-off.
    resize_handle(width, move |width| {
        // A drop here means the drag outlived the publication that started it,
        // and the column simply stays where it was.
        on_resize.call(Resized { column: id, width });
    })
    .min(column.min_width)
    .rail(rail)
    .build()
}

/// Which column is drawn in each slot, left to right, by its index among the
/// columns given: the first one always first, the rest as `order` names them,
/// and the ones it does not name after those, as they were given.
fn placed<T, C: Mark>(columns: &[ColumnSpec<T, C>], order: &ColumnOrder) -> Vec<usize> {
    let mut placed: Vec<usize> = (0..columns.len()).collect();
    if let Some((_, rest)) = placed.split_first_mut() {
        rest.sort_by_key(|&at| {
            let name = columns[at].id.name();
            order
                .names()
                .iter()
                .position(|named| *named == name)
                .unwrap_or(usize::MAX)
        });
    }
    placed
}

/// The slot each column is drawn in, by its index among the columns given.
fn slots(placed: &[usize]) -> Vec<usize> {
    let mut slots = vec![0; placed.len()];
    for (slot, &at) in placed.iter().enumerate() {
        slots[at] = slot;
    }
    slots
}

/// Where each column stands and how wide the grid's columns are, worked out
/// once for the header and every row.
struct Laid {
    slots: Vec<usize>,
    lengths: Vec<GridLength>,
    least: f64,
}

fn row_view<T, C: Mark>(row: &T, columns: &[ColumnSpec<T, C>], laid: &Laid) -> View {
    let cells: Vec<KeyedView> = columns
        .iter()
        .enumerate()
        .map(|(at, column)| {
            // The column renders whatever it likes, so what comes back is
            // erased - wrap it in the thing that carries padding and a place
            // in the grid.
            let cell = Border::new()
                .automation_id(column.id.name())
                .padding(Thickness::xy(
                    if column.flush {
                        0.0
                    } else {
                        CELL_HORIZONTAL_PADDING
                    },
                    0.0,
                ))
                .grid_column(laid.slots[at] as i32)
                .vertical_alignment(VerticalAlignment::Center)
                .content((column.cell)(row));

            keyed(column.id.name().to_string(), cell)
        })
        .collect();

    Grid::new()
        .columns(laid.lengths.clone())
        .min_width(laid.least)
        .keyed_children(cells)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    enum Col {
        Name,
        Size,
        Kind,
        Date,
    }

    impl Mark for Col {
        fn name(&self) -> &'static str {
            match self {
                Col::Name => "Name",
                Col::Size => "Size",
                Col::Kind => "Kind",
                Col::Date => "Date",
            }
        }
    }

    fn columns() -> Vec<ColumnSpec<(), Col>> {
        [Col::Name, Col::Size, Col::Kind, Col::Date]
            .into_iter()
            .map(|id| ColumnSpec::new(id, id.name(), 100.0, |_: &()| Grid::new().into()))
            .collect()
    }

    fn moving(at: usize, left: Option<f64>, right: Option<f64>) -> Moving {
        Moving {
            order: vec!["Name", "Size", "Kind", "Date"],
            at,
            left,
            right,
            on_reorder: Callback::new(|_: Reordered| {}),
        }
    }

    #[test]
    fn with_no_order_the_columns_go_as_given() {
        assert_eq!(placed(&columns(), &ColumnOrder::default()), [0, 1, 2, 3]);
    }

    #[test]
    fn the_order_places_every_column_but_the_first() {
        let order = ColumnOrder::new(vec!["Date", "Name", "Kind", "Size"]);
        assert_eq!(placed(&columns(), &order), [0, 3, 2, 1]);
    }

    #[test]
    fn a_column_the_order_does_not_name_goes_after_the_named() {
        let order = ColumnOrder::new(vec!["Kind", "Gone"]);
        assert_eq!(placed(&columns(), &order), [0, 2, 1, 3]);
    }

    #[test]
    fn slots_are_where_each_column_was_placed() {
        assert_eq!(slots(&[0, 3, 1, 2]), [0, 2, 3, 1]);
    }

    #[test]
    fn a_header_trades_places_past_half_its_neighbour() {
        let middle = moving(2, Some(80.0), Some(60.0));
        assert_eq!(middle.step(29.0), None);
        assert_eq!(
            middle.step(31.0),
            Some((vec!["Name", "Size", "Date", "Kind"], 60.0))
        );
        assert_eq!(middle.step(-39.0), None);
        assert_eq!(
            middle.step(-41.0),
            Some((vec!["Name", "Kind", "Size", "Date"], -80.0))
        );
    }

    #[test]
    fn a_header_does_not_pass_a_neighbour_it_may_not() {
        let second = moving(1, None, Some(60.0));
        assert_eq!(second.step(-500.0), None);
        let last = moving(3, Some(60.0), None);
        assert_eq!(last.step(500.0), None);
    }
}
