use guinea_devtools_model::chains::Stream;
use guinea_devtools_model::trace::{Class, Query};

#[derive(Debug, Clone)]
pub struct Filter(pub String);

#[derive(Debug, Clone)]
pub struct Freeze(pub Option<usize>);

#[derive(Debug, Clone)]
pub struct Select(pub Option<u64>);

#[derive(Debug, Clone)]
pub struct OpenStream(pub Stream);

/// What the trace page shows: which stream, a filter, whether a list of
/// records follows new ones, and which record is selected.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceState {
    pub stream: Stream,
    pub query: String,
    /// How many records there were when the list was frozen.
    pub frozen: Option<usize>,
    pub selected: Option<u64>,
}

impl Default for TraceState {
    fn default() -> Self {
        Self {
            stream: Stream::All,
            query: String::new(),
            frozen: None,
            selected: None,
        }
    }
}

impl TraceState {
    /// The same question the HTTP API is asked.
    pub fn query(&self) -> Query {
        Query {
            class: self.stream.class().unwrap_or(Class::Own),
            text: self.query.clone(),
            upto: self.frozen,
            ..Query::default()
        }
    }
}

#[derive(Clone, Debug)]
pub enum Change {
    Query(String),
    Freeze(Option<usize>),
    Select(Option<u64>),
    Stream(Stream),
}

#[guinea::reducer]
fn reduce_tracestate(this: &mut TraceState, change: Change) {
    match change {
        Change::Stream(stream) => this.stream = stream,
        Change::Query(query) => this.query = query,
        Change::Freeze(at) => this.frozen = at,
        Change::Select(id) => this.selected = id,
    }
}
