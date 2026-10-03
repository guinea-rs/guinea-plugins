#![cfg(windows)]

use guinea::app::Harness;
use guinea::feature::Segment;
use guinea::winui::harness::Mounted;
use guinea::winui::{MarkExt, Page, PageCx, page};
use guinea_plugin_l10n::{L10nAccess, L10nPlugin, Language, Localization, SwitchLanguage};
use windows_reactor::{Button, ChildrenControl, ContentControl, StackPanel, TextBlock, View};

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

#[derive(guinea::Mark)]
enum Marks {
    Russian,
}

#[derive(Default)]
pub struct Greeting;

#[page]
impl Page for Greeting {
    type Params = ();

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let switch = cx.language_switch::<Strings>();

        StackPanel::new()
            .children((
                TextBlock::new().text(format!("in {}", cx.l10n::<Strings>().0)),
                Button::new()
                    .mark(Marks::Russian)
                    .on_click(move || switch.to("ru"))
                    .content(TextBlock::new().text("Русский")),
            ))
    }
}

impl Segment for Greeting {
    type Installs = ();
    type Above = ();
}

fn installed() -> Harness {
    let mut h = Harness::new(1);
    h.plugin(L10nPlugin::<Strings>::new("en").persist(false))
        .expect("install");
    h
}

#[test]
fn a_page_reads_the_language_and_is_drawn_again_when_it_switches() {
    let h = installed();

    let mut page = Mounted::<Greeting>::mount(&h.segment(), ()).expect("mounted");
    assert!(page.find_text("in en").is_some(), "{:#?}", page.tree());

    h.act::<Language<Strings>>(SwitchLanguage("ru".into())).settle();
    page.settle();
    assert!(page.find_text("in ru").is_some(), "{:#?}", page.tree());
}

#[test]
fn a_button_switches_the_language_through_the_switch_the_page_took() {
    let h = installed();

    let mut page = Mounted::<Greeting>::mount(&h.segment(), ()).expect("mounted");
    page.click(Marks::Russian).settle();
    page.settle();

    assert!(page.find_text("in ru").is_some(), "{:#?}", page.tree());
}
