use rocket::request::{self, Request, FromRequest};
use rocket::{Outcome, State};
use hyper::header::Basic;
use rocket::http::Status;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct APIKey {
    pub user_id: String,
    pub user_key: Option<String>,
}

impl<'a, 'r> FromRequest<'a, 'r> for APIKey {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<APIKey, ()> {
        let basic_auth = get_basic_auth(request);
        let token_auth = get_token_auth(request);
        match (basic_auth, token_auth) {
            (None, None) => Outcome::Failure((Status::Unauthorized, ())),
            (Some(_), Some(_)) => {
                warn!("User passed both a TOK header and HTTP Basic Auth");
                Outcome::Failure((Status::Unauthorized, ()))
            },
            (Some(auth), None) => Outcome::Success(auth),
            (None, Some(auth)) => Outcome::Success(auth),
        }
    }
}


fn get_basic_auth<'a, 'r>(request: &'a Request<'r>) -> Option<APIKey> {
    let user_auth: String = match request.headers().get_one("Authorization") {
        Some(auth_data) => auth_data.to_string(),
        None => return None,
    };

    let base64_encoded_auth = user_auth.replace("Basic ", "");
    let authdata: Basic = match Basic::from_str(&base64_encoded_auth) {
        Ok(authdata) => authdata,
        Err(_) => return None,
    };
    Some(APIKey {
        user_id: authdata.username,
        user_key: authdata.password,
    })
}

fn get_token_auth<'a, 'r>(request: &'a Request<'r>) -> Option<APIKey> {
    let user_token: String = match request.headers().get_one("TOK") {
        Some(auth_data) => auth_data.to_string(),
        None => return None,
    };

    let user_data: Vec<&str> = user_token.split(':').collect();
    match user_data.len() {
        0 => {
            warn!("User passed empty token");
            None
        },
        1 => {
            Some(APIKey {
                user_id: user_data[0].to_string(),
                user_key: None,
            })
        },
        2 => {
            Some(APIKey {
                user_id: user_data[0].to_string(),
                user_key: Some(user_data[1].to_string()),
            })
        }
        x => {
            warn!("User passed token with too many fields ({})", x);
            None
        },
    }
}
