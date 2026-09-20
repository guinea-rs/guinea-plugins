//! Which editors this machine has, and where their launchers are.

use std::path::PathBuf;
use std::sync::OnceLock;

use super::contracts::Editor;

/// The editors on this machine, `GUINEA_EDITOR` first, the system's last.
pub fn installed() -> &'static [Editor] {
    static FOUND: OnceLock<Vec<Editor>> = OnceLock::new();
    FOUND.get_or_init(|| {
        Editor::ALL
            .into_iter()
            .filter(|editor| match editor {
                Editor::Configured => std::env::var_os("GUINEA_EDITOR").is_some(),
                Editor::System => true,
                other => program(*other).is_some(),
            })
            .collect()
    })
}

/// The editor remembered as `key`, if it is still here; otherwise the first
/// one found.
pub fn remembered(key: &str) -> Editor {
    installed()
        .iter()
        .copied()
        .find(|editor| editor.key() == key)
        .unwrap_or(installed()[0])
}

/// Where `editor`'s launcher is, found once.
pub fn program(editor: Editor) -> Option<PathBuf> {
    static FOUND: OnceLock<Vec<(Editor, Option<PathBuf>)>> = OnceLock::new();
    FOUND
        .get_or_init(|| {
            Editor::ALL
                .into_iter()
                .map(|editor| (editor, find(editor)))
                .collect()
        })
        .iter()
        .find(|(known, _)| *known == editor)
        .and_then(|(_, path)| path.clone())
}

fn find(editor: Editor) -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let local = |tail: &str| local.as_ref().map(|local| local.join(tail));

    match editor {
        Editor::VsCode => on_path(&["code"])
            .or_else(|| existing(local("Programs/Microsoft VS Code/bin/code.cmd"))),
        Editor::Cursor => on_path(&["cursor"])
            .or_else(|| existing(local("Programs/cursor/resources/app/bin/cursor.cmd"))),
        Editor::Zed => on_path(&["zed"]).or_else(|| existing(local("Programs/Zed/bin/zed.exe"))),
        Editor::RustRover => jetbrains("RustRover", "rustrover"),
        Editor::Clion => jetbrains("CLion", "clion"),
        Editor::Idea => jetbrains("IntelliJ IDEA", "idea"),
        Editor::Configured | Editor::System => None,
    }
}

fn existing(path: Option<PathBuf>) -> Option<PathBuf> {
    path.filter(|path| path.is_file())
}

/// The newest install of a JetBrains IDE, then whatever the path has.
fn jetbrains(product: &str, launcher: &str) -> Option<PathBuf> {
    let mut places: Vec<PathBuf> = Vec::new();
    for variable in ["ProgramFiles", "LOCALAPPDATA"] {
        if let Some(root) = std::env::var_os(variable).map(PathBuf::from) {
            places.push(root.join("JetBrains"));
            places.push(root.join("Programs"));
        }
    }

    let newest = places
        .iter()
        .filter_map(|place| std::fs::read_dir(place).ok())
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let version = version(name.strip_prefix(product)?)?;
            let exe = entry.path().join("bin").join(format!("{launcher}64.exe"));
            exe.is_file().then_some((version, exe))
        })
        .max_by(|(one, _), (other, _)| one.cmp(other))
        .map(|(_, exe)| exe);

    newest.or_else(|| on_path(&[launcher, &format!("{launcher}64"), &format!("{launcher}.sh")]))
}

/// ` 2026.2.1` as numbers; `None` for another product that shares the prefix.
fn version(rest: &str) -> Option<Vec<u32>> {
    let rest = rest.strip_prefix(' ').unwrap_or(rest);
    if rest.is_empty() {
        return Some(Vec::new());
    }

    rest.split('.').map(|part| part.parse().ok()).collect()
}

fn on_path(names: &[&str]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        names.iter().find_map(|name| {
            let candidates: Vec<PathBuf> = if cfg!(windows) {
                ["exe", "cmd", "bat"]
                    .iter()
                    .map(|extension| dir.join(format!("{name}.{extension}")))
                    .collect()
            } else {
                vec![dir.join(name)]
            };

            candidates.into_iter().find(|file| file.is_file())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_newest_install_wins_and_other_products_do_not_count() {
        assert_eq!(version(" 2026.2.1"), Some(vec![2026, 2, 1]));
        assert!(version(" 2026.2.1") > version(" 2025.12.3"));
        assert_eq!(version(" Community Edition 2024.1"), None);
    }

    #[test]
    fn the_system_is_always_offered() {
        assert_eq!(installed().last(), Some(&Editor::System));
    }
}
