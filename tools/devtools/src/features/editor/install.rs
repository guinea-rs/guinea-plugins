use guinea::feature::FeatureInitContext;
use guinea_macros::{feature, installs};
use guinea_plugin_store::StoreAccess;

use super::actor::EditorActor;
use super::contracts::EditorChoice;
use super::detect;
use super::settings::EditorSettings;

feature! {
    pub EditorFeature {
        exports { EditorChoice }
    }
}

#[installs]
fn editor(cx: &FeatureInitContext) -> anyhow::Result<EditorFeature> {
    let settings = cx.try_settings::<EditorSettings>()?;
    let seed = EditorChoice(detect::remembered(&settings.editor().get()));

    let (choice, _) = cx
        .state::<EditorChoice>()
        .seed(seed)
        .driven_by(|push| EditorActor::new(push, settings));

    Ok(EditorFeature(choice))
}
