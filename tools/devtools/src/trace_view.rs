//! What the trace page shows: which stream of chains, a filter, the kinds
//! hidden among records, whether they follow new ones, and which record is
//! selected.

use std::collections::BTreeSet;
use std::sync::Arc;

use guinea::feature::{Feature, FeatureInitContext};
use guinea_core::feature::Bound;
use guinea_core::messages;
use guinea_core::scope::Reducer;
use guinea_devtools_model::chains::Stream;
use guinea_devtools_model::trace::Query;
pub use guinea_devtools_model::trace::KINDS;
use guinea_macros::installs;

use crate::memory::Remembered;

messages! {
    Filter(String),
    Toggle(&'static str),
    Freeze(Option<usize>),
    Select(Option<u64>),
    OpenStream(Stream),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TraceView {
    pub stream: Stream,
    pub query: String,
    pub hidden: BTreeSet<&'static str>,
    /// How many records there were when the list was frozen.
    pub frozen: Option<usize>,
    pub selected: Option<u64>,
}

impl Default for TraceView {
    fn default() -> Self {
        Self {
            stream: Stream::All,
            query: String::new(),
            hidden: BTreeSet::from(["tick"]),
            frozen: None,
            selected: None,
        }
    }
}

impl TraceView {
    pub fn shows(&self, kind: &str) -> bool {
        !self.hidden.contains(kind)
    }

    /// The same question the HTTP API is asked.
    pub fn query(&self) -> Query {
        Query {
            hidden: self.hidden.iter().map(|kind| kind.to_string()).collect(),
            text: self.query.clone(),
            upto: self.frozen,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Change {
    Query(String),
    Toggle(&'static str),
    Freeze(Option<usize>),
    Select(Option<u64>),
    Stream(Stream),
}

impl Reducer for TraceView {
    type Update = Change;

    fn reduce(&mut self, change: Change) {
        match change {
            Change::Stream(stream) => self.stream = stream,
            Change::Query(query) => self.query = query,
            Change::Toggle(kind) => {
                if !self.hidden.remove(kind) {
                    self.hidden.insert(kind);
                }
            }
            Change::Freeze(at) => self.frozen = at,
            Change::Select(id) => self.selected = id,
        }
    }
}

pub struct TraceViewFeature {
    _view: Bound<TraceView>,
}

#[installs]
impl Feature for TraceViewFeature {
    type Exports = (TraceView,);

    fn install(cx: &FeatureInitContext, _params: &()) -> anyhow::Result<Self> {
        let remembered: Arc<Remembered> = cx.require()?;
        let hidden = remembered.hidden().get();

        let view = cx
            .state::<TraceView>()
            .seed(TraceView {
                stream: remembered.stream().get().parse().unwrap_or(Stream::All),
                query: remembered.query().get(),
                hidden: KINDS
                    .into_iter()
                    .filter(|kind| hidden.iter().any(|named| named == kind))
                    .collect(),
                ..TraceView::default()
            })
            .plain();

        let port = view.clone();
        let memory = remembered.clone();
        cx.answers(move |OpenStream(stream)| {
            let _ = memory.stream().set(stream.to_string());
            port.push(Change::Stream(stream));
        });

        let port = view.clone();
        let memory = remembered.clone();
        cx.answers(move |Filter(query)| {
            let _ = memory.query().set(query.clone());
            port.push(Change::Query(query));
        });

        let port = view.clone();
        cx.answers(move |Toggle(kind)| {
            let mut hidden = remembered.hidden().get();
            match hidden.iter().position(|named| named == kind) {
                Some(at) => {
                    hidden.remove(at);
                }
                None => hidden.push(kind.to_string()),
            }

            let _ = remembered.hidden().set(hidden);
            port.push(Change::Toggle(kind));
        });

        let port = view.clone();
        cx.answers(move |Freeze(at)| port.push(Change::Freeze(at)));
        let port = view.clone();
        cx.answers(move |Select(id)| port.push(Change::Select(id)));

        Ok(Self { _view: view })
    }
}
