use amethystate::store::builder::Backend;
use guinea::app::TestApp;
use guinea_plugin_store::{Store, StorePlugin, amethystate};

#[test]
fn a_store_in_memory_keeps_what_is_written_until_shutdown_and_no_longer() {
    for _ in 0..2 {
        let mut app = TestApp::new();
        app.install(StorePlugin::in_memory().backend(Backend::Json))
            .expect("install");

        let store = app.require::<Store>().expect("store provided");
        assert_eq!(
            store.get::<String>(["settings", "name"]).expect("get"),
            None,
            "nothing is left from before"
        );
        store.set(["settings", "name"], &"guinea").expect("set");
        assert_eq!(
            store.get::<String>(["settings", "name"]).expect("get"),
            Some("guinea".to_string())
        );

        drop(store);
        assert!(app.shutdown().is_empty());
    }
}
