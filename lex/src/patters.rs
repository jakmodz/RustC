use crate::token::{Token, TokenType};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref PATTERNS: Vec<(Regex, TokenType)> = {
        vec![
            (Regex::new(r"^continue\b").unwrap(), TokenType::Continue),
            (Regex::new(r"^break\b").unwrap(), TokenType::Break),
            (Regex::new(r"^return\b").unwrap(), TokenType::Return),
            (Regex::new(r"^while\b").unwrap(), TokenType::While),
            (Regex::new(r"^void\b").unwrap(), TokenType::Void),
            (Regex::new(r"^goto\b").unwrap(), TokenType::Goto),
            (Regex::new(r"^else\b").unwrap(), TokenType::Else),
            (Regex::new(r"^int\b").unwrap(), TokenType::Int),

            (Regex::new(r"^for\b").unwrap(), TokenType::For),
            (Regex::new(r"^<<=").unwrap(), TokenType::LeftShiftEqual),
            (Regex::new(r"^>>=").unwrap(), TokenType::RightShiftEqual),
            (Regex::new(r"^<<").unwrap(), TokenType::LeftShift),
            (Regex::new(r"^do\b").unwrap(), TokenType::Do),
            (Regex::new(r"^>>").unwrap(), TokenType::RightShift),
            (Regex::new(r"^\+\+").unwrap(), TokenType::PlusPlus),
            (Regex::new(r"^--").unwrap(), TokenType::HypenHypen),
            (Regex::new(r"^\+=").unwrap(), TokenType::PlusEqual),
            (Regex::new(r"^-=").unwrap(), TokenType::HypenEqual),
            (Regex::new(r"^\*=").unwrap(), TokenType::StarEqual),
            (Regex::new(r"^/=").unwrap(), TokenType::SlashEqual),
            (Regex::new(r"^%=").unwrap(), TokenType::PercentEqual),
            (Regex::new(r"^&=").unwrap(), TokenType::AmpersandEqual),
            (Regex::new(r"^\|=").unwrap(), TokenType::PipeEqual),
            (Regex::new(r"^\^=").unwrap(), TokenType::CaretEqual),
            (Regex::new(r"^&&").unwrap(), TokenType::AmpersandAmpersand),
            (Regex::new(r"^\|\|").unwrap(), TokenType::PipePipe),
            (Regex::new(r"^==").unwrap(), TokenType::EqualEqual),
            (Regex::new(r"^!=").unwrap(), TokenType::ExclamationEqual),
            (Regex::new(r"^<=").unwrap(), TokenType::LessEqual),
            (Regex::new(r"^>=").unwrap(), TokenType::GreaterEqual),
            (Regex::new(r"^if\b").unwrap(), TokenType::If),

            // Identifiers (after keywords)
            (Regex::new(r"^[a-zA-Z_]\w*").unwrap(), TokenType::Identifier),

            // Numbers
            (Regex::new(r"^[0-9]+").unwrap(), TokenType::Constant),

            // Punctuation and single-character operators
            (Regex::new(r"^\(").unwrap(), TokenType::OpenParen),
            (Regex::new(r"^\)").unwrap(), TokenType::CloseParen),
            (Regex::new(r"^\{").unwrap(), TokenType::OpenBrace),
            (Regex::new(r"^\}").unwrap(), TokenType::CloseBrace),
            (Regex::new(r"^;").unwrap(), TokenType::Semicolon),
            (Regex::new(r"^~").unwrap(), TokenType::Tilde),
            (Regex::new(r"^-").unwrap(), TokenType::Hypen),
            (Regex::new(r"^=").unwrap(), TokenType::Equal),
            (Regex::new(r"^\+").unwrap(), TokenType::Plus),
            (Regex::new(r"^\*").unwrap(), TokenType::Star),
            (Regex::new(r"^%").unwrap(), TokenType::Percent),
            (Regex::new(r"^/").unwrap(), TokenType::Slash),
            (Regex::new(r"^\|").unwrap(), TokenType::Pipe),
            (Regex::new(r"^&").unwrap(), TokenType::Ampersand),
            (Regex::new(r"^\^").unwrap(), TokenType::Caret),
            (Regex::new(r"^!").unwrap(), TokenType::Exclamation),
            (Regex::new(r"^<").unwrap(), TokenType::Less),
            (Regex::new(r"^>").unwrap(), TokenType::Greater),
            (Regex::new(r"^\?").unwrap(), TokenType::QuestionMark),
            (Regex::new(r"^:").unwrap(), TokenType::Colon),
        ]
    };

    pub static ref SKIP_PATTERNS: Vec<Regex> = {
        vec![
           Regex::new(r"^//[^\n]*\n?").unwrap(),
           Regex::new(r"^#[^\n]*\n?").unwrap()
        ]
    };

   pub static ref MULIT_LINE_COMMENT: Regex = Regex::new(r"^/\*[\s\S]*?\*/").unwrap();
}
