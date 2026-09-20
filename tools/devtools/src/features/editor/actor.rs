use guinea_core::actor::Context;
use guinea_core::feature::Push;
use guinea_macros::{actor, handler};

use super::contracts::{EditorChoice, PickEditor};
use super::settings::EditorSettings;

#[derive(Debug)]
pub struct EditorActor {
    push: Push<EditorChoice>,
    settings: EditorSettings,
}

impl EditorActor {
    pub fn new(push: Push<EditorChoice>, settings: EditorSettings) -> Self {
        Self { push, settings }
    }
}

actor! {
    EditorActor {
        handlers { PickEditor }
    }
}

#[handler]
fn pick_editor(this: &mut EditorActor, ctx: Context<EditorActor, PickEditor>) {
    let picked = ctx.msg.0;
    let _ = this.settings.editor().set(picked.key().to_string());
    this.push.send(picked);
}
