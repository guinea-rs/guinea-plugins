//! Remembers where an application's windows were, and opens them there again.
//!
//! ```no_run
//! # use guinea_plugin_window_state::WindowStatePlugin;
//! guinea::app::GuineaApp::new()
//!     .plugin(guinea_plugin_store::StorePlugin::for_app("app", "settings"))
//!     .plugin(WindowStatePlugin::new())
//!     # ;
//! ```
//!
//! Two halves, going opposite ways. Writing is a subscription: the shell
//! publishes [`WindowChanged`] whenever a window moves, and this stores it.
//! Reading is a service: the shell asks [`SavedGeometry`] *before* it shows a
//! window, because a size applied afterwards is a visible jump - so here the
//! plugin is asked rather than telling.
//!
//! Windows are keyed by their [label](guinea::app::roots::label), not by
//! `RootId`: ids last one run, and the whole point is to answer for the next
//! one.

mod geometry;

use std::sync::Arc;

use guinea::app::roots;
use guinea::app::windows::{Geometry, RestoreGeometry, SavedGeometry, WindowChanged};
use guinea::app::{Plugin, PluginBuilder};
use guinea_plugin_store::Store;

use geometry::Saved;

/// Where the geometries live in the store. Under one prefix, one entry per
/// window label.
const PREFIX: &str = "window";

pub struct WindowStatePlugin {
    prefix: String,
}

impl Default for WindowStatePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowStatePlugin {
    pub fn new() -> Self {
        Self {
            prefix: PREFIX.to_string(),
        }
    }

    /// Stores under another prefix, for an application that already uses
    /// `window` for something of its own.
    pub fn under(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
}

impl Plugin for WindowStatePlugin {
    const ID: &'static str = "guinea.window-state";
    type Exports = ();

    fn build(self, app: &mut PluginBuilder) -> anyhow::Result<()> {
        let store = app.require::<Store>()?;
        let memory = Arc::new(Memory {
            store,
            prefix: self.prefix,
        });

        // Asked by the shell, before a window is on screen.
        app.provide(SavedGeometry::from_arc(Arc::clone(&memory) as Arc<_>));

        // Told by the shell, every time one moves.
        app.subscribe_global::<WindowChanged>(move |changed| {
            let Some(label) = roots::label(changed.root) else {
                // A window nobody named is a window nobody can ask about
                // later, so there is nothing worth writing down.
                return;
            };
            memory.remember(&label, changed.geometry);
        });

        Ok(())
    }
}

struct Memory {
    store: Arc<Store>,
    prefix: String,
}

impl Memory {
    fn remember(&self, label: &str, geometry: Geometry) {
        let Some(saved) = self.read(label).unwrap_or_default().update(geometry) else {
            return;
        };

        // Straight to the store: `set` buffers, and amethystate's debouncer
        // decides when that reaches the disk. Doing it again here would only
        // add a second opinion about the same question.
        if let Err(e) = self.store.set(self.key(label), &saved) {
            tracing::warn!(error = %e, label, "could not remember the window");
        }
    }

    /// One entry per window, under the plugin's prefix.
    fn key<'a>(&'a self, label: &'a str) -> [&'a str; 2] {
        [self.prefix.as_str(), label]
    }

    fn read(&self, label: &str) -> Option<Saved> {
        match self.store.get::<Saved>(self.key(label)) {
            Ok(saved) => saved,
            Err(e) => {
                tracing::warn!(error = %e, label, "could not read the remembered window");
                None
            }
        }
    }
}

impl RestoreGeometry for Memory {
    fn for_label(&self, label: &str) -> Option<Geometry> {
        self.read(label).map(Saved::geometry)
    }
}
