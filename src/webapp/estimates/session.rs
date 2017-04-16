

use super::MyStoryDataProvider;

use errors::*;
use estimates::session::{PublicSession, SessionID};
use estimates::session::SessionState;
use rocket::State;

use rocket_contrib::{JSON, Value};
use service;

use webapp::apikey::APIKey;

use webapp::assumejson::AlwaysJSON;


#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SessionStateForm {
    state: SessionState,
}


#[post("/session")]
pub fn create_session(
    api_key: APIKey,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<JSON<PublicSession>> {
    let mut dal = storydata_provider.get();
    let requesting_user = super::get_authenticated_user(&*dal, api_key)?;
    let session_id: SessionID = service::create_session(&mut *dal, &requesting_user)?;
    service::lookup_session(&mut *dal, &session_id).map(|s| JSON(s.unwrap()))
}

#[get("/session/<session_id_string>")]
pub fn lookup_session(
    session_id_string: String,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<JSON<PublicSession>> {
    let dal = storydata_provider.get();

    let session_id = SessionID(session_id_string);
    service::lookup_session(&*dal, &session_id)
        .and_then(|session_opt| {
            session_opt.ok_or(
                    ErrorKind::ObjectNotFound(
                        format!("Session not found: {}", session_id),
                    )
                            .into()
                )
        })
        .map(|s| JSON(s))
}

#[delete("/session/<session_id_string>")]
pub fn delete_session(
    session_id_string: String,
    api_key: APIKey,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<()> {
    let mut dal = storydata_provider.get();
    let session_id = SessionID(session_id_string);
    let requesting_user = super::get_authenticated_user(&*dal, api_key)?;
    service::delete_session(&mut *dal, &session_id, &requesting_user)
}

#[patch("/session/<session_id_string>", data = "<session_state>")]
pub fn update_session(
    session_id_string: String,
    api_key: APIKey,
    session_state: Option<AlwaysJSON<SessionStateForm>>,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<JSON<Value>> {
    let mut dal = storydata_provider.get();
    let session_id = SessionID(session_id_string);

    let ref state = session_state
        .ok_or(ErrorKind::UserError("Please provide a state for the session".to_string()))?
        .state;

    let requesting_user = super::get_authenticated_user(&*dal, api_key)?;
    service::update_session(&mut *dal, &session_id, &state, &requesting_user)?;
    Ok(JSON(json!({})))
}


#[cfg(test)]
mod test {
    use super::super::test::*;
    use webapp;


    fn check_is_session(v: &Value) {
        if let &Value::Object(ref user_map) = v {
            assert!(user_map.contains_key("session_id"));
            assert!(user_map.contains_key("users"));
            assert!(user_map.contains_key("average"));
            assert!(user_map.contains_key("state"));
        } else {
            panic!("JSON for user data was not an object: {:?}", v);
        }
    }


    #[test]
    fn create_session() {
        let mem_data = SharedMemoryDB::new();
        let alice_header = basic_auth(&register_user(&mut *mem_data.get()));
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Post, "/api/session");
        req.add_header(alice_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let v: Value = serde_json::from_str(&body_str).unwrap();
        check_is_session(&v);
    }

    #[test]
    fn create_session_without_auth() {
        let mem_data = SharedMemoryDB::new();
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Post, "/api/session");
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Unauthorized);
    }
    #[test]
    fn create_session_with_bad_auth() {
        let mem_data = SharedMemoryDB::new();
        let alice_header = basic_auth(&fake_user());
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Post, "/api/session");
        req.add_header(alice_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn lookup_session() {
        let mem_data = SharedMemoryDB::new();
        let alice_header = basic_auth(&register_user(&mut *mem_data.get()));
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Post, "/api/session");
        req.add_header(alice_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let v: Value = serde_json::from_str(&body_str).unwrap();
        check_is_session(&v);
        let session_id = v.get("session_id").map(|sv| sv.as_str().unwrap()).unwrap();

        let mut req = MockRequest::new(Method::Get, format!("/api/session/{}", session_id));
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.body()
            .and_then(|b| b.into_string())
            .expect("could not get body string");
        let v: Value = serde_json::from_str(&body_str).expect("could not get serde value");
        check_is_session(&v);

    }
    #[test]
    fn lookup_missing_session() {
        let mem_data = SharedMemoryDB::new();
        let rocket = webapp::build_webapp(mem_data);
        let session_id = "this_is_not_valid";

        let mut req = MockRequest::new(Method::Get, format!("/api/session/{}", session_id));
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn delete_session() {
        let mem_data = SharedMemoryDB::new();
        let alice_header = basic_auth(&register_user(&mut *mem_data.get()));
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Post, "/api/session");
        req.add_header(alice_header.clone());
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let v: Value = serde_json::from_str(&body_str).unwrap();
        check_is_session(&v);
        let session_id = v.get("session_id").map(|sv| sv.as_str().unwrap()).unwrap();

        let mut req = MockRequest::new(Method::Delete, format!("/api/session/{}", session_id));
        req.add_header(alice_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let mut req = MockRequest::new(Method::Get, format!("/api/session/{}", session_id));
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::NotFound);

    }

    #[test]
    fn delete_session_without_auth() {
        let mem_data = SharedMemoryDB::new();
        let alice_header = basic_auth(&register_user(&mut *mem_data.get()));
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Post, "/api/session");
        req.add_header(alice_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let v: Value = serde_json::from_str(&body_str).unwrap();
        check_is_session(&v);
        let session_id = v.get("session_id").map(|sv| sv.as_str().unwrap()).unwrap();

        let mut req = MockRequest::new(Method::Delete, format!("/api/session/{}", session_id));
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Unauthorized);

        let mut req = MockRequest::new(Method::Get, format!("/api/session/{}", session_id));
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.body()
            .and_then(|b| b.into_string())
            .expect("could not get body string");
        let v: Value = serde_json::from_str(&body_str).expect("could not get serde value");
        check_is_session(&v);
    }

    #[test]
    fn delete_session_with_bad_auth() {
        let mem_data = SharedMemoryDB::new();
        let alice_header = basic_auth(&register_user(&mut *mem_data.get()));
        let bob_header = basic_auth(&fake_user());
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Post, "/api/session");
        req.add_header(alice_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let v: Value = serde_json::from_str(&body_str).unwrap();
        check_is_session(&v);
        let session_id = v.get("session_id").map(|sv| sv.as_str().unwrap()).unwrap();

        let mut req = MockRequest::new(Method::Delete, format!("/api/session/{}", session_id));
        req.add_header(bob_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Unauthorized);

        let mut req = MockRequest::new(Method::Get, format!("/api/session/{}", session_id));
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.body()
            .and_then(|b| b.into_string())
            .expect("could not get body string");
        let v: Value = serde_json::from_str(&body_str).expect("could not get serde value");
        check_is_session(&v);
    }

    #[test]
    fn delete_session_with_non_admin_auth() {
        let mem_data = SharedMemoryDB::new();
        let alice_header = basic_auth(&register_user(&mut *mem_data.get()));
        let bob_header = basic_auth(&register_user(&mut *mem_data.get()));
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Post, "/api/session");
        req.add_header(alice_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let v: Value = serde_json::from_str(&body_str).unwrap();
        check_is_session(&v);
        let session_id = v.get("session_id").map(|sv| sv.as_str().unwrap()).unwrap();

        let mut req = MockRequest::new(Method::Delete, format!("/api/session/{}", session_id));
        req.add_header(bob_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Forbidden);

        let mut req = MockRequest::new(Method::Get, format!("/api/session/{}", session_id));
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.body()
            .and_then(|b| b.into_string())
            .expect("could not get body string");
        let v: Value = serde_json::from_str(&body_str).expect("could not get serde value");
        check_is_session(&v);
    }
}
