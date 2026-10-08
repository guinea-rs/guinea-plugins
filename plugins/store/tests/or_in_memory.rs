use std::sync::Arc;

use guinea::app::TestApp;
use guinea_plugin_store::amethystate::store::builder::Backend;
use guinea_plugin_store::{Persistence, Store, StorePlugin};

#[test]
fn a_file_that_will_not_read_leaves_the_application_a_store_in_memory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("settings.json");
    std::fs::write(&path, "this is not json").expect("write");
    let mut app = TestApp::new();

    let installed = app
        .install(StorePlugin::at(&path).backend(Backend::Json).or_in_memory())
        .map(|_| ())
        .map_err(|error| format!("{error:#}"));

    assert_eq!(installed, Ok(()));
    let store = app.require::<Store>().expect("a store is provided");
    store.kv().set("greeting", &"hello").expect("set");
    assert_eq!(
        store.kv().get::<String>("greeting").expect("get"),
        Some("hello".to_string())
    );
    let persistence: Option<Arc<Persistence>> = app.require::<Persistence>().ok();
    assert!(
        persistence
            .as_deref()
            .is_some_and(Persistence::is_in_memory),
        "the application can tell it is in memory"
    );

    drop(store);
    drop(persistence);
    assert!(app.shutdown().is_empty());
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

    app.install(StorePlugin::at(&path).backend(Backend::Json).or_in_memory())
        .expect("install");

    let in_memory = app
        .require::<Persistence>()
        .ok()
        .map(|persistence| persistence.is_in_memory());
    assert_eq!(in_memory, Some(false));
    assert!(app.shutdown().is_empty());
}
