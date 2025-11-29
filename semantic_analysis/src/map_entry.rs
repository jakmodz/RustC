#[derive(Clone)]
pub(crate) struct VariableEntry {
    pub name: String,
    pub from_current_block: bool,
}

impl VariableEntry {
    pub fn new(name: String, from_current_block: bool) -> Self {
        Self {
            name,
            from_current_block,
        }
    }
}