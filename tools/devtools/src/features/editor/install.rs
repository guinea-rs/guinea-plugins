use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_macros::installs;

use super::actor::EditorActor;
use super::contracts::EditorChoice;
use super::detect;
use super::settings::EditorSettings;

pub struct EditorFeature {
    _choice: Bound<EditorChoice>,
}

#[installs]
impl Feature for EditorFeature {
    type Exports = (EditorChoice,);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let settings = EditorSettings::new()?;
        let seed = EditorChoice(detect::remembered(&settings.editor().get()));

        let (choice, _) = cx
            .state::<EditorChoice>()
            .seed(seed)
            .driven_by(|push| EditorActor::new(push, settings));

        Ok(Self { _choice: choice })
    }
}
