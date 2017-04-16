use super::Participant;
use estimates::vote::{PublicVote, PublicVoteState};
use user::{Nickname, UserID};

#[derive(Serialize)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublicParticipant {
    pub user_id: UserID,
    pub nickname: Nickname,
    pub vote_state: PublicVoteState,
    pub vote_amount: Option<u32>,
}


impl<'a> From<Participant> for PublicParticipant {
    fn from(p: Participant) -> PublicParticipant {
        let publicvote: PublicVote = (&p.vote).into();
        PublicParticipant {
            user_id: p.user_id,
            nickname: p.nickname,
            vote_state: publicvote.state,
            vote_amount: publicvote.amount,
        }
    }
}
