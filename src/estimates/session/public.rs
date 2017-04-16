use super::Session;
use super::SessionID;

use estimates::participant::{Participant, PublicParticipant};
use estimates::vote::VoteState;

use user::UserID;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Clean,
    Voting,
    Visible,
    Dirty,
}


#[derive(Serialize)]
#[derive(Debug)]
pub struct PublicSession {
    pub session_id: SessionID,
    pub users: Vec<PublicParticipant>,
    pub admins: Vec<UserID>,
    pub average: Option<f32>,
    pub state: SessionState,
}

impl PublicSession {
    pub fn new(
        session: Session,
        participants: Vec<Participant>,
        admins: Vec<UserID>
    ) -> PublicSession {
        let state = choose_session_state(&participants.iter().map(|p| p.vote).collect());
        PublicSession {
            session_id: session.session_id,
            users: participants.into_iter()
                .map(|p| PublicParticipant::from(p))
                .collect(),
            average: session.average,
            admins: admins,
            state: state,
        }
    }
}

fn choose_session_state(votes: &Vec<VoteState>) -> SessionState {
    let mut state = SessionState::Clean;
    for vote in votes {
        match (state, *vote) {
            (SessionState::Clean, VoteState::Empty) => state = SessionState::Voting,
            (SessionState::Clean, VoteState::Hidden(_)) => state = SessionState::Voting,
            (SessionState::Clean, VoteState::Visible(_)) => state = SessionState::Visible,
            (SessionState::Voting, VoteState::Visible(_)) => return SessionState::Dirty,
            (SessionState::Visible, VoteState::Empty) => return SessionState::Dirty,
            (SessionState::Visible, VoteState::Hidden(_)) => return SessionState::Dirty,
            _ => (),
        }
    }
    state
}
