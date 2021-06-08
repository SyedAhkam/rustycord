use std::fmt;

#[derive(Debug, Clone)]
pub struct Token(pub String);

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Token {
    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}
