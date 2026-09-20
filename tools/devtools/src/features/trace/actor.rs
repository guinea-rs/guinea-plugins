use guinea_core::actor::Context;
use guinea_core::feature::Push;
use guinea_macros::{actor, handler};

use super::contracts::{Change, Filter, Freeze, OpenStream, Select, Toggle, TraceState};
use super::settings::TraceSettings;

#[derive(Debug)]
pub struct TraceActor {
    push: Push<TraceState>,
    settings: TraceSettings,
}

impl TraceActor {
    pub fn new(push: Push<TraceState>, settings: TraceSettings) -> Self {
        Self { push, settings }
    }
}

actor! {
    TraceActor {
        handlers { OpenStream, Filter, Toggle, Freeze, Select }
    }
}

#[handler]
fn open_stream(this: &mut TraceActor, ctx: Context<TraceActor, OpenStream>) {
    let stream = ctx.msg.0;
    let _ = this.settings.stream().set(stream.to_string());
    this.push.send(Change::Stream(stream));
}

#[handler]
fn filter(this: &mut TraceActor, ctx: Context<TraceActor, Filter>) {
    let query = ctx.msg.0;
    let _ = this.settings.query().set(query.clone());
    this.push.send(Change::Query(query));
}

#[handler]
fn toggle(this: &mut TraceActor, ctx: Context<TraceActor, Toggle>) {
    let kind = ctx.msg.0;

    let mut hidden = this.settings.hidden().get();
    match hidden.iter().position(|named| named == kind.name()) {
        Some(at) => {
            hidden.remove(at);
        }
        None => hidden.push(kind.name().to_string()),
    }
    let _ = this.settings.hidden().set(hidden);

    this.push.send(Change::Toggle(kind));
}

#[handler]
fn freeze(this: &mut TraceActor, ctx: Context<TraceActor, Freeze>) {
    this.push.send(Change::Freeze(ctx.msg.0));
}

#[handler]
fn select(this: &mut TraceActor, ctx: Context<TraceActor, Select>) {
    this.push.send(Change::Select(ctx.msg.0));
}
