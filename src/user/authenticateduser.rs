use super::{User, UserID};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserID,
}

impl User for AuthenticatedUser {
    fn user_id(&self) -> &UserID {
        &self.user_id
    }
}

impl fmt::Display for AuthenticatedUser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.user_id)
    }
}
