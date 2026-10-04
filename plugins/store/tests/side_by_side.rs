use amethystate::store::builder::Backend;
use guinea::app::TestApp;
use guinea_plugin_store::{Store, StorePlugin, amethystate};

fn name(app: &TestApp) -> Option<String> {
    let store = app.require::<Store>().expect("store provided");
    store.get::<String>(["settings", "name"]).expect("get")
}

#[test]
fn stores_in_memory_are_open_side_by_side_and_keep_to_themselves() {
    let mut first = TestApp::new();
    first.install(StorePlugin::in_memory()).expect("install");
    let mut second = TestApp::new();
    let installed = second.install(StorePlugin::in_memory()).map(|_| ());
    assert!(installed.is_ok(), "a second store in memory: {installed:?}");

    first
        .require::<Store>()
        .expect("store provided")
        .set(["settings", "name"], &"first")
        .expect("set");

    assert_eq!(name(&first), Some("first".to_string()));
    assert_eq!(
        name(&second),
        None,
        "the second store holds nothing of the first's"
    );

    assert!(first.shutdown().is_empty());
    assert!(second.shutdown().is_empty());
}

#[test]
fn stores_on_disk_are_open_side_by_side() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut first = TestApp::new();
    first
        .install(StorePlugin::at(dir.path().join("first")).backend(Backend::Json))
        .expect("install");
    let mut second = TestApp::new();
    let installed = second
        .install(StorePlugin::at(dir.path().join("second")).backend(Backend::Json))
        .map(|_| ());
    assert!(installed.is_ok(), "a second store on disk: {installed:?}");

    assert!(first.shutdown().is_empty());
    assert!(second.shutdown().is_empty());
}
