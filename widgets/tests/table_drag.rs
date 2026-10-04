#![cfg(all(windows, feature = "winui"))]

//! A table's header under the harness: a boundary dragged resizes its column,
//! and a header dragged along the row moves it - all but the first.

use guinea::app::Harness;
use guinea::prelude::*;
use guinea::winui::harness::{Drag, Mounted};
use guinea::winui::{Page, PageCx, UpdateCx, page};
use guinea_mark::Mark;
use guinea_widgets::table::{
    ColumnOrder, ColumnSpec, ColumnWidths, Reordered, Resized, SortState, table,
};
use windows_reactor::{Grid, StackPanel, TextBlock, View};

#[derive(Clone, Copy, PartialEq)]
pub enum Col {
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

struct Edge(&'static str);

impl Mark for Edge {
    fn name(&self) -> &'static str {
        self.0
    }
}

#[derive(Default)]
pub struct Columns {
    widths: ColumnWidths,
    order: ColumnOrder,
    sorted: Vec<&'static str>,
}

pub enum Changed {
    Resized(Resized),
    Reordered(Reordered),
    Sorted(Col),
}

#[page]
impl Page for Columns {
    type Params = ();
    type Installs = ();
    type Message = Changed;

    fn install(_ctx: &FeatureInitContext, _params: &()) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, message: Changed, _cx: &mut UpdateCx<'_, Self>) {
        match message {
            Changed::Resized(resized) => self.widths.apply(resized),
            Changed::Reordered(moved) => self.order.apply(moved),
            Changed::Sorted(column) => self.sorted.push(column.name()),
        }
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let columns = [Col::Name, Col::Size, Col::Kind, Col::Date]
            .into_iter()
            .map(|id| ColumnSpec::new(id, id.name(), 100.0, |_: &()| Grid::new().into()).sortable())
            .collect();
        let sort = SortState {
            field_id: None,
            descending: false,
        };

        let shown = format!(
            "order {} size {:?} sorted {}",
            self.order.names().join(","),
            self.widths.get("Size"),
            self.sorted.join(","),
        );

        StackPanel::new()
            .children((
                table(Vec::<()>::new(), columns)
                    .widths(&self.widths)
                    .on_resize(cx.on(Changed::Resized))
                    .order(&self.order)
                    .on_reorder(cx.on(Changed::Reordered))
                    .sort(sort, cx.on(Changed::Sorted))
                    .build(),
                TextBlock::new().text(shown),
            ))
            .into()
    }
}

fn shows(page: &Mounted<'_, Columns>, seen: &str) {
    assert!(page.find_text(seen).is_some(), "{:#?}", page.tree());
}

fn drag_edge(page: &mut Mounted<'_, Columns>, column: &'static str, drag: Drag) {
    let tree = page.tree();
    let edge = tree
        .find(Edge(column))
        .and_then(|edge| edge.children.first())
        .unwrap_or_else(|| panic!("no handle for {column}: {tree:#?}"));
    page.at(edge.at).drag_here(drag).settle();
}

#[guinea::test(iterations = 2)]
fn a_header_dragged_past_half_its_neighbour_takes_its_place(h: &mut Harness) {
    let mut page = Mounted::<Columns>::mount(h.segment(), ()).unwrap();

    page.drag(Col::Kind, Drag::by(-40.0, 0.0)).settle();
    shows(&page, "order  size None sorted ");

    page.drag(Col::Kind, Drag::by(-60.0, 0.0)).settle();
    shows(&page, "order Name,Kind,Size,Date size None sorted ");
}

#[guinea::test(iterations = 2)]
fn a_header_keeps_trading_places_while_it_is_dragged(h: &mut Harness) {
    let mut page = Mounted::<Columns>::mount(h.segment(), ()).unwrap();

    page.drag(Col::Date, Drag::by(-160.0, 0.0).steps(8))
        .settle();
    shows(&page, "order Name,Date,Size,Kind size None sorted ");
}

#[guinea::test(iterations = 2)]
fn nothing_passes_the_first_column(h: &mut Harness) {
    let mut page = Mounted::<Columns>::mount(h.segment(), ()).unwrap();

    page.drag(Col::Size, Drag::by(-500.0, 0.0).steps(10))
        .settle();
    shows(&page, "order  size None sorted ");
}

#[guinea::test(iterations = 2)]
fn a_click_sorts_and_a_drag_does_not(h: &mut Harness) {
    let mut page = Mounted::<Columns>::mount(h.segment(), ()).unwrap();

    page.click(Col::Kind).settle();
    shows(&page, "order  size None sorted Kind");

    page.drag(Col::Kind, Drag::by(-60.0, 0.0)).settle();
    shows(&page, "order Name,Kind,Size,Date size None sorted Kind");
}

#[guinea::test(iterations = 2)]
fn a_header_whose_capture_was_lost_still_sorts_on_the_next_click(h: &mut Harness) {
    let mut page = Mounted::<Columns>::mount(h.segment(), ()).unwrap();

    page.drag(Col::Kind, Drag::by(-60.0, 0.0).lost()).settle();
    shows(&page, "order Name,Kind,Size,Date size None sorted ");

    page.click(Col::Kind).settle();
    shows(&page, "order Name,Kind,Size,Date size None sorted Kind");
}

#[guinea::test(iterations = 2)]
fn a_boundary_dragged_resizes_its_column_down_to_its_least(h: &mut Harness) {
    let mut page = Mounted::<Columns>::mount(h.segment(), ()).unwrap();

    drag_edge(&mut page, "Size/resize", Drag::by(40.0, 0.0));
    shows(&page, "order  size Some(140.0) sorted ");

    drag_edge(&mut page, "Size/resize", Drag::by(-400.0, 0.0));
    shows(&page, "order  size Some(24.0) sorted ");
}

#[guinea::test(iterations = 2)]
fn a_boundary_whose_capture_was_lost_keeps_the_width_it_reached(h: &mut Harness) {
    let mut page = Mounted::<Columns>::mount(h.segment(), ()).unwrap();

    drag_edge(&mut page, "Size/resize", Drag::by(40.0, 0.0).lost());
    shows(&page, "order  size Some(140.0) sorted ");
}
