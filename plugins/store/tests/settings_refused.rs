use guinea::app::TestApp;
use guinea_plugin_store::amethystate::store::{Invalid, OpenStruct};
use guinea_plugin_store::amethystate::{Open, Schema, Store, amethystate};
use guinea_plugin_store::{StoreAccess, StorePlugin};

#[amethystate(prefix = "window", open = manual)]
pub struct Window {
    #[amestate(default = 400u32)]
    pub min: u32,

    #[amestate(default = 1600u32)]
    pub max: u32,
}

impl Open for Window {
    fn new_with(store: &Store) -> Result<Self, OpenStruct> {
        let window = <Self as Schema>::open(store)?;
        if window.min().get() > window.max().get() {
            return Err(Invalid::new("the smallest window is wider than the largest").into());
        }
        Ok(window)
    }
}

#[test]
fn a_refused_open_comes_back_to_whoever_tried() {
    let mut app = TestApp::new();
    app.install(StorePlugin::in_memory()).expect("install");

    let window: Window = app.settings().expect("opens with its defaults");
    window.min().set(2000).expect("set");

    let Err(OpenStruct::Declined(said)) = app.settings::<Window>() else {
        panic!("a window wider at its smallest than at its largest opened");
    };
    assert_eq!(said.reason(), "the smallest window is wider than the largest");

    drop(window);
    assert!(app.shutdown().is_empty());
}
