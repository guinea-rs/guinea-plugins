use amethystate::amethystate;

#[amethystate(prefix = "tab")]
pub struct TabSettings {
    /// The tab open last, by its title.
    #[amestate(default = "Elements".to_string())]
    pub tab: String,
}
