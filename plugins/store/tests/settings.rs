use guinea::app::TestApp;
use guinea_plugin_store::amethystate::amethystate;
use guinea_plugin_store::{StoreAccess, StoreConfig, StorePlugin};

#[amethystate(prefix = "general")]
pub struct General {
    #[amestate(default = 8080u16)]
    pub port: u16,
}

#[test]
fn settings_open_over_the_selected_store() -> anyhow::Result<()> {
    let mut app = TestApp::new();
    let StorePlugin(first, _) = app.plugin::<StorePlugin>(StoreConfig::in_memory())?;
    let StorePlugin(second, _) = app.plugin::<StorePlugin>(StoreConfig::in_memory())?;
    assert!(app.service::<guinea_plugin_store::Store>().is_none());
    let written: General = first.settings();
    written.port().set(9000);
    let read: General = first.try_settings()?;
    assert_eq!(read.port().get(), 9000);
    let other: General = second.settings();
    assert_eq!(other.port().get(), 8080);
    app.provide(first);
    assert_eq!(
        app.service::<guinea_plugin_store::Store>()
            .unwrap()
            .settings::<General>()
            .port()
            .get(),
        9000
    );
    Ok(())
}
