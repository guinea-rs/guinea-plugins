//! What plugins and backends contribute as trees of their own, as rows: the
//! panels with their sections, and the nodes of one section.
//!
//! Two properties mean something to the rows. A node's `value` is shown
//! beside its label instead of its kind. A section with `languages` (tags,
//! comma-separated) and `language` (the one the application shows) has rows
//! that carry their text under each tag, and the rows show the text in the
//! language picked, or in `language` when none is.

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

/// The languages a section's messages can be read in, and the one the
/// application shows; nothing for a section without messages.
pub fn languages(section: &Node) -> (Vec<String>, Option<String>) {
    let offered = property(section, "languages")
        .map(|tags| {
            tags.split(',')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    (offered, property(section, "language").map(str::to_string))
}

fn property<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
    node.properties
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

/// What a row of the side list stands for.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Entry {
    Panel(String),
    /// The sections of a panel whose names start with `prefix`: `pages/`.
    Folder { panel: String, prefix: String },
    Section(Place),
}

/// One row of the side list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Side {
    pub depth: usize,
    pub entry: Entry,
    pub branch: bool,
    pub open: bool,
    pub words: Vec<Word>,
}

/// Every panel as a row with its sections under it: what the side list
/// shows. A section named like a path, `pages/processes.ftl`, sits in a
/// folder of its own. A row in `closed` hides what is under it, except on
/// the way to the section `open` is.
pub fn sections(listed: &[Listed], closed: &HashSet<Entry>, open: Option<&Place>) -> Vec<Side> {
    let mut rows = Vec::new();

    for listed in listed {
        let entry = Entry::Panel(listed.key.clone());
        let branch = !listed.panel.nodes.is_empty();
        let holds_open = open.is_some_and(|open| open.panel == listed.key);
        let shown = branch && (holds_open || !closed.contains(&entry));
        rows.push(Side {
            depth: 0,
            entry,
            branch,
            open: shown,
            words: vec![Word::new(&listed.title, Tone::Accent)],
        });
        if shown {
            folder(&mut rows, listed, "", 1, closed, open);
        }
    }

    rows
}

/// The sections of `listed` under `prefix`, and the folders their names
/// make below it, in the order the sections come.
fn folder(
    rows: &mut Vec<Side>,
    listed: &Listed,
    prefix: &str,
    depth: usize,
    closed: &HashSet<Entry>,
    open: Option<&Place>,
) {
    let mut seen: Vec<&str> = Vec::new();

    for (index, section) in listed.panel.nodes.iter().enumerate() {
        let Some(rest) = section.label.strip_prefix(prefix) else {
            continue;
        };

        let Some((name, _)) = rest.split_once('/') else {
            rows.push(Side {
                depth,
                entry: Entry::Section(Place {
                    panel: listed.key.clone(),
                    path: vec![index],
                }),
                branch: false,
                open: false,
                words: vec![Word::new(rest, Tone::Plain)],
            });
            continue;
        };
        if seen.contains(&name) {
            continue;
        }
        seen.push(name);

        let inner = format!("{prefix}{name}/");
        let holds_open = open.is_some_and(|open| {
            open.panel == listed.key
                && open
                    .path
                    .first()
                    .and_then(|at| listed.panel.nodes.get(*at))
                    .is_some_and(|section| section.label.starts_with(&inner))
        });
        let entry = Entry::Folder {
            panel: listed.key.clone(),
            prefix: inner.clone(),
        };
        let shown = holds_open || !closed.contains(&entry);
        rows.push(Side {
            depth,
            entry,
            branch: true,
            open: shown,
            words: vec![Word::new(name, Tone::Plain)],
        });
        if shown {
            folder(rows, listed, &inner, depth + 1, closed, open);
        }
    }
}

/// The nodes of one section, top to bottom, the section itself left out.
/// Rows in `closed` hide what is under them, except on the way to `reveal`.
/// A section with languages shows its messages' text in `language`, or in
/// the one the application shows.
pub fn lines(
    listed: &Listed,
    section: usize,
    closed: &HashSet<Place>,
    reveal: Option<&Place>,
    language: Option<&str>,
) -> Vec<Line> {
    let mut lines = Vec::new();
    let Some(node) = listed.panel.nodes.get(section) else {
        return lines;
    };

    let shown = property(node, "languages").and(language.or_else(|| property(node, "language")));
    let mut place = Place {
        panel: listed.key.clone(),
        path: vec![section],
    };
    for (index, child) in node.children.iter().enumerate() {
        place.path.push(index);
        rows(&mut lines, child, &mut place, closed, reveal, shown);
        place.path.pop();
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
        depth: place.path.len() - 2,
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

    fn sides(rows: &[Side]) -> Vec<(usize, String)> {
        rows.iter()
            .map(|row| (row.depth, crate::words::text(&row.words)))
            .collect()
    }

    #[test]
    fn the_side_list_is_the_panels_with_their_sections() {
        let listed = application();

        assert_eq!(
            sides(&sections(&listed, &HashSet::new(), None)),
            [
                (0, "Store".to_string()),
                (1, "Keys".to_string()),
                (0, "Localization".to_string()),
                (1, "main.ftl".to_string()),
            ]
        );

        let closed = HashSet::from([Entry::Panel("guinea.store".into())]);
        assert_eq!(sections(&listed, &closed, None).len(), 3);

        let keys = Place {
            panel: "guinea.store".into(),
            path: vec![0],
        };
        assert_eq!(sections(&listed, &closed, Some(&keys)).len(), 4, "the open section stays in view");
    }

    #[test]
    fn a_section_named_like_a_path_sits_in_its_folder() {
        let files = ["layouts/shell.ftl", "pages/processes.ftl", "pages/wsl.ftl", "taskmgr.ftl"];
        let listed = vec![Listed {
            key: "guinea.l10n".into(),
            title: "Localization".into(),
            panel: Panel {
                id: "guinea.l10n".into(),
                title: "Localization".into(),
                nodes: files.iter().map(|file| node(file, "file", &[], vec![])).collect(),
            },
        }];

        assert_eq!(
            sides(&sections(&listed, &HashSet::new(), None)),
            [
                (0, "Localization".to_string()),
                (1, "layouts".to_string()),
                (2, "shell.ftl".to_string()),
                (1, "pages".to_string()),
                (2, "processes.ftl".to_string()),
                (2, "wsl.ftl".to_string()),
                (1, "taskmgr.ftl".to_string()),
            ]
        );

        let pages = Entry::Folder {
            panel: "guinea.l10n".into(),
            prefix: "pages/".into(),
        };
        let closed = HashSet::from([pages]);
        assert_eq!(sections(&listed, &closed, None).len(), 5);

        let wsl = Place {
            panel: "guinea.l10n".into(),
            path: vec![2],
        };
        let rows = sections(&listed, &closed, Some(&wsl));
        assert!(rows.iter().any(|row| row.entry == Entry::Section(wsl.clone())), "the open section stays in view");
    }

    #[test]
    fn a_section_is_its_nodes_and_a_row_shows_what_it_holds() {
        let listed = application();

        assert_eq!(
            shown(&lines(&listed[0], 0, &HashSet::new(), None, None)),
            [(0, "app  group".to_string()), (1, "language  \"ru\"".to_string())]
        );
        assert_eq!(
            shown(&lines(&listed[1], 0, &HashSet::new(), None, None)),
            [(0, "hello  Hello".to_string()), (0, "bye  Bye".to_string())]
        );
        assert_eq!(
            languages(&listed[1].panel.nodes[0]),
            (vec!["en".into(), "ru".into()], Some("en".into()))
        );
        assert_eq!(languages(&listed[0].panel.nodes[0]), (Vec::new(), None));
    }

    #[test]
    fn a_picked_language_shows_its_text_or_that_there_is_none() {
        let listed = application();
        let texts: Vec<String> = shown(&lines(&listed[1], 0, &HashSet::new(), None, Some("ru")))
            .into_iter()
            .map(|(_, text)| text)
            .collect();

        assert_eq!(texts, ["hello  Привет", "bye  untranslated"]);
    }

    #[test]
    fn a_closed_row_hides_what_is_under_it_but_not_the_way_to_a_reveal() {
        let listed = application();
        let app = Place {
            panel: "guinea.store".into(),
            path: vec![0, 0],
        };
        let closed = HashSet::from([app]);

        assert_eq!(lines(&listed[0], 0, &closed, None, None).len(), 1);

        let wanted = Place::holding(&listed, "path", "app.language").expect("found");
        assert_eq!(wanted.path, [0, 0, 0]);
        let lines = lines(&listed[0], 0, &closed, Some(&wanted), None);
        assert!(lines.iter().any(|line| line.place == wanted));
        assert_eq!(wanted.node(&listed).map(|node| node.label.as_str()), Some("language"));
    }
}
