use dal;

use errors::*;
use rocket::Route;

use service;
use user::{AuthenticatedUser, UserID};

use webapp::apikey::APIKey;

mod user;
mod session;
mod participant;


pub fn routes() -> Vec<Route> {
    return routes![
        self::user::create_user,
        self::user::check_user,
        self::session::create_session,
        self::session::lookup_session,
        self::session::delete_session,
        self::session::update_session,
        self::participant::join_session,
        self::participant::kick_user,
        self::participant::place_vote,
        self::participant::grant_admin,
        self::participant::revoke_admin,
    ];
}

#[cfg(not(feature = "redis_estimates"))]
type MyStoryDataProvider = dal::SharedMemoryDB;
#[cfg(feature = "redis_estimates")]
type MyStoryDataProvider = dal::RedisDBManager;


pub fn get_authenticated_user<D>(dal: &D, api_key: APIKey) -> Result<AuthenticatedUser>
    where D: dal::StoryData
{
    let err = ErrorKind::UserUnauthorized;
    if let APIKey { user_id, user_key: Some(user_key) } = api_key {
        service::authenticate_user(dal, &UserID(user_id), &user_key.into())
            ?
            .ok_or(err.into())
    } else {
        Err(err.into())
    }
}

#[cfg(test)]
mod test {
    extern crate data_encoding;


    use dal;
    pub use dal::SharedMemoryDB;
    pub use rocket::http::{Header, Method, Status};
    pub use rocket::testing::MockRequest;

    pub use serde_json::{self, Value};
    use service;
    pub use user::BasicUser;
    use webapp;

    pub fn basic_auth(user: &BasicUser) -> Header<'static> {
        let auth_string = format!("{}:{}", user.user_id, user.user_token.0);
        let b64_auth = data_encoding::base64::encode(auth_string.as_bytes());
        Header::new("Authorization".to_string(), format!("Basic {}", b64_auth))
    }

    pub fn register_user<D>(dal: &mut D) -> BasicUser
        where D: dal::StoryData
    {
        service::create_user(&mut *dal).unwrap()
    }

    pub fn fake_user() -> BasicUser {
        BasicUser::new()
    }

}
