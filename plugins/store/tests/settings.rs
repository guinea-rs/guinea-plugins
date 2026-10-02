use std::panic::{AssertUnwindSafe, catch_unwind};

use guinea::app::Harness;
use guinea_plugin_store::amethystate::amethystate;
use guinea_plugin_store::{StoreAccess, StorePlugin};

#[amethystate(prefix = "general")]
pub struct General {
    #[amestate(default = 8080u16)]
    pub port: u16,
}

#[test]
fn settings_open_over_the_store_the_plugin_installed() {
    let mut h = Harness::new(1);
    h.plugin(StorePlugin::in_memory()).expect("install");

    let segment = h.segment();
    let written: General = segment.settings();
    written.port().set(9000).expect("set");

    let read: General = segment.context().settings();
    assert_eq!(read.port().get(), 9000, "a feature's context reads what a segment wrote");

    let store = segment.store().expect("the plugin provides it");
    let opened = General::new_with(&store).expect("opens");
    assert_eq!(opened.port().get(), 9000);
}

#[test]
fn settings_without_the_plugin_say_what_is_missing() {
    let h = Harness::new(1);
    let segment = h.segment();

    let Err(panicked) = catch_unwind(AssertUnwindSafe(|| segment.settings::<General>())) else {
        panic!("settings opened with no store installed");
    };
    let said = panicked
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| panicked.downcast_ref::<&str>().map(|said| said.to_string()))
        .unwrap_or_default();
    assert!(said.contains("StorePlugin"), "{said}");
    assert!(said.contains("General"), "{said}");
}
