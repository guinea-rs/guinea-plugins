use guinea::app::TestApp;
use guinea_plugin_store::amethystate::store::builder::Backend;
use guinea_plugin_store::{StoreConfig, StorePlugin};

#[test]
fn a_file_that_will_not_read_leaves_the_application_a_store_in_memory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("settings.json");
    std::fs::write(&path, "this is not json").expect("write");
    let mut app = TestApp::new();

    let StorePlugin(store, persistence) = app
        .plugin::<StorePlugin>(StoreConfig::at(&path).backend(Backend::Json).or_in_memory())
        .expect("install");
    store.kv().set("greeting", &"hello").expect("set");
    assert_eq!(
        store.kv().get::<String>("greeting").expect("get"),
        Some("hello".to_string())
    );
    assert!(
        persistence
            .as_ref()
            .is_some_and(|persistence| persistence.is_in_memory()),
        "the application can tell it is in memory"
    );

    drop(store);
    drop(persistence);
    drop(app);
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        "this is not json",
        "the file is left for whoever can read it"
    );
}

#[test]
fn a_file_that_opens_is_used_and_said_to_be_on_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("settings.json");
    let mut app = TestApp::new();

    let StorePlugin(_, persistence) = app
        .plugin::<StorePlugin>(StoreConfig::at(&path).backend(Backend::Json).or_in_memory())
        .expect("install");

    let in_memory = persistence.map(|persistence| persistence.is_in_memory());
    assert_eq!(in_memory, Some(false));
    drop(app);
}
