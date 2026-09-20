//! What devtools see of the localisation: every message under the `.ftl` it
//! is written in, with what it interpolates, what it reads as right now, and
//! who has not translated it.

use guinea_core::devtools::{self, Panel, PanelGuard, PanelNode};

use crate::store::{Key, L10n, Localization};

/// The panel, for as long as the guard is held.
pub(crate) fn watch<S: Localization>() -> PanelGuard {
    devtools::contribute_to_app(|| {
        let keys = S::keys();
        if keys.is_empty() {
            return None;
        }

        Some(Panel {
            id: "guinea.l10n",
            title: "Localization",
            nodes: files::<S>(keys),
        })
    })
}

/// One node per `.ftl`, its messages under it, in the order the file has
/// them.
fn files<S: Localization>(keys: &'static [Key]) -> Vec<PanelNode> {
    let strings = L10n::<S>::current();

    let mut files: Vec<PanelNode> = Vec::new();
    for key in keys {
        let file = match files.iter_mut().find(|node| node.label == key.file) {
            Some(node) => node,
            None => {
                files.push(PanelNode {
                    label: key.file.to_string(),
                    kind: "file".to_string(),
                    ..PanelNode::default()
                });
                files.last_mut().expect("just pushed")
            }
        };

        // Only a message that interpolates nothing can be asked for outright:
        // a resolver handed no arguments answers with an error, not a string.
        let value = key.variables.is_empty().then(|| strings.value(key.id)).flatten();
        let named = key.id.split('.').collect::<Vec<_>>();
        put(&mut file.children, &named, message(key, value));
    }

    for file in &mut files {
        let messages = counted(&file.children);
        let untranslated = untranslated(&file.children);

        file.properties = vec![
            ("messages".to_string(), messages.to_string()),
            ("untranslated".to_string(), untranslated.to_string()),
        ];
    }

    files.sort_by(|one, other| one.label.cmp(&other.label));
    files
}

/// Puts `message` under `named`, a dotted id split into its parts, making the
/// groups above it as it goes. `menu.file.open` nests; `app-title` does not.
fn put(nodes: &mut Vec<PanelNode>, named: &[&str], message: PanelNode) {
    let Some((step, rest)) = named.split_first() else {
        return;
    };

    if rest.is_empty() {
        let mut leaf = message;
        leaf.label = (*step).to_string();
        nodes.push(leaf);
        return;
    }

    let group = match nodes.iter().position(|node| node.label == *step && node.kind == "group") {
        Some(at) => &mut nodes[at],
        None => {
            nodes.push(PanelNode {
                label: (*step).to_string(),
                kind: "group".to_string(),
                ..PanelNode::default()
            });
            nodes.last_mut().expect("just pushed")
        }
    };

    put(&mut group.children, rest, message);
}

/// The messages under `nodes`, groups counted as what they hold.
fn counted(nodes: &[PanelNode]) -> usize {
    nodes
        .iter()
        .map(|node| match node.kind.as_str() {
            "group" => counted(&node.children),
            _ => 1,
        })
        .sum()
}

fn untranslated(nodes: &[PanelNode]) -> usize {
    nodes
        .iter()
        .map(|node| match node.kind.as_str() {
            "group" => untranslated(&node.children),
            "untranslated" => 1,
            _ => 0,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(label: &str, kind: &str) -> PanelNode {
        PanelNode {
            label: label.to_string(),
            kind: kind.to_string(),
            ..PanelNode::default()
        }
    }

    #[test]
    fn a_dotted_id_nests_and_a_plain_one_does_not() {
        let mut nodes = Vec::new();
        put(&mut nodes, &["menu", "file", "open"], leaf("", "message"));
        put(&mut nodes, &["menu", "file", "save"], leaf("", "untranslated"));
        put(&mut nodes, &["app-title"], leaf("", "message"));

        let menu = nodes.iter().find(|node| node.label == "menu").expect("menu");
        let file = &menu.children[0];

        assert_eq!(file.label, "file");
        assert_eq!(
            file.children.iter().map(|node| node.label.as_str()).collect::<Vec<_>>(),
            ["open", "save"]
        );
        assert_eq!(nodes[1].label, "app-title");

        assert_eq!(counted(&nodes), 3);
        assert_eq!(untranslated(&nodes), 1);
    }
}

fn message(key: &Key, value: Option<String>) -> PanelNode {
    let mut properties = vec![
        ("id".to_string(), key.id.to_string()),
        ("source".to_string(), key.text.to_string()),
    ];

    if let Some(value) = value.filter(|value| value != key.text) {
        properties.push(("now".to_string(), value));
    }
    if !key.variables.is_empty() {
        properties.push(("takes".to_string(), key.variables.join(", ")));
    }
    if !key.missing.is_empty() {
        properties.push(("missing".to_string(), key.missing.join(", ")));
    }
    if key.line > 0 {
        properties.push(("at".to_string(), format!("{}:{}", key.file, key.line)));
    }

    PanelNode {
        label: key.id.to_string(),
        kind: if key.missing.is_empty() { "message" } else { "untranslated" }.to_string(),
        properties,
        children: Vec::new(),
    }
}
