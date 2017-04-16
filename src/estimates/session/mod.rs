use super::participant::Participant;
use super::vote::PublicVote;

mod id;
mod public;

pub use self::id::SessionID;
pub use self::public::{PublicSession, SessionState};


#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub session_id: SessionID,
    pub average: Option<f32>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            session_id: SessionID::new(),
            average: None,
        }
    }

    pub fn take_votes(&mut self, participants: &mut Vec<Participant>) {
        let (count, total): (u32, u32) = participants.iter_mut()
            .map(|p| {
                p.vote.reveal();
                p.vote
            })
            .filter_map(|v| PublicVote::from(&v).amount)
            .fold((0, 0), |(count, sum), i| (count + 1, sum + i));

        if count > 0 {
            self.average = Some(total as f32 / count as f32);
        } else {
            self.average = None;
        }
    }

    pub fn clear(&mut self, participants: &mut Vec<Participant>) {
        for participant in participants {
            participant.vote.clear();
        }
        self.average = None;
    }

    pub fn reset(&mut self, participants: &mut Vec<Participant>) {
        for participant in participants {
            participant.vote.reset();
        }
        self.average = None;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use user::{BasicUser, Nickname};

    #[test]
    fn create_session() {
        let s = Session::new();
        assert_eq!(s.average, None);
    }

    #[test]
    fn unique_session_id() {
        let s1 = Session::new();
        let s2 = Session::new();
        assert!(s1.session_id != s2.session_id);
    }

    #[test]
    fn take_votes_average() {
        let s = Session::new();
        let new_user = BasicUser::new();
        let mut u = Participant::new(&new_user, s.session_id.clone(), Nickname::new("bob"));
        let new_user2 = BasicUser::new();
        let mut u2 = Participant::new(&new_user2, s.session_id.clone(), Nickname::new("bill"));
        u.vote(4);
        u2.vote(6);
        // s.join_participant(u).unwrap();
        // s.join_participant(u2).unwrap();
        // s.take_votes();
        // assert_eq!(s.average, Some(5f32));
    }
}
