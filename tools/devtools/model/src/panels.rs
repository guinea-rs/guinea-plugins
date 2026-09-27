//! What plugins and backends contribute as trees of their own, and all of
//! them as one tree of rows.
//!
//! Two properties mean something to the rows. A node's `value` is shown
//! beside its label instead of its kind. A top-level node with `languages`
//! (tags, comma-separated) and `language` (the one the application shows)
//! has rows that carry their text under each tag, and the rows show the text
//! in the language picked, or in `language` when none is.

use std::collections::HashSet;

use guinea_devtools_protocol::{Node, Panel, Snapshot};
use serde::{Deserialize, Serialize};

use crate::names::window_name;
use crate::words::{Level, Tone, Word};

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

/// A row of the tree: a panel by its key, and the path to a node in it. An
/// empty path is the panel itself.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Place {
    pub panel: String,
    pub path: Vec<usize>,
}

impl Place {
    /// The first node in any panel whose property `name` reads `value`.
    pub fn holding(listed: &[Listed], name: &str, value: &str) -> Option<Place> {
        listed.iter().find_map(|listed| {
            holding(&listed.panel.nodes, name, value).map(|path| Place {
                panel: listed.key.clone(),
                path,
            })
        })
    }

    /// The node it is, or `None` for a panel or a place that is gone.
    pub fn node<'a>(&self, listed: &'a [Listed]) -> Option<&'a Node> {
        let panel = listed.iter().find(|listed| listed.key == self.panel)?;
        find(&panel.panel.nodes, &self.path)
    }
}

/// One row of the tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub depth: usize,
    pub place: Place,
    /// Whether it has anything under it.
    pub branch: bool,
    /// Whether what is under it follows.
    pub open: bool,
    pub words: Vec<Word>,
}

/// The languages the panels offer, in the order the first to offer them
/// gives them, and the one the application shows.
pub fn languages(listed: &[Listed]) -> (Vec<String>, Option<String>) {
    let mut offered: Vec<String> = Vec::new();
    let mut showing = None;

    for section in listed.iter().flat_map(|listed| &listed.panel.nodes) {
        if let Some(tags) = property(section, "languages") {
            for tag in tags.split(',').map(str::trim).filter(|tag| !tag.is_empty()) {
                if !offered.iter().any(|known| known == tag) {
                    offered.push(tag.to_string());
                }
            }
        }
        if showing.is_none() {
            showing = property(section, "language").map(str::to_string);
        }
    }

    (offered, showing)
}

fn property<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
    node.properties
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

/// Every panel as a row with its nodes under it, top to bottom. Rows in
/// `closed` hide what is under them, except on the way to `reveal`. Rows of a
/// section with languages show their text in `language`.
pub fn lines(
    listed: &[Listed],
    closed: &HashSet<Place>,
    reveal: Option<&Place>,
    language: Option<&str>,
) -> Vec<Line> {
    let mut lines = Vec::new();

    for listed in listed {
        let mut place = Place {
            panel: listed.key.clone(),
            path: Vec::new(),
        };
        let branch = !listed.panel.nodes.is_empty();
        let open = branch && (leads_to(&place, reveal) || !closed.contains(&place));
        lines.push(Line {
            depth: 0,
            place: place.clone(),
            branch,
            open,
            words: vec![Word::new(&listed.title, Tone::Accent)],
        });
        if !open {
            continue;
        }

        for (index, section) in listed.panel.nodes.iter().enumerate() {
            let shown = property(section, "languages")
                .and(language.or_else(|| property(section, "language")));
            place.path.push(index);
            rows(&mut lines, section, &mut place, closed, reveal, shown);
            place.path.pop();
        }
    }

    lines
}

fn leads_to(place: &Place, reveal: Option<&Place>) -> bool {
    reveal.is_some_and(|target| {
        target.panel == place.panel
            && target.path.len() > place.path.len()
            && target.path.starts_with(&place.path)
    })
}

fn rows(
    lines: &mut Vec<Line>,
    node: &Node,
    place: &mut Place,
    closed: &HashSet<Place>,
    reveal: Option<&Place>,
    language: Option<&str>,
) {
    let branch = !node.children.is_empty();
    let open = branch && (leads_to(place, reveal) || !closed.contains(place));
    lines.push(Line {
        depth: place.path.len(),
        place: place.clone(),
        branch,
        open,
        words: words(node, language),
    });
    if !open {
        return;
    }

    for (index, child) in node.children.iter().enumerate() {
        place.path.push(index);
        rows(lines, child, place, closed, reveal, language);
        place.path.pop();
    }
}

/// A node's label, then what it holds: its text in `language`, its value, or
/// else its kind.
fn words(node: &Node, language: Option<&str>) -> Vec<Word> {
    let label = if node.label.is_empty() { &node.kind } else { &node.label };
    let mut words = vec![Word::new(label, Tone::Plain)];

    let translated = language.filter(|_| node.children.is_empty()).map(|tag| property(node, tag));
    let beside = match translated {
        Some(Some(text)) => Some(Word::new(text, Tone::Quote)),
        Some(None) => Some(Word::new("untranslated", Tone::Level(Level::Warn))),
        None => match property(node, "value") {
            Some(value) => Some(Word::new(value, Tone::Quote)),
            None => (!node.label.is_empty() && !node.kind.is_empty())
                .then(|| Word::new(&node.kind, Tone::Muted)),
        },
    };
    if let Some(beside) = beside {
        words.push(Word::new("  ", Tone::Plain));
        words.push(beside);
    }

    words
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

    fn node(label: &str, kind: &str, properties: &[(&str, &str)], children: Vec<Node>) -> Node {
        Node {
            label: label.into(),
            kind: kind.into(),
            properties: properties
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
            children,
        }
    }

    fn application() -> Vec<Listed> {
        let keys = node(
            "Keys",
            "store",
            &[],
            vec![node(
                "app",
                "group",
                &[],
                vec![node("language", "value", &[("value", "\"ru\""), ("path", "app.language")], vec![])],
            )],
        );
        let file = node(
            "main.ftl",
            "file",
            &[("languages", "en, ru"), ("language", "en")],
            vec![
                node("hello", "message", &[("en", "Hello"), ("ru", "Привет")], vec![]),
                node("bye", "untranslated", &[("en", "Bye")], vec![]),
            ],
        );

        [("guinea.store", "Store", keys), ("guinea.l10n", "Localization", file)]
            .into_iter()
            .map(|(key, title, section)| Listed {
                key: key.into(),
                title: title.into(),
                panel: Panel {
                    id: key.into(),
                    title: title.into(),
                    nodes: vec![section],
                },
            })
            .collect()
    }

    fn shown(lines: &[Line]) -> Vec<(usize, String)> {
        lines
            .iter()
            .map(|line| (line.depth, crate::words::text(&line.words)))
            .collect()
    }

    #[test]
    fn every_panel_is_one_tree_and_a_row_shows_what_it_holds() {
        let listed = application();
        let lines = lines(&listed, &HashSet::new(), None, None);

        assert_eq!(
            shown(&lines),
            [
                (0, "Store".to_string()),
                (1, "Keys  store".to_string()),
                (2, "app  group".to_string()),
                (3, "language  \"ru\"".to_string()),
                (0, "Localization".to_string()),
                (1, "main.ftl  file".to_string()),
                (2, "hello  Hello".to_string()),
                (2, "bye  Bye".to_string()),
            ]
        );
        assert_eq!(languages(&listed), (vec!["en".into(), "ru".into()], Some("en".into())));
    }

    #[test]
    fn a_picked_language_shows_its_text_or_that_there_is_none() {
        let listed = application();
        let lines = lines(&listed, &HashSet::new(), None, Some("ru"));
        let texts: Vec<String> = shown(&lines).into_iter().skip(6).map(|(_, text)| text).collect();

        assert_eq!(texts, ["hello  Привет", "bye  untranslated"]);
    }

    #[test]
    fn a_closed_row_hides_what_is_under_it_but_not_the_way_to_a_reveal() {
        let listed = application();
        let store = Place {
            panel: "guinea.store".into(),
            path: Vec::new(),
        };
        let closed = HashSet::from([store]);

        assert_eq!(lines(&listed, &closed, None, None).len(), 5);

        let wanted = Place::holding(&listed, "path", "app.language").expect("found");
        assert_eq!(wanted.path, [0, 0, 0]);
        let lines = lines(&listed, &closed, Some(&wanted), None);
        assert!(lines.iter().any(|line| line.place == wanted));
        assert_eq!(wanted.node(&listed).map(|node| node.label.as_str()), Some("language"));
    }
}
