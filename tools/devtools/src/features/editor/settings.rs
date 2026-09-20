use amethystate::amethystate;

#[amethystate(prefix = "editor")]
pub struct EditorSettings {
    /// The editor source links open in, by its key.
    #[amestate(default = String::new())]
    pub editor: String,
}
