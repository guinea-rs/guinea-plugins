//! Opening a source file at a line.
//!
//! `GUINEA_EDITOR`, when set, is a command with `{file}`, `{line}` and
//! `{column}` filled in: `rustrover --line {line} {file}`. Whatever the
//! system opens the file with is the fallback.

use std::path::Path;
use std::process::Command;

use guinea_devtools_protocol::Declared;

use super::contracts::Editor;
use super::detect::program;

pub fn open(editor: Editor, declared: &Declared) {
    let opened = command(editor, declared).is_some_and(|mut command| command.spawn().is_ok());

    if !opened && system(declared).spawn().is_err() {
        tracing::warn!(file = declared.file, editor = editor.title(), "could not open the file");
    }
}

fn command(editor: Editor, declared: &Declared) -> Option<Command> {
    let place = format!("{}:{}:{}", declared.file, declared.line, declared.column);

    match editor {
        Editor::Configured => configured(&std::env::var("GUINEA_EDITOR").ok()?, declared),
        Editor::VsCode | Editor::Cursor => {
            let mut command = launch(&program(editor)?);
            command.args(["--goto", &place]);
            Some(command)
        }
        Editor::Zed => {
            let mut command = launch(&program(editor)?);
            command.arg(place);
            Some(command)
        }
        Editor::RustRover | Editor::Clion | Editor::Idea => {
            let mut command = launch(&program(editor)?);
            command
                .args(["--line", &declared.line.to_string()])
                .args(["--column", &declared.column.to_string()])
                .arg(&declared.file);
            Some(command)
        }
        Editor::System => Some(system(declared)),
    }
}

fn launch(program: &Path) -> Command {
    let command = Command::new(program);

    #[cfg(windows)]
    let command = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut command = command;
        command.creation_flags(CREATE_NO_WINDOW);
        command
    };

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
}
