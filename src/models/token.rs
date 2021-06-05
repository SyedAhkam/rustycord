use std::fmt;

#[derive(Debug, Clone)]
pub struct Token(pub String);

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
