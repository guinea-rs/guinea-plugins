use std::panic::{AssertUnwindSafe, catch_unwind};

use guinea::app::TestApp;
use guinea_plugin_store::amethystate::amethystate;
use guinea_plugin_store::amethystate::store::OpenStruct;
use guinea_plugin_store::{StoreAccess, StorePlugin};

#[amethystate(prefix = "window")]
pub struct Window {
    #[amestate(default = 400u32)]
    pub min: u32,
}

#[test]
fn settings_that_will_not_read_come_back_to_whoever_tried() {
    let mut app = TestApp::new();
    app.install(StorePlugin::in_memory()).expect("install");
    app.store()
        .expect("the plugin provides it")
        .set(["window", "min"], &"wide")
        .expect("set");

    let tried = catch_unwind(AssertUnwindSafe(|| app.try_settings::<Window>()));

    assert!(
        matches!(tried, Ok(Err(OpenStruct::WillNotRead { .. }))),
        "{}",
        match &tried {
            Ok(Ok(_)) => "it opened".to_string(),
            Ok(Err(other)) => format!("it said {other}"),
            Err(_) => "it panicked".to_string(),
        }
    );
    assert!(app.shutdown().is_empty());
}
