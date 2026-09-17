//! Opening a source file at a line, in the editor picked in the status bar.
//!
//! Only editors found on this machine are offered. `GUINEA_EDITOR`, when set,
//! is one more choice: a command with `{file}`, `{line}` and `{column}` filled
//! in, `rustrover --line {line} {file}`. Whatever the system opens the file
//! with is always there.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use guinea_devtools_protocol::Declared;

/// An editor source files can be opened in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Editor {
    Configured,
    RustRover,
    VsCode,
    Cursor,
    Zed,
    Clion,
    Idea,
    System,
}

impl Editor {
    const ALL: [Editor; 8] = [
        Editor::Configured,
        Editor::RustRover,
        Editor::VsCode,
        Editor::Cursor,
        Editor::Zed,
        Editor::Clion,
        Editor::Idea,
        Editor::System,
    ];

    /// How the choice is remembered.
    pub fn key(self) -> &'static str {
        match self {
            Editor::Configured => "configured",
            Editor::RustRover => "rustrover",
            Editor::VsCode => "vscode",
            Editor::Cursor => "cursor",
            Editor::Zed => "zed",
            Editor::Clion => "clion",
            Editor::Idea => "idea",
            Editor::System => "system",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Editor::Configured => "GUINEA_EDITOR",
            Editor::RustRover => "RustRover",
            Editor::VsCode => "VS Code",
            Editor::Cursor => "Cursor",
            Editor::Zed => "Zed",
            Editor::Clion => "CLion",
            Editor::Idea => "IntelliJ IDEA",
            Editor::System => "System default",
        }
    }

    /// The editor remembered as `key`, if it is still here; otherwise the
    /// first one found.
    pub fn remembered(key: &str) -> Editor {
        installed()
            .iter()
            .copied()
            .find(|editor| editor.key() == key)
            .unwrap_or(installed()[0])
    }

    fn command(self, declared: &Declared) -> Option<Command> {
        let place = format!("{}:{}:{}", declared.file, declared.line, declared.column);

        match self {
            Editor::Configured => configured(&std::env::var("GUINEA_EDITOR").ok()?, declared),
            Editor::VsCode | Editor::Cursor => {
                let mut command = launch(&program(self)?);
                command.args(["--goto", &place]);
                Some(command)
            }
            Editor::Zed => {
                let mut command = launch(&program(self)?);
                command.arg(place);
                Some(command)
            }
            Editor::RustRover | Editor::Clion | Editor::Idea => {
                let mut command = launch(&program(self)?);
                command
                    .args(["--line", &declared.line.to_string()])
                    .args(["--column", &declared.column.to_string()])
                    .arg(&declared.file);
                Some(command)
            }
            Editor::System => Some(system(declared)),
        }
    }
}

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

pub fn open(editor: Editor, declared: &Declared) {
    let opened = editor
        .command(declared)
        .is_some_and(|mut command| command.spawn().is_ok());

    if !opened && system(declared).spawn().is_err() {
        tracing::warn!(file = declared.file, editor = editor.title(), "could not open the file");
    }
}

/// Where something was declared, as a link that opens it in `editor`.
pub fn link(ui: &mut egui::Ui, declared: &Declared, editor: Editor) {
    let file = Path::new(&declared.file);
    let name = file
        .file_name()
        .map_or_else(|| declared.file.clone(), |name| name.to_string_lossy().into_owned());
    let shown = format!("{name}:{}", declared.line);

    if !declared.found {
        ui.label(crate::style::dim(shown)).on_hover_text(&declared.file);
        return;
    }

    let link = ui
        .add(egui::Link::new(crate::style::mono(shown).underline()))
        .on_hover_text(format!("{}\nopen in {}", declared.file, editor.title()));
    if link.clicked() {
        open(editor, declared);
    }
}

fn program(editor: Editor) -> Option<PathBuf> {
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

fn launch(program: &Path) -> Command {
    let mut command = Command::new(program);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
}

fn fill(part: &str, declared: &Declared) -> String {
    part.replace("{file}", &declared.file)
        .replace("{line}", &declared.line.to_string())
        .replace("{column}", &declared.column.to_string())
}

/// The command `template` asks for, split on whitespace.
fn configured(template: &str, declared: &Declared) -> Option<Command> {
    let mut parts = template.split_whitespace().map(|part| fill(part, declared));
    let mut command = launch(Path::new(&parts.next()?));
    command.args(parts);
    Some(command)
}

fn system(declared: &Declared) -> Command {
    if cfg!(windows) {
        let mut command = launch(Path::new("cmd"));
        command.args(["/C", "start", "", &declared.file]);
        command
    } else if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(&declared.file);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(&declared.file);
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_template_is_filled_with_the_place() {
        let declared = Declared {
            file: "C:/src/actor.rs".into(),
            line: 12,
            column: 5,
            found: true,
        };

        let command = configured("rustrover --line {line} {file}", &declared).expect("a command");
        let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();

        assert_eq!(command.get_program(), "rustrover");
        assert_eq!(args, ["--line", "12", "C:/src/actor.rs"]);
    }

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
