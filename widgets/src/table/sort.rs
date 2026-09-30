/// Which column a table is sorted by, and which way.
#[derive(Clone)]
pub struct SortState<SID> {
    pub field_id: Option<SID>,
    pub descending: bool,
}
