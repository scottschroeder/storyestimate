use super::VoteState;

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PublicVoteState {
    Empty,
    Hidden,
    Visible,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PublicVote {
    pub state: PublicVoteState,
    pub amount: Option<u32>,
}

impl<'a> From<&'a VoteState> for PublicVote {
    fn from(vote: &VoteState) -> PublicVote {
        let (vote_state, vote_amount) = match *vote {
            VoteState::Empty => (PublicVoteState::Empty, None),
            VoteState::Hidden(_) => (PublicVoteState::Hidden, None),
            VoteState::Visible(x) => (PublicVoteState::Visible, Some(x)),
        };
        PublicVote {
            state: vote_state,
            amount: vote_amount,
        }
    }
}

#[test]
fn convert_empty() {
    let v = VoteState::new();
    let pv = PublicVote::from(&v);
    assert_eq!(v, VoteState::Empty);
    assert_eq!(pv,
               PublicVote {
                   state: PublicVoteState::Empty,
                   amount: None,
               });
}

#[test]
fn convert_hidden() {
    let mut v = VoteState::new();
    v.vote(3);
    let pv = PublicVote::from(&v);
    assert_eq!(v, VoteState::Hidden(3));
    assert_eq!(pv,
               PublicVote {
                   state: PublicVoteState::Hidden,
                   amount: None,
               });
}

#[test]
fn convert_visible() {
    let mut v = VoteState::new();
    v.vote(3);
    v.reveal();
    let pv = PublicVote::from(&v);
    assert_eq!(v, VoteState::Visible(3));
    assert_eq!(pv,
               PublicVote {
                   state: PublicVoteState::Visible,
                   amount: Some(3),
               });
}
