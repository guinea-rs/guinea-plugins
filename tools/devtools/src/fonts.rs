//! The system's own fonts in place of egui's, where they can be found.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

/// Candidates for each family, the first one found wins.
const TEXT: &[&str] = &["segoeui.ttf"];
const MONO: &[&str] = &["CascadiaMono.ttf", "consola.ttf"];
/// Glyphs the text fonts lack: arrows, box drawing, dots.
const SYMBOLS: &[&str] = &["seguisym.ttf"];

fn directory() -> Option<PathBuf> {
    std::env::var_os("WINDIR").map(|windows| PathBuf::from(windows).join("Fonts"))
}

fn load(directory: &Path, names: &[&str]) -> Option<(String, Vec<u8>)> {
    names.iter().find_map(|name| {
        std::fs::read(directory.join(name))
            .ok()
            .map(|bytes| (name.to_string(), bytes))
    })
}

/// Puts the system fonts first in their families, keeping egui's behind them
/// for whatever they lack. Changes nothing where none are found.
pub fn install(ctx: &egui::Context) {
    let Some(directory) = directory() else {
        return;
    };

    let mut fonts = FontDefinitions::default();
    let mut changed = false;

    for (family, names) in [
        (FontFamily::Proportional, TEXT),
        (FontFamily::Monospace, MONO),
    ] {
        let Some((name, bytes)) = load(&directory, names) else {
            continue;
        };

        fonts
            .font_data
            .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
        fonts.families.entry(family).or_default().insert(0, name);
        changed = true;
    }

    if let Some((name, bytes)) = load(&directory, SYMBOLS) {
        fonts
            .font_data
            .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));

        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            let list = fonts.families.entry(family).or_default();
            list.insert(list.len().min(1), name.clone());
        }

        changed = true;
    }

    if changed {
        ctx.set_fonts(fonts);
    }
}

/// [`install`], the first time a frame asks.
pub fn once(ctx: &egui::Context) {
    let done = egui::Id::new("system-fonts");
    if ctx.data(|data| data.get_temp::<bool>(done)).is_none() {
        ctx.data_mut(|data| data.insert_temp(done, true));
        install(ctx);
    }
}
