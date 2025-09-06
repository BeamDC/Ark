use std::fmt;
use std::fmt::Formatter;

/// tags to be applied to files
#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub name: String,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self { name: name.to_lowercase() }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
