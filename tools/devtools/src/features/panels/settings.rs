use amethystate::amethystate;

#[amethystate(prefix = "panels")]
pub struct PanelsSettings {
    /// The panel open last, by its key.
    #[amestate(default = String::new())]
    pub panel: String,

    /// Which of its sections.
    #[amestate(default = 0usize)]
    pub section: usize,
}
