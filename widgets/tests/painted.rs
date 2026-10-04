#![cfg(all(windows, feature = "winui"))]

//! A painted view under the harness: the pointer over it comes back to the
//! page in DIPs, through a drag that holds it and one that loses it.

use guinea::app::Harness;
use guinea::prelude::*;
use guinea::winui::harness::{Drag, Mounted};
use guinea::winui::{Page, PageCx, UpdateCx, page};
use guinea_widgets::painted::{Metrics, Paint, Painted, Pointer};
use windows_canvas::{DrawingSession, GpuDevice};
use windows_reactor::{Border, StackPanel, TextBlock, View};

#[derive(Default)]
struct Blank;

impl Paint for Blank {
    fn paint(&self, _session: &DrawingSession<'_>, _device: &GpuDevice, _metrics: Metrics) {}
}

#[derive(Default)]
pub struct Canvas {
    painted: Painted<Blank>,
    seen: Vec<String>,
}

#[page]
impl Page for Canvas {
    type Params = ();
    type Installs = ();
    type Message = Pointer;

    fn install(_ctx: &FeatureInitContext, _params: &()) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, pointer: Pointer, _cx: &mut UpdateCx<'_, Self>) {
        self.seen.push(match pointer {
            Pointer::Moved(at) => format!("moved {},{}", at.x, at.y),
            Pointer::Pressed(at) => format!("pressed {},{}", at.x, at.y),
            Pointer::Released(at) => format!("released {},{}", at.x, at.y),
            Pointer::Exited => "exited".into(),
            Pointer::Lost => "lost".into(),
        });
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        StackPanel::new()
            .children((
                Border::new()
                    .automation_id("canvas")
                    .content(self.painted.view(cx.on(|pointer| pointer))),
                TextBlock::new().text(self.seen.join("; ")),
            ))
            .into()
    }
}

struct Named(&'static str);

impl guinea_mark::Mark for Named {
    fn name(&self) -> &'static str {
        self.0
    }
}

fn drag(page: &mut Mounted<'_, Canvas>, drag: Drag) {
    let tree = page.tree();
    let painted = tree
        .find(Named("canvas"))
        .and_then(|canvas| canvas.children.first())
        .unwrap_or_else(|| panic!("no painted view: {tree:#?}"));
    page.at(painted.at).drag_here(drag).settle();
}

fn shows(page: &Mounted<'_, Canvas>, seen: &str) {
    assert!(page.find_text(seen).is_some(), "{:#?}", page.tree());
}

#[guinea::test(iterations = 2)]
fn a_drag_comes_back_in_dips_from_press_to_release(h: &mut Harness) {
    let mut page = Mounted::<Canvas>::mount(h.segment(), ()).unwrap();

    drag(&mut page, Drag::by(20.0, 0.0).from(10.0, 20.0).steps(2));
    shows(
        &page,
        "pressed 10,20; moved 20,20; moved 30,20; lost; released 30,20",
    );
}

#[guinea::test(iterations = 2)]
fn a_drag_whose_capture_was_lost_says_so_and_is_not_released(h: &mut Harness) {
    let mut page = Mounted::<Canvas>::mount(h.segment(), ()).unwrap();

    drag(
        &mut page,
        Drag::by(0.0, 10.0).from(5.0, 5.0).steps(1).lost(),
    );
    shows(&page, "pressed 5,5; moved 5,15; lost");
}
