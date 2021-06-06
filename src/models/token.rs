use std::fmt;

#[derive(Debug, Clone)]
pub struct Token(pub &'static str);

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
