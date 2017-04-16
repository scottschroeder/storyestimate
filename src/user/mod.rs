use std::fmt;
use util::generator;

mod authenticateduser;
pub use self::authenticateduser::AuthenticatedUser;

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct UserID(pub String);

impl UserID {
    pub fn new() -> Self {
        UserID(generator::user_id())
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Nickname(String);

impl Nickname {
    pub fn new<S>(nickname: S) -> Self
        where S: Into<String>
    {
        Nickname(nickname.into())
    }
}

#[derive(Serialize, Deserialize)]
#[derive(PartialEq, Eq, Clone)]
pub struct UserToken(pub String);

impl UserToken {
    pub fn new() -> Self {
        UserToken(generator::authtoken())
    }
}

impl From<String> for UserToken {
    fn from(s: String) -> UserToken {
        UserToken(s)
    }
}

#[derive(Serialize, Deserialize)]
#[derive(PartialEq, Eq, Clone)]
pub struct BasicUser {
    pub user_id: UserID,
    pub user_token: UserToken,
}

impl fmt::Display for UserID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for BasicUser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "BasicUser{{ user_id: {:?}, user_token: REDACTED }}",
               self.user_id)
    }
}



impl BasicUser {
    pub fn new() -> Self {
        BasicUser {
            user_id: UserID::new(),
            user_token: UserToken::new(),
        }
    }

    pub fn authenticate(&self, token: &UserToken) -> Option<AuthenticatedUser> {
        if self.user_token == *token {
            Some(AuthenticatedUser { user_id: self.user_id.clone() })
        } else {
            None
        }
    }
}

pub trait User {
    fn user_id(&self) -> &UserID;
}

impl User for BasicUser {
    fn user_id(&self) -> &UserID {
        &self.user_id
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn get_valid_auth_user() {
        let new_user = BasicUser::new();
        let my_token = new_user.user_token.clone();
        let auth_user = new_user.authenticate(&my_token).unwrap();
        assert_eq!(auth_user.user_id, new_user.user_id);
    }
    #[test]
    fn get_invalid_auth_user() {
        let new_user = BasicUser::new();
        let my_token = UserToken::new();
        let auth_user = new_user.authenticate(&my_token);
        assert_eq!(auth_user, None);
    }
}
