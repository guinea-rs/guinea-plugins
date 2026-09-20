use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2, pos2, vec2};
use guinea::eframe::{Page, PageCx};
use guinea::feature::FeatureInitContext;
use guinea_devtools_model::graph::{self, ClusterKind, EdgeKind, Graph, NodeKind};
use guinea_devtools_model::words::Kind;
use guinea_devtools_protocol::{BusKind, Channel};

use crate::components;
use crate::features::focus::contracts::Focus;
use crate::features::sessions::contracts::Live;
use crate::theme;

const PAD: f32 = 10.0;
const TITLE: f32 = 20.0;
const GAP: f32 = 14.0;
const COLUMN_GAP: f32 = 90.0;

#[derive(Default)]
pub struct Graphs;

impl Page for Graphs {
    type Params = crate::routes::GraphsParams;
    type Installs = ();

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&mut self, cx: &mut PageCx<'_, Self>) {
        let (live, _) = cx.state::<Live, _>();
        let (focus, _) = cx.state::<Focus, _>();
        let ui = cx.ui();

        let graph = {
            let sessions = live.read();
            let Some(session) = sessions.get(focus.app) else {
                return;
            };
            graph::build(&session.snapshot, &session.trace)
        };

        components::block(ui, legend);
        components::rule(ui);

        if graph.nodes.is_empty() {
            components::block(ui, |ui| ui.label(components::dim("nothing to draw yet")));
            return;
        }

        egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
            components::block(ui, |ui| {
                let layout = Layout::of(ui, &graph);
                let (canvas, _) = ui.allocate_exact_size(layout.size, Sense::hover());
                draw(ui, &graph, &layout, canvas.min.to_vec2());
            })
        });
    }
}

fn legend(ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        for (text, kind) in [
            ("send", EdgeKind::Flow(Channel::Send)),
            ("bg", EdgeKind::Flow(Channel::Bg)),
            ("emit / ask", EdgeKind::Flow(Channel::Emit)),
            ("publish", EdgeKind::Publish),
            ("deliver", EdgeKind::Deliver),
            ("drives", EdgeKind::Drives),
            ("action", EdgeKind::Action),
        ] {
            ui.colored_label(edge_color(kind), format!("— {text}"));
        }

        ui.label(components::dim(format!(
            "· bright edges carried traffic in the last {} s",
            graph::RECENT / 1_000_000
        )));
    });
}

struct Layout {
    clusters: Vec<Rect>,
    nodes: Vec<Rect>,
    size: Vec2,
}

impl Layout {
    fn of(ui: &egui::Ui, graph: &Graph) -> Self {
        let font = FontId::proportional(13.0);
        let node_size: Vec<Vec2> = graph
            .nodes
            .iter()
            .map(|node| {
                ui.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap(node.label.clone(), font.clone(), Color32::WHITE)
                        .size()
                }) + vec2(20.0, 10.0)
            })
            .collect();

        let mut layout = Layout {
            clusters: vec![Rect::NOTHING; graph.clusters.len()],
            nodes: vec![Rect::NOTHING; graph.nodes.len()],
            size: Vec2::ZERO,
        };

        let top: Vec<usize> = (0..graph.clusters.len())
            .filter(|&c| graph.clusters[c].parent.is_none())
            .collect();
        let side: Vec<usize> = top
            .iter()
            .copied()
            .filter(|&c| matches!(graph.clusters[c].kind, ClusterKind::App | ClusterKind::Bus(_)))
            .collect();
        let windows: Vec<usize> = top
            .iter()
            .copied()
            .filter(|&c| graph.clusters[c].kind == ClusterKind::Window)
            .collect();

        let mut x = 0.0;
        let mut y = 0.0;
        let mut widest: f32 = 0.0;

        for cluster in side {
            let size = layout.place(graph, &node_size, cluster, pos2(x, y));
            y += size.y + GAP;
            widest = widest.max(size.x);
        }

        layout.size = vec2(widest, y - GAP);
        x += widest + COLUMN_GAP;

        for window in windows {
            let size = layout.place(graph, &node_size, window, pos2(x, 0.0));
            layout.size = layout.size.max(vec2(x + size.x, size.y));
            x += size.x + COLUMN_GAP;
        }

        layout
    }

    /// Lays `cluster` out with its top-left at `at`, and returns its size.
    fn place(&mut self, graph: &Graph, node_size: &[Vec2], cluster: usize, at: Pos2) -> Vec2 {
        let kind = graph.clusters[cluster].kind;
        let children: Vec<usize> = (0..graph.clusters.len())
            .filter(|&c| graph.clusters[c].parent == Some(cluster))
            .collect();
        let own: Vec<usize> = (0..graph.nodes.len())
            .filter(|&n| graph.nodes[n].cluster == cluster)
            .collect();
        let horizontal = kind == ClusterKind::Segment;

        let mut cursor = at + vec2(PAD, TITLE + PAD * 0.5);
        let mut extent = vec2(0.0, 0.0);
        let advance = |cursor: &mut Pos2, extent: &mut Vec2, size: Vec2, start: Pos2| {
            if horizontal {
                extent.x = cursor.x + size.x - start.x;
                extent.y = extent.y.max(size.y);
                cursor.x += size.x + GAP;
            } else {
                extent.x = extent.x.max(size.x);
                extent.y = cursor.y + size.y - start.y;
                cursor.y += size.y + GAP * 0.5;
            }
        };
        let start = cursor;

        for node in own {
            let size = node_size[node];
            self.nodes[node] = Rect::from_min_size(cursor, size);
            advance(&mut cursor, &mut extent, size, start);
        }

        let mut ordered = children;
        ordered.sort_by_key(|&c| matches!(graph.clusters[c].kind, ClusterKind::Bus(_)));
        for child in ordered {
            let size = self.place(graph, node_size, child, cursor);
            advance(&mut cursor, &mut extent, size, start);
        }

        let title_width = graph.clusters[cluster].label.len() as f32 * 7.5;
        let size = vec2(
            extent.x.max(title_width).max(120.0) + PAD * 2.0,
            extent.y + TITLE + PAD * 1.5,
        );

        self.clusters[cluster] = Rect::from_min_size(at, size);
        size
    }
}

fn cluster_color(kind: ClusterKind) -> Color32 {
    match kind {
        ClusterKind::App => theme::MUTED,
        ClusterKind::Bus(BusKind::Global) => theme::kind_color(Kind::Publish),
        ClusterKind::Bus(BusKind::Window) => theme::kind_color(Kind::Deliver),
        ClusterKind::Window => theme::ACCENT,
        ClusterKind::Segment => theme::kind_color(Kind::Send),
        ClusterKind::Feature => theme::FIELD_BORDER,
    }
}

fn edge_color(kind: EdgeKind) -> Color32 {
    match kind {
        EdgeKind::Flow(Channel::Send) => theme::kind_color(Kind::Send),
        EdgeKind::Flow(Channel::Bg) => theme::MUTED,
        EdgeKind::Flow(Channel::Emit | Channel::Ask) => theme::kind_color(Kind::Spawn),
        EdgeKind::Publish => theme::kind_color(Kind::Publish),
        EdgeKind::Deliver => theme::kind_color(Kind::Deliver),
        EdgeKind::Drives => theme::kind_color(Kind::Push),
        EdgeKind::Action => theme::kind_color(Kind::Action),
    }
}

fn node_look(kind: NodeKind) -> (Color32, f32) {
    match kind {
        NodeKind::Actor => (theme::kind_color(Kind::Handle), 6.0),
        NodeKind::Reducer => (theme::kind_color(Kind::Push), 2.0),
        NodeKind::Listener => (theme::kind_color(Kind::Deliver), 10.0),
        NodeKind::Event => (theme::kind_color(Kind::Publish), 10.0),
        NodeKind::Message => (theme::GONE, 2.0),
        NodeKind::Ui => (theme::kind_color(Kind::Action), 10.0),
    }
}

fn draw(ui: &egui::Ui, graph: &Graph, layout: &Layout, offset: Vec2) {
    let painter = ui.painter();
    let visuals = ui.visuals();

    for (cluster, rect) in graph.clusters.iter().zip(&layout.clusters) {
        let rect = rect.translate(offset);
        let color = cluster_color(cluster.kind);

        painter.rect(
            rect,
            8.0,
            color.gamma_multiply(0.06),
            Stroke::new(1.0, color.gamma_multiply(0.7)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            rect.left_top() + vec2(PAD, 4.0),
            Align2::LEFT_TOP,
            &cluster.label,
            FontId::proportional(12.0),
            color,
        );
    }

    let rects: Vec<Rect> = layout.nodes.iter().map(|r| r.translate(offset)).collect();
    for edge in &graph.edges {
        draw_edge(painter, rects[edge.from], rects[edge.to], edge);
    }

    for (node, rect) in graph.nodes.iter().zip(&rects) {
        let (stroke, rounding) = node_look(node.kind);

        painter.rect(
            *rect,
            rounding,
            visuals.extreme_bg_color,
            Stroke::new(1.2, stroke),
            egui::StrokeKind::Inside,
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            &node.label,
            FontId::proportional(13.0),
            visuals.strong_text_color(),
        );
    }
}

fn draw_edge(painter: &egui::Painter, from: Rect, to: Rect, edge: &graph::Edge) {
    let base = edge_color(edge.kind);
    let (color, width) = if edge.recent > 0 {
        (base, 2.5)
    } else {
        (base.gamma_multiply(0.35), 1.0)
    };
    let stroke = Stroke::new(width, color);

    let font = FontId::proportional(11.0);
    let label = if edge.recent > 0 {
        format!("{} ×{}", edge.label, edge.recent)
    } else {
        edge.label.clone()
    };

    if from == to {
        let top = from.right_top();
        let points = [
            top + vec2(-14.0, 0.0),
            top + vec2(-10.0, -24.0),
            top + vec2(18.0, -20.0),
            top + vec2(0.0, 6.0),
        ];

        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
            points,
            false,
            Color32::TRANSPARENT,
            stroke,
        ));
        arrow_head(painter, points[2], points[3], color);
        painter.text(top + vec2(20.0, -18.0), Align2::LEFT_BOTTOM, label, font, color);
        return;
    }

    let forward = to.center().x >= from.center().x;
    let (start, end) = if (to.center().x - from.center().x).abs() < 40.0 {
        (from.right_center(), to.right_center())
    } else if forward {
        (from.right_center(), to.left_center())
    } else {
        (from.left_center(), to.right_center())
    };

    let bend = ((end.x - start.x).abs() * 0.5).max(40.0);
    let (out, into) = if (to.center().x - from.center().x).abs() < 40.0 {
        (vec2(bend, 0.0), vec2(bend, 0.0))
    } else if forward {
        (vec2(bend, 0.0), vec2(-bend, 0.0))
    } else {
        (vec2(-bend, 0.0), vec2(bend, 0.0))
    };
    let points = [start, start + out, end + into, end];

    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        points,
        false,
        Color32::TRANSPARENT,
        stroke,
    ));
    arrow_head(painter, points[2], end, color);

    if edge.recent > 0 || edge.kind != EdgeKind::Deliver {
        let middle = egui::epaint::CubicBezierShape::from_points_stroke(
            points,
            false,
            Color32::TRANSPARENT,
            stroke,
        )
        .sample(0.5);
        painter.text(middle + vec2(0.0, -3.0), Align2::CENTER_BOTTOM, label, font, color);
    }
}

fn arrow_head(painter: &egui::Painter, from: Pos2, to: Pos2, color: Color32) {
    let direction = (to - from).normalized();
    if !direction.is_finite() {
        return;
    }

    let side = direction.rot90() * 4.0;
    let back = to - direction * 9.0;

    painter.add(egui::Shape::convex_polygon(
        vec![to, back + side, back - side],
        color,
        Stroke::NONE,
    ));
}
