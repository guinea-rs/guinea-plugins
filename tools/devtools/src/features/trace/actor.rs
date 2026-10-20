use guinea::prelude::In;
use guinea::prelude::Push;
use guinea::{actor, handler};

use super::contracts::{Change, Filter, Freeze, OpenStream, Select, TraceState};
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
        handlers { OpenStream, Filter, Freeze, Select }
    }
}

#[handler]
fn open_stream(In(this, OpenStream(stream), _): In<TraceActor, OpenStream>) {
    this.settings.stream().set(stream.to_string());
    this.push.send(Change::Stream(stream));
}

#[handler]
fn filter(In(this, Filter(query), _): In<TraceActor, Filter>) {
    this.settings.query().set(query.clone());
    this.push.send(Change::Query(query));
}

#[handler]
fn freeze(In(this, Freeze(kept), _): In<TraceActor, Freeze>) {
    this.push.send(Change::Freeze(kept));
}

#[handler]
fn select(In(this, Select(selected), _): In<TraceActor, Select>) {
    this.push.send(Change::Select(selected));
}
