mod semantic_error;
mod analyze;

pub use semantic_error::SemanticError;
pub use analyze::SemanticAnalyzer;
#[cfg(test)]
mod tests {
    use super::*;


}
