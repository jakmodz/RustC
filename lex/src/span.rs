#[derive(PartialEq, Debug, Clone)]
pub struct Span {
    pub column: usize,
    pub line: usize,
}

impl Span {
    pub fn new(column: usize, line: usize) -> Self {
        Self { column, line }
    }
}
