#![cfg(windows)]

use guinea::app::Harness;
use guinea::feature::Segment;
use guinea::winui::harness::Mounted;
use guinea::winui::{Page, PageCx, page};
use guinea_plugin_l10n::{L10nAccess, L10nPlugin, Language, Localization, SwitchLanguage};
use windows_reactor::{TextBlock, View};

#[derive(Clone, Default, Debug, PartialEq)]
struct Strings(String);

impl Localization for Strings {
    fn for_tag(tag: &str) -> Option<Self> {
        Some(Self(tag.to_string()))
    }

    fn tag(&self) -> String {
        self.0.clone()
    }

    fn languages() -> &'static [&'static str] {
        &["en", "ru"]
    }
}

#[derive(Default)]
pub struct Greeting;

#[page]
impl Page for Greeting {
    type Params = ();

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        TextBlock::new().text(format!("in {}", cx.l10n::<Strings>().0)).into()
    }
}

impl Segment for Greeting {
    type Installs = ();
    type Above = ();
}

#[test]
fn a_page_reads_the_language_and_is_drawn_again_when_it_switches() {
    let mut h = Harness::new(1);
    h.plugin(L10nPlugin::<Strings>::new("en").persist(false))
        .expect("install");

    let mut page = Mounted::<Greeting>::mount(&h.segment(), ()).expect("mounted");
    assert!(page.find_text("in en").is_some(), "{:#?}", page.tree());

    h.act::<Language<Strings>>(SwitchLanguage("ru".into())).settle();
    page.settle();
    assert!(page.find_text("in ru").is_some(), "{:#?}", page.tree());
}
