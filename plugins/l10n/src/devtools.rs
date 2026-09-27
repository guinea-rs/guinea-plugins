//! What devtools see of the localisation: every message under the `.ftl` it
//! is written in, with what it interpolates, what each language says for it,
//! and who has not translated it.
//!
//! A file says which languages there are and which one the application shows
//! (`languages`, `language`); each message carries its text under every
//! language's tag, so devtools can show any of them beside its name.

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
            nodes: files(keys, S::languages(), &L10n::<S>::current().tag()),
        })
    })
}

/// One node per `.ftl`, its messages under it, in the order the file has
/// them.
fn files(keys: &'static [Key], languages: &[&str], showing: &str) -> Vec<PanelNode> {
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

        let named = key.id.split('.').collect::<Vec<_>>();
        put(&mut file.children, &named, message(key, languages));
    }

    for file in &mut files {
        let messages = counted(&file.children);
        let untranslated = untranslated(&file.children);

        file.properties = vec![
            ("messages".to_string(), messages.to_string()),
            ("untranslated".to_string(), untranslated.to_string()),
        ];
        if !languages.is_empty() {
            file.properties
                .push(("languages".to_string(), languages.join(", ")));
            file.properties
                .push(("language".to_string(), showing.to_string()));
        }
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

    let group = match nodes
        .iter()
        .position(|node| node.label == *step && node.kind == "group")
    {
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

/// The locale `key` was compiled from: the one that neither translated it
/// nor is missing it.
fn reference<'a>(key: &Key, languages: &[&'a str]) -> Option<&'a str> {
    languages.iter().copied().find(|tag| {
        !key.missing.contains(tag) && !key.translations.iter().any(|(done, _)| done == tag)
    })
}

fn message(key: &Key, languages: &[&str]) -> PanelNode {
    let mut properties = vec![("id".to_string(), key.id.to_string())];

    let written = reference(key, languages)
        .map(|tag| (tag, key.text))
        .into_iter()
        .chain(key.translations.iter().copied());
    let mut texts: Vec<(&str, &str)> = written.collect();
    texts.sort_by_key(|(tag, _)| languages.iter().position(|known| known == tag));
    if texts.is_empty() {
        properties.push(("source".to_string(), key.text.to_string()));
    }
    properties.extend(
        texts
            .into_iter()
            .map(|(tag, text)| (tag.to_string(), text.to_string())),
    );

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
        kind: if key.missing.is_empty() {
            "message"
        } else {
            "untranslated"
        }
        .to_string(),
        properties,
        children: Vec::new(),
    }
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
        put(
            &mut nodes,
            &["menu", "file", "save"],
            leaf("", "untranslated"),
        );
        put(&mut nodes, &["app-title"], leaf("", "message"));

        let menu = nodes
            .iter()
            .find(|node| node.label == "menu")
            .expect("menu");
        let file = &menu.children[0];

        assert_eq!(file.label, "file");
        assert_eq!(
            file.children
                .iter()
                .map(|node| node.label.as_str())
                .collect::<Vec<_>>(),
            ["open", "save"]
        );
        assert_eq!(nodes[1].label, "app-title");

        assert_eq!(counted(&nodes), 3);
        assert_eq!(untranslated(&nodes), 1);
    }

    #[test]
    fn a_message_carries_its_text_in_every_language_and_a_file_names_them() {
        static KEYS: &[Key] = &[Key {
            id: "hello",
            file: "main.ftl",
            line: 1,
            text: "Hello",
            variables: &[],
            translations: &[("ru", "Привет")],
            missing: &["de"],
        }];

        let files = files(KEYS, &["de", "en", "ru"], "ru");
        let file = &files[0];
        let hello = &file.children[0];

        assert!(
            file.properties
                .contains(&("languages".into(), "de, en, ru".into()))
        );
        assert!(file.properties.contains(&("language".into(), "ru".into())));
        assert_eq!(
            hello.properties[..3],
            [
                ("id".to_string(), "hello".to_string()),
                ("en".to_string(), "Hello".to_string()),
                ("ru".to_string(), "Привет".to_string()),
            ]
        );
        assert_eq!(hello.kind, "untranslated");
    }
}
