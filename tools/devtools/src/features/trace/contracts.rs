use std::collections::BTreeSet;

use guinea_core::scope::Reducer;
use guinea_devtools_model::chains::Stream;
use guinea_devtools_model::trace::Query;
use guinea_devtools_model::words::Kind;

#[derive(Debug, Clone)]
pub struct Filter(pub String);

#[derive(Debug, Clone)]
pub struct Toggle(pub Kind);

#[derive(Debug, Clone)]
pub struct Freeze(pub Option<usize>);

#[derive(Debug, Clone)]
pub struct Select(pub Option<u64>);

#[derive(Debug, Clone)]
pub struct OpenStream(pub Stream);

/// What the trace page shows: which stream of chains, a filter, the kinds
/// hidden among records, whether they follow new ones, and which record is
/// selected.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceState {
    pub stream: Stream,
    pub query: String,
    pub hidden: BTreeSet<Kind>,
    /// How many records there were when the list was frozen.
    pub frozen: Option<usize>,
    pub selected: Option<u64>,
}

impl Default for TraceState {
    fn default() -> Self {
        Self {
            stream: Stream::All,
            query: String::new(),
            hidden: BTreeSet::from([Kind::Tick]),
            frozen: None,
            selected: None,
        }
    }
}

impl TraceState {
    pub fn shows(&self, kind: Kind) -> bool {
        !self.hidden.contains(&kind)
    }

    /// The same question the HTTP API is asked.
    pub fn query(&self) -> Query {
        Query {
            hidden: self.hidden.iter().map(|kind| kind.name().to_string()).collect(),
            text: self.query.clone(),
            upto: self.frozen,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Change {
    Query(String),
    Toggle(Kind),
    Freeze(Option<usize>),
    Select(Option<u64>),
    Stream(Stream),
}

impl Reducer for TraceState {
    type Update = Change;

    fn reduce(&mut self, change: Change) {
        match change {
            Change::Stream(stream) => self.stream = stream,
            Change::Query(query) => self.query = query,
            Change::Toggle(kind) => {
                if !self.hidden.remove(&kind) {
                    self.hidden.insert(kind);
                }
            }
            Change::Freeze(at) => self.frozen = at,
            Change::Select(id) => self.selected = id,
        }
    }
}
