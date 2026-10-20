use guinea::app::TestApp;
use guinea_plugin_store::amethystate::store::builder::Backend;
use guinea_plugin_store::{StoreConfig, StorePlugin};

#[test]
fn stores_in_memory_are_open_side_by_side_and_keep_to_themselves() {
    let mut first = TestApp::new();
    let StorePlugin(first_store, _) = first
        .plugin::<StorePlugin>(StoreConfig::in_memory())
        .unwrap();
    let mut second = TestApp::new();
    let StorePlugin(second_store, _) = second
        .plugin::<StorePlugin>(StoreConfig::in_memory())
        .unwrap();
    first_store.set(["settings", "name"], &"first").unwrap();
    assert_eq!(
        first_store.get::<String>(["settings", "name"]).unwrap(),
        Some("first".into())
    );
    assert_eq!(
        second_store.get::<String>(["settings", "name"]).unwrap(),
        None
    );
}

#[test]
fn stores_on_disk_are_open_side_by_side() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = TestApp::new();
    let StorePlugin(first, _) = app
        .plugin::<StorePlugin>(StoreConfig::at(dir.path().join("first")).backend(Backend::Json))
        .unwrap();
    let StorePlugin(second, _) = app
        .plugin::<StorePlugin>(StoreConfig::at(dir.path().join("second")).backend(Backend::Json))
        .unwrap();
    first.set(["settings", "name"], &"first").unwrap();
    assert_eq!(second.get::<String>(["settings", "name"]).unwrap(), None);
}
