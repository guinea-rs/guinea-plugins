//! What devtools see of the store: every change as a trace point under what
//! caused it, and a panel with the files, the migrations and the keys.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use amethystate::migration::{ComponentOutcome, NotMigrated};
use amethystate::observability::resolve_field;
use amethystate::store::{StoreLayout, StorePath};
use amethystate::{
    MigrationReport, Store, StoreBackend, StoreEvent, StoreOp, StoreSubscription, SubscriptionKind,
};
use guinea_core::observability::panels::{self, Panel, PanelGuard, PanelNode};
use guinea_core::observability::mark_anywhere;
use guinea_core::trace::{self, Point};

/// How many keys the panel lists; past that it says how many it left out.
const KEYS_SHOWN: usize = 2_000;
/// How much of a value the panel shows.
const VALUE_SHOWN: usize = 240;

/// Devtools' view of one store, for as long as it is held.
pub(crate) struct Watching {
    _subscription: StoreSubscription,
    _panel: PanelGuard,
}

impl Watching {
    pub(crate) fn start(store: &Store, report: &MigrationReport) -> Self {
        let written = Arc::new(AtomicBool::new(true));
        let subscription = store.subscribe(SubscriptionKind::Any, {
            let written = written.clone();
            Arc::new(move |event| {
                written.store(true, Ordering::Relaxed);
                changed(event);
                Ok(())
            })
        });

        let fixed = vec![files(store), migrations(report)];
        let reader = store.clone();
        let listed = RefCell::new(None);
        let panel = panels::contribute_to_app(move || {
            let mut listed = listed.borrow_mut();
            if written.swap(false, Ordering::Relaxed) || listed.is_none() {
                *listed = Some(keys(&reader));
            }

            let mut nodes = fixed.clone();
            nodes.extend(listed.clone());
            Some(Panel {
                id: "guinea.store",
                title: "Store",
                nodes,
            })
        });

        Self {
            _subscription: subscription,
            _panel: panel,
        }
    }
}

fn changed(event: &StoreEvent) {
    if !trace::is_observed_anywhere() && !tracing::enabled!(target: "guinea", tracing::Level::DEBUG)
    {
        return;
    }
    let op = match event.op {
        StoreOp::Set => trace::StoreOp::Set,
        StoreOp::Delete => trace::StoreOp::Delete,
        StoreOp::DeletePrefix => trace::StoreOp::DeletePrefix,
    };
    let path = event.path.clone();
    let outside = event.is_external_edit();
    mark_anywhere(move || Point::Store {
        op,
        field: field(&path),
        path: path.to_string(),
        outside,
    });
}

/// The declared field at `path`: `Settings.theme`.
fn field(path: &StorePath) -> Option<String> {
    resolve_field(path).map(|meta| format!("{}.{}", bare(meta.struct_type_name), meta.field_name))
}

fn bare(type_name: &str) -> &str {
    let generic = type_name.find('<').unwrap_or(type_name.len());
    let start = type_name[..generic].rfind("::").map_or(0, |at| at + 2);
    &type_name[start..]
}

fn node(label: impl Into<String>, kind: &str) -> PanelNode {
    PanelNode {
        label: label.into(),
        kind: kind.to_string(),
        ..PanelNode::default()
    }
}

fn files(store: &Store) -> PanelNode {
    let named = |pairs: &[(&str, &std::path::Path)]| {
        pairs
            .iter()
            .map(|(name, path)| (name.to_string(), path.display().to_string()))
            .collect()
    };
    let properties = match store.files_layout() {
        Some(StoreLayout::Single { data }) => named(&[("data", &data)]),
        Some(StoreLayout::Sidecars {
            data,
            meta,
            data_backup,
            meta_backup,
        }) => named(&[
            ("data", &data),
            ("meta", &meta),
            ("data backup", &data_backup),
            ("meta backup", &meta_backup),
        ]),
        _ => vec![("where".into(), "not on disk".into())],
    };
    PanelNode {
        properties,
        ..node("Files", "layout")
    }
}

fn migrations(report: &MigrationReport) -> PanelNode {
    let mut all = node("Migrations", "report");
    all.properties = vec![(
        "at open".into(),
        if report.has_failures() {
            "failed".into()
        } else if report.has_drift() {
            "drift".into()
        } else {
            "fine".into()
        },
    )];
    all.children = report
        .components
        .iter()
        .map(|component| {
            let prefixes: Vec<String> = component
                .prefixes
                .iter()
                .map(StorePath::to_string)
                .collect();
            let label = if prefixes.is_empty() {
                "(root)".to_string()
            } else {
                prefixes.join(", ")
            };
            let (kind, mut properties) = match &component.outcome {
                ComponentOutcome::Committed { steps } => (
                    "applied",
                    steps
                        .iter()
                        .map(|step| {
                            (
                                format!("{} v{}", step.prefix, step.target_version),
                                step.description.clone().unwrap_or_default(),
                            )
                        })
                        .collect(),
                ),
                ComponentOutcome::Skipped(NotMigrated::UpToDate) => ("up to date", Vec::new()),
                ComponentOutcome::Skipped(NotMigrated::BookkeepingLost { taken_as }) => (
                    "bookkeeping lost",
                    vec![("taken as".into(), format!("v{taken_as}"))],
                ),
                ComponentOutcome::Failed { error } => {
                    ("failed", vec![("error".into(), format!("{error:?}"))])
                }
                _ => ("other", Vec::new()),
            };
            for nagging in &component.nagging {
                let (added, removed) = nagging
                    .diff
                    .as_ref()
                    .map_or((0, 0), |diff| (diff.added.len(), diff.removed.len()));
                properties.push((
                    format!("drift at {}", nagging.prefix),
                    format!(
                        "{added} added, {removed} removed, {} moved",
                        nagging.moved.len()
                    ),
                ));
            }
            PanelNode {
                properties,
                ..node(label, kind)
            }
        })
        .collect();
    all
}

/// The keys, nested by level; each knows its whole path.
fn keys(store: &Store) -> PanelNode {
    let mut tree = node("Keys", "store");
    let listed = match store.scan_prefix(StorePath::root()) {
        Ok(listed) => listed,
        Err(error) => {
            tree.properties = vec![("unreadable".into(), format!("{error:?}"))];
            return tree;
        }
    };
    tree.properties = vec![("count".to_string(), listed.len().to_string())];
    if listed.len() > KEYS_SHOWN {
        tree.properties
            .push(("shown".into(), format!("the first {KEYS_SHOWN}")));
    }
    for (path, bytes) in listed.iter().take(KEYS_SHOWN) {
        let levels: Vec<String> = path
            .segments()
            .map(|level| level.as_str().to_string())
            .collect();
        insert(&mut tree.children, &levels, value(store, path, bytes));
    }
    tree
}

fn value(store: &Store, path: &StorePath, bytes: &[u8]) -> PanelNode {
    let shown = match store.decode::<amethystate::serde_json::Value>(bytes) {
        Ok(value) => value.to_string(),
        Err(_) => format!("{} bytes that will not decode", bytes.len()),
    };
    let mut properties = vec![
        ("value".to_string(), cut(shown)),
        ("bytes".to_string(), bytes.len().to_string()),
        ("path".to_string(), path.to_string()),
    ];
    if let Some(meta) = resolve_field(path) {
        properties.push((
            "field".into(),
            format!("{}.{}", meta.struct_type_name, meta.field_name),
        ));
        properties.push(("type".into(), meta.value_type_name.to_string()));
    }
    PanelNode {
        properties,
        ..node(String::new(), "value")
    }
}

fn cut(mut text: String) -> String {
    if text.chars().count() > VALUE_SHOWN {
        text = text.chars().take(VALUE_SHOWN).collect();
        text.push('…');
    }
    text
}

/// Puts `leaf` at `levels` under `nodes`, making the groups on the way.
fn insert(nodes: &mut Vec<PanelNode>, levels: &[String], leaf: PanelNode) {
    let Some((first, rest)) = levels.split_first() else {
        return;
    };
    let at = match nodes.iter().position(|node| node.label == *first) {
        Some(at) => at,
        None => {
            nodes.push(node(first.clone(), "group"));
            nodes.len() - 1
        }
    };
    let found = &mut nodes[at];
    if rest.is_empty() {
        if found.children.is_empty() {
            found.kind = leaf.kind;
        }
        found.properties = leaf.properties;
    } else {
        found.kind = "group".to_string();
        insert(&mut found.children, rest, leaf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_nest_by_level() {
        let mut nodes = Vec::new();
        let leaf = |value: &str| PanelNode {
            properties: vec![("value".into(), value.into())],
            ..node(String::new(), "value")
        };
        insert(&mut nodes, &["app".into()], leaf("{\"language\":\"ru\"}"));
        insert(
            &mut nodes,
            &["app".into(), "language".into()],
            leaf("\"ru\""),
        );
        insert(
            &mut nodes,
            &["app".into(), "theme".into()],
            leaf("\"dark\""),
        );
        insert(&mut nodes, &["window".into()], leaf("{}"));

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].label, "app");
        assert_eq!(
            nodes[0].kind, "group",
            "a key with keys under it reads as a group"
        );
        assert_eq!(nodes[0].properties[0].1, "{\"language\":\"ru\"}");
        let names: Vec<&str> = nodes[0].children.iter().map(|n| n.label.as_str()).collect();
        assert_eq!(names, ["language", "theme"]);
        assert_eq!(nodes[1].kind, "value");
    }

    #[test]
    fn a_write_is_traced_under_its_cause_and_the_panel_shows_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = amethystate::StoreBuilder::new(dir.path().join("store"))
            .backend(amethystate::store::builder::Backend::Json)
            .build()
            .expect("open");
        let watching = Watching::start(&store, &MigrationReport::default());

        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let sink = seen.clone();
        trace::observe(move |record| {
            if let trace::Trace::Mark(record) = record
                && let Point::Store { op, path, .. } = &record.point
            {
                sink.borrow_mut().push((record.parent, *op, path.clone()));
            }
        });
        let action = trace::mark(|| Point::Action { message: "Save" });
        {
            let _resumed = trace::resume(Some(action));
            store.kv().set("greeting", &"hello").expect("set");
        }
        trace::stop_observing();

        assert_eq!(
            *seen.borrow(),
            [(Some(action), trace::StoreOp::Set, "greeting".to_string())]
        );

        let panels = panels::for_app();
        let panel = panels
            .iter()
            .find(|p| p.id == "guinea.store")
            .expect("offered");
        let sections: Vec<&str> = panel.nodes.iter().map(|n| n.label.as_str()).collect();
        assert_eq!(sections, ["Files", "Migrations", "Keys"]);
        let greeting = &panel.nodes[2].children[0];
        assert_eq!(greeting.label, "greeting");
        assert!(
            greeting
                .properties
                .contains(&("value".to_string(), "\"hello\"".to_string())),
            "{:?}",
            greeting.properties
        );

        drop(watching);
        assert!(panels::for_app().is_empty());
    }

    #[test]
    fn a_long_value_is_cut() {
        let long = "x".repeat(VALUE_SHOWN + 10);
        assert_eq!(cut(long).chars().count(), VALUE_SHOWN + 1);
    }
}
