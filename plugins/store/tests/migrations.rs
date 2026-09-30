use amethystate::StoreBuilder;
use amethystate::store::builder::Backend;
use guinea::app::TestApp;
use guinea_plugin_store::{Store, StorePlugin, amethystate};

#[test]
fn steps_handed_to_the_plugin_run_when_the_store_opens() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("store");

    let written = StoreBuilder::new(&path)
        .backend(Backend::Json)
        .build()
        .expect("a store to start from");
    written.set(["settings", "name"], &"guinea").expect("set");
    written.set(["window", "width"], &800u32).expect("set");
    written.close().expect("closed");

    let mut app = TestApp::new();
    app.install(
        StorePlugin::at(&path)
            .backend(Backend::Json)
            .migrations(|migrations| {
                migrations
                    .for_prefix("settings")
                    .step(1, "greet", |context| {
                        let name: String = context.get("name")?.unwrap_or_default();
                        context.set("greeting", &format!("hello, {name}"))
                    });
            })
            .migrations(|migrations| {
                migrations
                    .for_prefix("window")
                    .step(1, "double", |context| {
                        let width: u32 = context.get("width")?.unwrap_or_default();
                        context.set("width", &(width * 2))
                    });
            }),
    )
    .expect("install");

    let store = app.require::<Store>().expect("store provided");
    assert_eq!(
        store.get::<String>(["settings", "greeting"]).expect("get"),
        Some("hello, guinea".to_string()),
        "the first call's steps ran"
    );
    assert_eq!(
        store.get::<u32>(["window", "width"]).expect("get"),
        Some(1600),
        "and the second's"
    );

    assert!(app.shutdown().is_empty());
}
