use rocket::data::{self, Data, FromData};
use rocket::http::Status;

use rocket::outcome::Outcome;
use rocket::request::Request;

use serde::Deserialize;

use serde_json;

pub use serde_json::Value;
pub use serde_json::error::Error as SerdeError;
use std::io::Read;
use std::ops::{Deref, DerefMut};


#[derive(Debug)]
pub struct AlwaysJSON<T = Value>(pub T);

// TODO Remove
//impl<T> AlwaysJSON<T> {
//    #[inline(always)]
//    pub fn into_inner(self) -> T {
//        self.0
//    }
//}

/// Default limit for JSON is 1MB.
const MAX_SIZE: u64 = 1048576;



impl<T: Deserialize> FromData for AlwaysJSON<T> {
    type Error = SerdeError;

    fn from_data(_request: &Request, data: Data) -> data::Outcome<Self, SerdeError> {
        let reader = data.open().take(MAX_SIZE);
        match serde_json::from_reader(reader).map(|val| AlwaysJSON(val)) {
            Ok(value) => Outcome::Success(value),
            Err(e) => {
                warn!("Couldn't parse JSON body: {:?}", e);
                Outcome::Failure((Status::BadRequest, e))
            },
        }
    }
}

impl<T> Deref for AlwaysJSON<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        &self.0
    }
}

impl<T> DerefMut for AlwaysJSON<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.0
    }
}
