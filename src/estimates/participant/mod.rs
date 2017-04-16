use super::session::SessionID;
use super::vote::VoteState;
use user::{Nickname, User, UserID};

mod public;
pub use self::public::PublicParticipant;

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Participant {
    pub user_id: UserID,
    pub session_id: SessionID,
    pub nickname: Nickname,
    pub vote: VoteState,
}

impl Participant {
    pub fn new<U>(user: &U, session_id: SessionID, nickname: Nickname) -> Self
        where U: User
    {
        Participant {
            user_id: user.user_id().clone(),
            session_id: session_id,
            nickname: nickname,
            vote: VoteState::new(),
        }
    }

    pub fn vote(&mut self, amount: u32) {
        self.vote.vote(amount)
    }
}
