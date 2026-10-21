use guinea::app::{FeatureBuilder, TestApp};
use guinea_plugin_store::amethystate::amethystate;
use guinea_plugin_store::{Store, StoreAccess, StoreConfig, StorePlugin};

#[amethystate(prefix = "general")]
pub struct General {
    #[amestate(default = 8080u16)]
    pub port: u16,
}

#[derive(Clone)]
struct Port(u16);

guinea::feature! {
    ReadSettings {
        provides { Port }
    }
}

#[guinea::installs]
fn read_settings(_app: &mut FeatureBuilder, store: Store) -> anyhow::Result<ReadSettings> {
    Ok(ReadSettings(Port(
        store.try_settings::<General>()?.port().get(),
    )))
}

#[test]
fn settings_open_over_the_selected_store() -> anyhow::Result<()> {
    let mut app = TestApp::new();
    let StorePlugin(first, _) = app.plugin::<StorePlugin>(StoreConfig::in_memory())?;
    let mut other_app = TestApp::new();
    let StorePlugin(second, _) = other_app.plugin::<StorePlugin>(StoreConfig::in_memory())?;
    let written: General = first.settings();
    written.port().set(9000);
    let read: General = first.try_settings()?;
    assert_eq!(read.port().get(), 9000);
    let other: General = second.settings();
    assert_eq!(other.port().get(), 8080);
    let ReadSettings(Port(selected)) = app.install::<ReadSettings>()?;
    assert_eq!(selected, 9000);
    let ReadSettings(Port(other)) = other_app.install::<ReadSettings>()?;
    assert_eq!(other, 8080);
    Ok(())
}
