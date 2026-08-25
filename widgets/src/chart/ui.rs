//! The widget itself: a host element wired to a surface and a pointer.

use std::cell::Cell;
use std::rc::Rc;

use windows_reactor::{
    BackgroundExt, Color, Element, InputExt, PointerEventInfo, RenderCx, composition_host,
};

use super::hover::hover_at;
use super::model::{HoverInfo, LineChartOptions, Series};
use super::surface::ChartHost;

pub fn line_chart(
    cx: &mut RenderCx,
    series: Vec<Series>,
    on_hover: impl Fn(Option<HoverInfo>) + 'static,
) -> Element {
    line_chart_with_options(cx, series, on_hover, LineChartOptions::default())
}

pub fn line_chart_with_options(
    cx: &mut RenderCx,
    series: Vec<Series>,
    on_hover: impl Fn(Option<HoverInfo>) + 'static,
    options: LineChartOptions,
) -> Element {
    let chart = ChartHost::new(cx);
    chart.publish(series, options);

    // Where the pointer was last seen, or `None` when it is away.
    let pointer = cx.use_ref(Cell::new(None::<f32>));
    let on_hover = Rc::new(on_hover);

    // A composition drawing surface (unlike `animated_canvas`) presents a frame
    // only when it is drawn into, instead of every vsync forever - a continuous
    // render loop for a chart that changes a few times a second was burning
    // several percent of a CPU core for nothing. So drawing happens here, gated
    // on the data having actually grown, rather than on every render.
    //
    // The readout is recomputed in the same breath: the series tick on their
    // own, and a pointer resting on the chart should follow them rather than
    // report whatever was under it when it last moved.
    let revision = chart.revision();
    let redraw = chart.clone();
    let pointer_at_tick = pointer.clone();
    let hover_at_tick = on_hover.clone();
    cx.use_effect((revision,), move || {
        redraw.redraw();
        if let Some(x) = pointer_at_tick.borrow().get() {
            hover_at_tick(hover_at(&redraw.series(), x, redraw.width()));
        }
    });

    let on_mount = chart.clone();
    let on_size = chart.clone();
    let on_gone = chart.clone();
    let moved_over = chart.clone();
    let pointer_on_move = pointer.clone();
    let hover_on_move = on_hover.clone();

    composition_host()
        .on_mounted(move |handle| on_mount.attach(handle))
        .on_resize(move |w, h| on_size.resize(w as f32, h as f32))
        .on_unmounted(move |_| on_gone.detach())
        // A composition child visual is invisible to XAML hit-testing, so
        // without a brush on the host itself the chart would never see a
        // pointer. Transparent is enough, and stays out of the way of whatever
        // the chart draws.
        .background(Color::transparent())
        .on_pointer_moved(move |info: PointerEventInfo| {
            let x = info.x as f32;
            pointer_on_move.borrow().set(Some(x));
            hover_on_move(hover_at(&moved_over.series(), x, moved_over.width()));
        })
        .on_pointer_exited(move || {
            pointer.borrow().set(None);
            on_hover(None);
        })
        .into()
}
