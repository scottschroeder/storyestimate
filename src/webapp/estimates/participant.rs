use super::MyStoryDataProvider;

use errors::*;
use estimates::session::SessionID;
use rocket::State;

use rocket_contrib::{JSON, Value};
use service;
use user::{Nickname, UserID};

use webapp::apikey::APIKey;
use webapp::assumejson::AlwaysJSON;


#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NicknameForm {
    nickname: Nickname,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VoteForm {
    vote: u32,
}

#[put("/session/<session_id_string>/user/<user_id_string>", data = "<public_nickname>")]
pub fn join_session(
    session_id_string: String,
    user_id_string: String,
    public_nickname: Option<AlwaysJSON<NicknameForm>>,
    api_key: APIKey,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<JSON<Value>> {

    let ref nickname = public_nickname
        .ok_or(ErrorKind::UserError("Please provide nickname to update session".to_string()))?
        .nickname;
    let session_id = SessionID(session_id_string);
    let user_id = UserID(user_id_string);

    let mut dal = storydata_provider.get();
    let requesting_user = super::get_authenticated_user(&*dal, api_key)?;
    service::join_session(&mut *dal,
                          &session_id,
                          &user_id,
                          &requesting_user,
                          &nickname)?;
    Ok(JSON(json!({})))
}

#[post("/session/<session_id_string>/user/<user_id_string>/vote", data = "<vote_form>")]
pub fn place_vote(
    session_id_string: String,
    user_id_string: String,
    vote_form: Option<AlwaysJSON<VoteForm>>,
    api_key: APIKey,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<JSON<Value>> {

    let ref vote =
        vote_form.ok_or(ErrorKind::UserError("Please provide a vote to cast".to_string()))?
            .vote;
    let session_id = SessionID(session_id_string);
    let user_id = UserID(user_id_string);

    let mut dal = storydata_provider.get();
    let requesting_user = super::get_authenticated_user(&*dal, api_key)?;
    service::place_vote(&mut *dal, &session_id, &user_id, &requesting_user, *vote)?;
    Ok(JSON(json!({})))
}

#[delete("/session/<session_id_string>/user/<user_id_string>")]
pub fn kick_user(
    session_id_string: String,
    user_id_string: String,
    api_key: APIKey,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<JSON<Value>> {

    let session_id = SessionID(session_id_string);
    let user_id = UserID(user_id_string);

    let mut dal = storydata_provider.get();
    let requesting_user = super::get_authenticated_user(&*dal, api_key)?;
    service::kick_user(&mut *dal, &session_id, &user_id, &requesting_user)?;
    Ok(JSON(json!({})))
}

#[post("/session/<session_id_string>/admin/<user_id_string>")]
pub fn grant_admin(
    session_id_string: String,
    user_id_string: String,
    api_key: APIKey,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<JSON<Value>> {

    let session_id = SessionID(session_id_string);
    let user_id = UserID(user_id_string);

    let mut dal = storydata_provider.get();
    let requesting_user = super::get_authenticated_user(&*dal, api_key)?;
    service::grant_admin(&mut *dal, &session_id, &user_id, &requesting_user)?;
    Ok(JSON(json!({})))
}

#[delete("/session/<session_id_string>/admin/<user_id_string>")]
pub fn revoke_admin(
    session_id_string: String,
    user_id_string: String,
    api_key: APIKey,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<JSON<Value>> {

    let session_id = SessionID(session_id_string);
    let user_id = UserID(user_id_string);

    let mut dal = storydata_provider.get();
    let requesting_user = super::get_authenticated_user(&*dal, api_key)?;
    service::revoke_admin(&mut *dal, &session_id, &user_id, &requesting_user)?;
    Ok(JSON(json!({})))
}

#[cfg(test)]
mod test {
    use super::super::test::*;
    use webapp;
}
