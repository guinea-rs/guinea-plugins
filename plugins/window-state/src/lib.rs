//! Remembers where an application's windows were, and opens them there again.
//!
//! ```no_run
//! use guinea_plugin_store::{StoreConfig, StorePlugin};
//! use guinea_plugin_window_state::{WindowStateConfig, WindowStatePlugin};
//! fn app(app: &mut guinea::app::FeatureBuilder) -> anyhow::Result<()> {
//!     let StorePlugin(store, _) = app.plugin::<StorePlugin>(StoreConfig::for_app("app", "settings"))?;
//!     let WindowStatePlugin(saved) = app.plugin::<WindowStatePlugin>(WindowStateConfig::new(store))?;
//!     let _saved = saved;
//!     Ok(())
//! }
//! ```
//!
//! Two halves, going opposite ways. Writing is a subscription: the shell
//! publishes [`WindowChanged`] whenever a window moves, and this stores it.
//! Reading is a service: the shell asks [`SavedGeometry`] *before* it shows a
//! window, because a size applied afterwards is a visible jump - so here the
//! plugin is asked rather than telling.
//!
//! Windows are keyed by their label, not by
//! `RootId`: ids last one run, and the whole point is to answer for the next
//! one.

mod geometry;

use std::sync::Arc;

use guinea::app::PluginBuilder;
use guinea::app::windows::{Geometry, RestoreGeometry, SavedGeometry, WindowChanged};
use guinea_plugin_store::Store;

use geometry::Saved;

/// Where the geometries live in the store. Under one prefix, one entry per
/// window label.
const PREFIX: &str = "window";

pub struct WindowStateConfig {
    store: Store,
    prefix: String,
}

impl WindowStateConfig {
    pub fn new(store: Store) -> Self {
        Self {
            store,
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

guinea::plugin! {
    pub WindowStatePlugin { provides { SavedGeometry } }
}

#[guinea::builds]
fn window_state(
    app: &mut PluginBuilder,
    config: WindowStateConfig,
) -> anyhow::Result<WindowStatePlugin> {
    let memory = Arc::new(Memory {
        store: config.store,
        prefix: config.prefix,
    });

    // Asked by the shell, before a window is on screen.
    let saved = SavedGeometry::from_arc(Arc::clone(&memory) as Arc<_>);

    // Told by the shell, every time one moves.
    app.subscribe(move |changed: WindowChanged| {
        let Some(label) = changed.label else {
            // A window nobody named is a window nobody can ask about
            // later, so there is nothing worth writing down.
            return;
        };
        memory.remember(&label, changed.geometry);
    });

    Ok(WindowStatePlugin(saved))
}

struct Memory {
    store: Store,
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

#[cfg(test)]
mod tests {
    use super::*;
    use guinea::app::Harness;
    use guinea::app::roots::Root;
    use guinea::app::windows::{Position, Size};
    use guinea_plugin_store::{StoreConfig, StorePlugin};

    #[test]
    fn labelled_geometry_is_written_to_the_selected_store_and_returned_to_the_shell() {
        let mut harness = Harness::new(1);
        let StorePlugin(store, _) = harness
            .plugin::<StorePlugin>(StoreConfig::in_memory())
            .unwrap();
        let WindowStatePlugin(saved) = harness
            .plugin::<WindowStatePlugin>(WindowStateConfig::new(store.clone()).under("placement"))
            .unwrap();
        let root = Root::under(&harness.application());
        let geometry = Geometry {
            size: Some(Size {
                width: 800.0,
                height: 600.0,
            }),
            position: Some(Position { x: 10.0, y: 20.0 }),
            ..Geometry::default()
        };
        harness.bus().publish(WindowChanged {
            root: root.id(),
            label: Some("main".into()),
            geometry,
        });
        harness.settled();
        assert_eq!(saved.for_label("main"), Some(geometry));
        assert!(store.get::<Saved>(["placement", "main"]).unwrap().is_some());
        assert!(store.get::<Saved>(["window", "main"]).unwrap().is_none());
        harness.bus().publish(WindowChanged {
            root: root.id(),
            label: None,
            geometry,
        });
        harness.settled();
        assert!(saved.for_label("unnamed").is_none());
        drop(root);
    }
}
