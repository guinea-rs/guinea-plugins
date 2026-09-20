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

use std::collections::BTreeMap;
use std::rc::Rc;

use windows_reactor::{
    Border, Callback, ChildrenControl, Color, ContentControl, Grid, GridChildExt, GridLength,
    HorizontalAlignment, IntoPayloadCallback, ItemsRepeater, LayoutControl, Orientation, PointerEventInfo, Rectangle,
    ScrollViewer, StackPanel, TextBlock, Thickness, VerticalAlignment, View, VirtualSource,
};

use crate::resize::resize_handle;

const MIN_COLUMN_WIDTH: f64 = 24.0;
const HEADER_SEPARATOR_COLOR: Color = Color {
    a: 48,
    r: 128,
    g: 128,
    b: 128,
};

/// Horizontal inset shared by header and body cells so column content never
/// sits flush against the resize handle or the row edge. Opt out per column
/// with [`ColumnSpec::flush`].
const CELL_HORIZONTAL_PADDING: f64 = 12.0;
/// Extra vertical room for the header row - it can carry two-line content
/// (label + aggregate value) where body rows stay a single fixed height.
const HEADER_VERTICAL_PADDING: f64 = 8.0;
/// A body row's height, the one a `ListView` item used to impose.
const ROW_HEIGHT: f64 = 32.0;
const SELECTED_ROW_COLOR: Color = Color {
    a: 40,
    r: 128,
    g: 128,
    b: 128,
};
const TRANSPARENT: Color = Color {
    a: 0,
    r: 0,
    g: 0,
    b: 0,
};

/// What each column is currently wide, by column id.
///
/// Plain data, and the whole point of it: the table draws with this and never
/// writes to it. A drag is reported and the owner decides.
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
    pub column: &'static str,
    pub width: f64,
}

pub struct ColumnSpec<T> {
    pub id: &'static str,
    pub header: Rc<dyn Fn() -> View>,
    pub initial_width: f64,
    pub min_width: f64,
    pub sortable: bool,
    pub flush: bool,
    pub cell: Rc<dyn Fn(&T) -> View>,
}

impl<T> ColumnSpec<T> {
    pub fn new(
        id: &'static str,
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
            cell: Rc::new(cell),
        }
    }

    /// Use an arbitrary view as the column header instead of plain text. The
    /// factory is called on every draw, so it may depend on state.
    pub fn new_with_header(
        id: &'static str,
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
}

/// A table being described. Nothing is drawn until [`build`](Table::build).
pub struct Table<T> {
    rows: Vec<T>,
    columns: Vec<ColumnSpec<T>>,
    widths: ColumnWidths,
    on_resize: Option<Callback<Resized>>,
    sort: Option<(SortState<String>, Callback<String>)>,
    selection: Option<(Option<usize>, Callback<Option<usize>>)>,
    sort_indicator: Option<Rc<dyn Fn(bool) -> View>>,
}

/// A table of `rows`, one per line, in the order given.
///
/// Rows have no identity of their own: the row drawn at a line is whatever
/// sits at that index now. A list re-sorted every tick then changes what the
/// visible lines show and nothing else, where rows keyed by an id made the
/// reactor reset the whole collection - blanking the body and losing the
/// scroll position each time.
pub fn table<T: 'static>(rows: Vec<T>, columns: Vec<ColumnSpec<T>>) -> Table<T> {
    Table {
        widths: ColumnWidths::default(),
        rows,
        columns,
        on_resize: None,
        sort: None,
        selection: None,
        sort_indicator: None,
    }
}

impl<T: 'static> Table<T> {
    /// The widths to draw with. Without this the table uses what the columns
    /// declared, which is right for a table nobody can resize.
    pub fn widths(mut self, widths: &ColumnWidths) -> Self {
        self.widths = widths.clone();
        self
    }

    /// Where a drag goes. Without it the handles are not drawn at all - a
    /// handle that reported to nobody would move and snap back.
    ///
    /// Takes what every reactor widget takes: a plain closure, or a `Callback`
    /// a segment already made. The second is the usual one - `cx.on(..)` seals
    /// the drag as one of the page's own messages, and passing it straight in
    /// beats wrapping it in a closure that calls it and throws the answer away.
    pub fn on_resize(mut self, on_resize: impl IntoPayloadCallback<Resized>) -> Self {
        self.on_resize = Some(on_resize.into_payload_callback());
        self
    }

    pub fn sort(
        mut self,
        state: SortState<String>,
        on_sort: impl IntoPayloadCallback<String>,
    ) -> Self {
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
            sort,
            selection,
            sort_indicator,
        } = self;

        let (sort_state, on_sort) = match sort {
            Some((state, callback)) => (Some(state), Some(callback)),
            None => (None, None),
        };

        // Keyed rather than positional: a column that gains a resize handle
        // when `on_resize` is given must not be mistaken for the column that
        // used to sit at that index.
        let last = columns.len().saturating_sub(1);
        let mut header_cells: Vec<(String, View)> = Vec::with_capacity(columns.len() * 2);
        for (at, column) in columns.iter().enumerate() {
            header_cells.push((
                column.id.to_string(),
                header_cell(
                    column,
                    width_of(&widths, column),
                    sort_state.as_ref(),
                    on_sort.as_ref(),
                    sort_indicator.as_ref(),
                ),
            ));

            if at != last && let Some(on_resize) = &on_resize {
                header_cells.push((
                    format!("{}::handle", column.id),
                    handle(column, width_of(&widths, column), on_resize.clone()),
                ));
            }
        }

        let header = StackPanel::new()
            .orientation(Orientation::Horizontal)
            .grid_row(0)
            .children((View::keyed_fragment(header_cells),));

        let separator = Rectangle::new()
            .fill(HEADER_SEPARATOR_COLOR)
            .height(1.0)
            .grid_row(1);

        let body = ScrollViewer::new()
            .grid_row(2)
            .content(
                ItemsRepeater::new()
                    .horizontal_alignment(HorizontalAlignment::Stretch)
                    .virtual_source(rows_source(rows, columns, widths, selection)),
            );

        Grid::new()
            .rows([GridLength::Auto, GridLength::Auto, GridLength::Star(1.0)])
            .children((header, separator, body))
    }
}

/// The rows, built only for what is on screen.
///
/// Keyed by index, so the keys are `0..len` and change only with the length:
/// the length is the whole revision, and every tick - re-sorted or not -
/// rebuilds the visible rows and nothing else.
fn rows_source<T: 'static>(
    rows: Vec<T>,
    columns: Vec<ColumnSpec<T>>,
    widths: ColumnWidths,
    selection: Option<(Option<usize>, Callback<Option<usize>>)>,
) -> VirtualSource {
    let len = rows.len();
    let rows = Rc::new(rows);
    let columns = Rc::new(columns);
    let (selected, on_select) = match selection {
        Some((at, callback)) => (at, Some(callback)),
        None => (None, None),
    };

    VirtualSource::new(
        len as u64,
        len,
        |index| index,
        move |index| {
            let cells = row_view(&rows[index], &columns, &widths);
            row_frame(cells, index, selected == Some(index), on_select.as_ref())
        },
    )
}

/// What makes a row a row rather than a strip of cells: its height, the
/// selection, and the click that selects it.
///
/// Drawn here because the rows no longer sit in a `ListView`, which used to
/// do all three. The background is always set, transparent when unselected,
/// so the gaps between cells take the click too.
fn row_frame(
    cells: View,
    index: usize,
    selected: bool,
    on_select: Option<&Callback<Option<usize>>>,
) -> View {
    let row = Border::new()
        .min_height(ROW_HEIGHT)
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .background(if selected { SELECTED_ROW_COLOR } else { TRANSPARENT });

    match on_select {
        Some(on_select) => {
            let on_select = on_select.clone();
            row.on_pointer_released(Callback::new(move |_: PointerEventInfo| {
                let _ = on_select.call(Some(index));
            }))
            .content(cells)
        }
        None => row.content(cells),
    }
}

fn header_cell<T>(
    column: &ColumnSpec<T>,
    width: f64,
    sort_state: Option<&SortState<String>>,
    on_sort: Option<&Callback<String>>,
    sort_indicator: Option<&Rc<dyn Fn(bool) -> View>>,
) -> View {
    let active = sort_state.filter(|s| column.sortable && s.field_id.as_deref() == Some(column.id));

    let base = (column.header)();
    let content = match active {
        Some(state) => {
            let indicator = match sort_indicator {
                Some(render) => render(state.descending),
                None => TextBlock::new()
                    .text(if state.descending { "▼" } else { "▲" })
                    .into(),
            };
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .children((base, indicator))
        }
        None => base,
    };

    // Kept as a `Border` rather than collapsed to a bare view: padding and
    // width are capabilities of the widget, and an erased node has neither.
    let cell = Border::new()
        .padding(Thickness::xy(
            if column.flush {
                0.0
            } else {
                CELL_HORIZONTAL_PADDING
            },
            HEADER_VERTICAL_PADDING,
        ))
        .width(width);

    match (column.sortable, on_sort) {
        (true, Some(on_sort)) => {
            let id = column.id.to_string();
            let on_sort = on_sort.clone();
            // `on_tapped` is gone; a release over the cell is the same gesture
            // for a header, and the only one a `Border` still offers.
            cell.on_pointer_released(Callback::new(move |_: PointerEventInfo| {
                // `false` means the segment that owns this table is not
                // publishing, so the sort would land nowhere.
                if !on_sort.call(id.clone()) {
                    tracing::debug!(column = %id, "sort dropped: no active publication");
                }
            }))
            .content(content)
        }
        _ => cell.content(content),
    }
}

/// What this column is drawn at: what it was dragged to, or what it declared.
fn width_of<T>(widths: &ColumnWidths, column: &ColumnSpec<T>) -> f64 {
    widths
        .get(column.id)
        .unwrap_or(column.initial_width)
        .max(column.min_width)
}

fn handle<T>(column: &ColumnSpec<T>, width: f64, on_resize: Callback<Resized>) -> View {
    let id = column.id;
    // The one place a closure is still the right shape: the handle reports a
    // width and the table turns it into a `Resized` for the column it belongs
    // to, which is a mapping rather than a hand-off.
    resize_handle(width, move |width| {
        // A drop here means the drag outlived the publication that started it,
        // and the column simply stays where it was.
        let _ = on_resize.call(Resized { column: id, width });
    })
    .min(column.min_width)
    .rail(HEADER_SEPARATOR_COLOR)
    .build()
}

fn row_view<T>(row: &T, columns: &[ColumnSpec<T>], widths: &ColumnWidths) -> View {
    let cells: Vec<(String, View)> = columns
        .iter()
        .map(|column| {
            // The column renders whatever it likes, so what comes back is
            // erased - wrap it in the thing that carries padding and width.
            let cell = Border::new()
                .padding(Thickness::xy(
                    if column.flush {
                        0.0
                    } else {
                        CELL_HORIZONTAL_PADDING
                    },
                    0.0,
                ))
                .width(width_of(widths, column))
                .vertical_alignment(VerticalAlignment::Center)
                .content((column.cell)(row));

            (column.id.to_string(), cell)
        })
        .collect();

    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .children((View::keyed_fragment(cells),))
}
