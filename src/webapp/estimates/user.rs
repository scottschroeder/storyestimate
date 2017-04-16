use super::MyStoryDataProvider;



use errors::*;
use rocket::State;


use rocket_contrib::{JSON, Value};
use service;
use user::BasicUser;
use webapp::apikey::APIKey;

#[post("/user")]
//fn create_user(pool: State<RedisPool>) -> Result<CORS<JSON<Value>>> {
pub fn create_user(storydata_provider: State<MyStoryDataProvider>) -> Result<JSON<BasicUser>> {
    let mut dal = storydata_provider.get();
    service::create_user(&mut *dal).map(|u| JSON(u))
}

#[get("/user")]
pub fn check_user(
    api_key: APIKey,
    storydata_provider: State<MyStoryDataProvider>
) -> Result<JSON<Value>> {
    let dal = storydata_provider.get();
    let _ = super::get_authenticated_user(&*dal, api_key)?;
    Ok(JSON(json!({})))
}

#[cfg(test)]
mod test {
    use super::super::test::*;
    use webapp;

    #[test]
    fn create_user_has_fields() {
        let mem_data = SharedMemoryDB::new();
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Post, "/api/user");
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let v: Value = serde_json::from_str(&body_str).unwrap();
        if let Value::Object(user_map) = v {
            assert!(user_map.contains_key("user_id"));
            assert!(user_map.contains_key("user_token"));
        } else {
            panic!("JSON for user data was not an object: {:?}", v);
        }
    }

    #[test]
    fn check_user() {
        let mem_data = SharedMemoryDB::new();
        let alice_header = basic_auth(&register_user(&mut *mem_data.get()));
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Get, "/api/user");
        req.add_header(alice_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn check_user_without_auth() {
        let mem_data = SharedMemoryDB::new();
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Get, "/api/user");
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn check_user_with_bad_auth() {
        let mem_data = SharedMemoryDB::new();
        let alice_header = basic_auth(&fake_user());
        let rocket = webapp::build_webapp(mem_data);
        let mut req = MockRequest::new(Method::Get, "/api/user");
        req.add_header(alice_header);
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Unauthorized);
    }

}
