use std::fmt;
use util::generator;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionID(pub String);

impl SessionID {
    pub fn new() -> Self {
        SessionID(generator::session_id())
    }
}

impl fmt::Display for SessionID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
