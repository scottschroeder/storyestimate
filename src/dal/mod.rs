use errors::*;
use estimates::participant::Participant;
use estimates::session::{Session, SessionID};
use user::{BasicUser, UserID};

mod memory;
mod redis;
mod sharedmemory;
pub use self::memory::MemoryDB;
pub use self::redis::RedisDB;
pub use self::sharedmemory::{RedisDBManager, SharedMemoryDB};

pub trait StoryData {
    fn get_user(&self, user_id: &UserID) -> Result<Option<BasicUser>>;
    fn add_user(&mut self, user: BasicUser) -> Result<()>;

    fn get_session(&self, session_id: &SessionID) -> Result<Option<Session>>;
    fn add_session(&mut self, session: Session) -> Result<()>;
    fn del_session(&mut self, session_id: &SessionID) -> Result<()>;
    fn update_session<F>(&mut self, session_id: &SessionID, plan: F) -> Result<()>
        where F: FnMut(&mut Session, &mut Vec<Participant>) -> Result<()>;

    fn get_participants(&self, session_id: &SessionID) -> Result<Vec<Participant>>;
    fn add_participant(&mut self, participant: Participant) -> Result<()>;
    fn del_participant(&mut self, user_id: &UserID, session_id: &SessionID) -> Result<()>;
    fn update_participant<F>(
        &mut self,
        session_id: &SessionID,
        user_id: &UserID,
        plan: F
    ) -> Result<()>
        where F: FnMut(&mut Participant) -> Result<()>;
    fn get_admins(&self, session_id: &SessionID) -> Result<Vec<UserID>>;
    fn add_admin(&mut self, user_id: UserID, session_id: SessionID) -> Result<()>;
    fn del_admin(&mut self, user_id: &UserID, session_id: &SessionID) -> Result<()>;


    fn is_admin(&self, session_id: &SessionID, user_id: &UserID) -> Result<bool> {
        let admins = self.get_admins(session_id)?;
        Ok(admins.contains(user_id))
    }
}


pub trait StoryDataProvider: Send + Sync {}
// {
//     type D: StoryData;

//     fn get_story_data<'a>(&'a mut self) -> &'a mut Self::D;
// }
