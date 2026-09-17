//! What plugins and backends contribute as trees of their own.

use guinea_devtools_protocol::{Node, Panel, Snapshot};
use serde::{Deserialize, Serialize};

use crate::names::window_name;

/// Whether a panel is a window's view tree, which the elements show instead.
pub fn is_view(panel: &Panel) -> bool {
    panel.nodes.first().is_some_and(|node| node.kind == "segment")
}

/// A panel, with the key it is found by and what it is called in a list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Listed {
    pub key: String,
    pub title: String,
    pub panel: Panel,
}

/// Every panel but the view trees: the application's, then each window's.
pub fn listed(snapshot: &Snapshot) -> Vec<Listed> {
    let mut listed: Vec<Listed> = snapshot
        .panels
        .iter()
        .map(|panel| Listed {
            key: panel.id.clone(),
            title: panel.title.clone(),
            panel: panel.clone(),
        })
        .collect();

    for (index, root) in snapshot.roots.iter().enumerate() {
        let window = window_name(root.label.as_deref(), index);
        for panel in root.panels.iter().filter(|panel| !is_view(panel)) {
            listed.push(Listed {
                key: format!("{}/{}", root.id, panel.id),
                title: format!("{} · {window}", panel.title),
                panel: panel.clone(),
            });
        }
    }

    listed
}

/// The node at `path`, each step an index into the level below.
pub fn find<'a>(nodes: &'a [Node], path: &[usize]) -> Option<&'a Node> {
    let (first, rest) = path.split_first()?;
    let node = nodes.get(*first)?;

    if rest.is_empty() {
        Some(node)
    } else {
        find(&node.children, rest)
    }
}

/// The path to the first node whose property `name` reads `value`.
pub fn holding(nodes: &[Node], name: &str, value: &str) -> Option<Vec<usize>> {
    nodes.iter().enumerate().find_map(|(index, node)| {
        if node.properties.iter().any(|(key, found)| key == name && found == value) {
            return Some(vec![index]);
        }
        holding(&node.children, name, value).map(|mut path| {
            path.insert(0, index);
            path
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_node_is_found_by_a_property() {
        let nodes = vec![Node {
            label: "Keys".into(),
            children: vec![Node {
                label: "language".into(),
                properties: vec![("path".into(), "app.language".into())],
                ..Node::default()
            }],
            ..Node::default()
        }];

        let path = holding(&nodes, "path", "app.language").expect("found");
        assert_eq!(path, [0, 0]);
        assert_eq!(find(&nodes, &path).map(|n| n.label.as_str()), Some("language"));
    }
}
