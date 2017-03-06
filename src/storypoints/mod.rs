
pub fn points() -> String {
    "3".to_string()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum VoteState {
    Empty,
    Hidden(u32),
    Visible(u32),
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct User {
    user_token: u32,
    nickname: String,
    vote: VoteState
}

impl User {
    fn new(name: &str) -> Self {
        User {
            user_token: 1,
            nickname: name.to_owned(),
            vote: VoteState::Empty,
        }
    }

    fn vote(&mut self, points: u32) {
        self.vote = VoteState::Hidden(points);
    }

    /// Take any hidden votes and make them visible
    fn reveal(&mut self) {
        self.vote = match self.vote {
            VoteState::Empty => VoteState::Empty,
            VoteState::Hidden(x) => VoteState::Visible(x),
            VoteState::Visible(x) => VoteState::Visible(x),
        };
    }

    /// Take any visible votes and make them empty
    fn clear(&mut self) {
        self.vote = match self.vote {
            VoteState::Empty => VoteState::Empty,
            VoteState::Hidden(x) => VoteState::Hidden(x),
            VoteState::Visible(_) => VoteState::Empty,
        };
    }

    /// Take any visible votes and make them empty
    fn reset(&mut self) {
        self.vote = VoteState::Empty;
    }
}

#[derive(Debug)]
struct Session {
    session_id: u32,
    session_admin_token: u32,
    users: Vec<User>,
    average: Option<f32>,
}

impl Session {
    pub fn new() -> Self {
        Session {
            session_id: 1,
            session_admin_token: 1,
            users: vec![],
            average: None,
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.push(user);
    }

    pub fn take_votes(&mut self) {
        let mut total = 0u32;
        let mut count = 0u32;
        for user in &mut self.users {
            user.reveal();
            match user.vote {
                VoteState::Visible(x) => {
                    total += x;
                    count += 1;
                },
                _ => (),
            }
        }
        self.average = Some(total as f32 / count as f32);
    }

    pub fn clear(&mut self) {
        for user in &mut self.users {
            user.clear();
        }
        self.average = None;
    }

    pub fn reset(&mut self) {
        for user in &mut self.users {
            user.reset();
        }
        self.average = None;
    }

    pub fn kick_user(&mut self, nickname: &str) {
        self.users.retain(|ref x| x.nickname != nickname);
    }

}

#[test]
fn create_session() {
    let s = Session::new();
    assert_eq!(s.users, vec![]);
    assert_eq!(s.average, None);
}

#[test]
fn add_user_to_session() {
    let mut s = Session::new();
    assert_eq!(s.users, vec![]);
    let u = User::new("joe");
    let u_vec = vec![u.clone()];
    s.add_user(u);
    assert_eq!(u_vec, s.users);
}

#[test]
fn user_vote() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    assert_eq!(u.vote, VoteState::Hidden(3));
}

#[test]
fn user_vote_reveal() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reveal();
    assert_eq!(u.vote, VoteState::Visible(3));
}

#[test]
fn user_vote_clear() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reveal();
    assert_eq!(u.vote, VoteState::Visible(3));
    u.clear();
    assert_eq!(u.vote, VoteState::Empty);
}

#[test]
fn user_dont_clear_hidden() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    assert_eq!(u.vote, VoteState::Hidden(3));
    u.clear();
    assert_eq!(u.vote, VoteState::Hidden(3));
}

#[test]
fn user_vote_reset() {
    let mut u = User::new("joe");
    assert_eq!(u.vote, VoteState::Empty);
    u.vote(3);
    u.reset();
    assert_eq!(u.vote, VoteState::Empty);
}

#[test]
fn take_votes_average() {
    let mut s = Session::new();
    let mut u = User::new("joe");
    let mut u2 = User::new("john");
    u.vote(4);
    u2.vote(6);
    s.add_user(u);
    s.add_user(u2);
    s.take_votes();
    assert_eq!(s.average, Some(5f32));
}

#[test]
fn session_kick_user() {
    let mut s = Session::new();
    let u = User::new("joe");
    let u2 = User::new("john");
    let u_vec = vec![u.clone()];
    let u2_vec = vec![u.clone(), u2.clone()];
    s.add_user(u);
    s.add_user(u2);
    assert_eq!(s.users, u2_vec);
    s.kick_user("john");
    assert_eq!(s.users, u_vec);
}
