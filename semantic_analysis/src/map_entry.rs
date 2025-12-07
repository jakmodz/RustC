#[derive(Clone)]
pub enum SymbolKind {
    Var,
    Fun
}

#[derive(Clone)]
pub(crate) struct VariableEntry {
    pub name: String,
    pub has_linkage: bool,
    pub kind: SymbolKind,
    pub from_current_block: bool,
}

impl VariableEntry {
    pub fn new(name: String, from_current_block: bool,kind : SymbolKind,has_linkage:bool) -> Self {
        Self {
            name,
            has_linkage,
            kind,
            from_current_block,
        }
    }
}
