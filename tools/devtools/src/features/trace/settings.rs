use amethystate::amethystate;

#[amethystate(prefix = "trace")]
pub struct TraceSettings {
    /// The trace filter.
    #[amestate(default = String::new())]
    pub query: String,

    /// The trace stream open last, by its id.
    #[amestate(default = "all".to_string())]
    pub stream: String,
}
