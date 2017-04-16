use super::StoryData;
use errors::*;
use estimates::participant::Participant;
use estimates::session::{Session, SessionID};
use std::collections::BTreeMap;
use user::{BasicUser, User, UserID};

/// An in memory database of story entities
/// Designed for testing, not production
/// Makes gratuitous copies to simulate DB
pub struct MemoryDB {
    users: BTreeMap<UserID, BasicUser>,
    sessions: BTreeMap<SessionID, Session>,
    participants: BTreeMap<SessionID, Vec<Participant>>,
    admins: BTreeMap<SessionID, Vec<UserID>>,
}

impl MemoryDB {
    pub fn new() -> Self {
        MemoryDB {
            users: BTreeMap::new(),
            sessions: BTreeMap::new(),
            participants: BTreeMap::new(),
            admins: BTreeMap::new(),
        }
    }
}

impl StoryData for MemoryDB {
    fn get_user(&self, user_id: &UserID) -> Result<Option<BasicUser>> {
        Ok(self.users.get(user_id).map(|u| u.clone()))
    }
    fn add_user(&mut self, user: BasicUser) -> Result<()> {
        self.users.insert(user.user_id().clone(), user);
        Ok(())
    }

    fn get_session(&self, session_id: &SessionID) -> Result<Option<Session>> {
        Ok(self.sessions.get(session_id).map(|s| s.clone()))
    }
    fn add_session(&mut self, session: Session) -> Result<()> {
        self.sessions.insert(session.session_id.clone(), session);
        Ok(())
    }
    fn update_session<F>(&mut self, session_id: &SessionID, mut plan: F) -> Result<()>
        where F: FnMut(&mut Session, &mut Vec<Participant>) -> Result<()>
    {
        let mut participants = self.participants
            .entry(session_id.clone())
            .or_insert(Vec::new());
        self.sessions
            .get_mut(session_id)
            .ok_or(ErrorKind::ObjectNotFound(format!("Could not find session: {:?}", session_id))
                .into())
            .and_then(|s| plan(s, &mut participants))
    }

    fn del_session(&mut self, session_id: &SessionID) -> Result<()> {
        self.sessions
            .remove(&session_id)
            .ok_or(ErrorKind::ObjectNotFound(format!("Could not find session: {:?}", session_id))
                .into())
            .map(|_| ())
    }

    fn get_participants(&self, session_id: &SessionID) -> Result<Vec<Participant>> {
        Ok(self.participants
            .get(session_id)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|p| p.clone())
            .collect())
    }
    // TODO These should get error codes
    fn add_participant(&mut self, participant: Participant) -> Result<()> {
        let session_id = participant.session_id.clone();
        let mut participants = self.get_participants(&session_id)?;
        for current_participant in &participants {
            if current_participant.user_id == participant.user_id {
                bail!(ErrorKind::UserError("That user is already part of this session".to_string()))
            }
        }

        participants.push(participant);
        self.participants.insert(session_id, participants);
        Ok(())
    }

    fn del_participant(&mut self, user_id: &UserID, session_id: &SessionID) -> Result<()> {
        let mut participants = self.get_participants(&session_id)?;
        let before = participants.len();
        participants.retain(|p| p.user_id != *user_id);
        let after = participants.len();
        if before == after {
            bail!(ErrorKind::ObjectNotFound(format!("User {} was not a member of session {}",
                                                    user_id,
                                                    session_id)));
        }
        self.participants.insert(session_id.clone(), participants);
        Ok(())
    }

    fn update_participant<F>(
        &mut self,
        session_id: &SessionID,
        user_id: &UserID,
        mut plan: F
    ) -> Result<()>
        where F: FnMut(&mut Participant) -> Result<()>
    {
        self.participants
            .entry(session_id.clone())
            .or_insert(Vec::new())
            .iter_mut()
            .filter(|p| p.user_id == *user_id)
            .nth(0)
            .ok_or(ErrorKind::ObjectNotFound(format!("Could not find user {:?} in session {:?}",
                                                     user_id,
                                                     session_id))
                .into())
            .and_then(|p| plan(p))
    }

    fn get_admins(&self, session_id: &SessionID) -> Result<Vec<UserID>> {
        Ok(self.admins
            .get(session_id)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|p| p.clone())
            .collect())
    }


    fn add_admin(&mut self, user_id: UserID, session_id: SessionID) -> Result<()> {
        let mut admins = self.get_admins(&session_id)?;
        for current_admin in &admins {
            if *current_admin == user_id {
                bail!(ErrorKind::UserError("That user is already an admin of this session"
                    .to_string()))
            }
        }
        admins.push(user_id);
        self.admins.insert(session_id, admins);
        Ok(())
    }
    fn del_admin(&mut self, user_id: &UserID, session_id: &SessionID) -> Result<()> {
        let mut admins = self.get_admins(&session_id)?;
        let before = admins.len();
        admins.retain(|uid| *uid != *user_id);
        let after = admins.len();
        if before == after {
            bail!(ErrorKind::UserError(format!("User {} was not an admin of session {}",
                                               user_id,
                                               session_id)));
        }
        self.admins.insert(session_id.clone(), admins);
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use user::Nickname;

    #[test]
    fn save_and_get_user() {
        let mut memdal = MemoryDB::new();
        let my_user = BasicUser::new();
        memdal.add_user(my_user.clone()).unwrap();
        let same_user = memdal.get_user(&my_user.user_id).unwrap().unwrap();
        assert!(my_user == same_user);
    }

    #[test]
    fn get_non_existant_user() {
        let memdal = MemoryDB::new();
        let user_id = UserID::new();
        let user_opt = memdal.get_user(&user_id).unwrap();
        assert!(user_opt == None);
    }

    #[test]
    fn save_and_get_session() {
        let mut memdal = MemoryDB::new();
        let my_session = Session::new();
        memdal.add_session(my_session.clone()).unwrap();
        let same_session = memdal.get_session(&my_session.session_id).unwrap().unwrap();
        assert!(my_session == same_session);
    }

    #[test]
    fn get_non_existant_session() {
        let memdal = MemoryDB::new();
        let session_id = SessionID::new();
        let session_opt = memdal.get_session(&session_id).unwrap();
        assert!(session_opt == None);
    }

    fn add_n_admins_and_check(n: usize) {
        let mut memdal = MemoryDB::new();
        let s = Session::new();
        let mut all_admins = Vec::new();
        for _ in 0..n {
            let u = BasicUser::new();
            memdal.add_admin(u.user_id.clone(), s.session_id.clone())
                .unwrap();
            all_admins.push(u.user_id);
        }

        let same_admins = memdal.get_admins(&s.session_id).unwrap();
        assert_eq!(same_admins, all_admins);
    }

    fn add_n_participants_and_check(n: usize) {
        let mut memdal = MemoryDB::new();
        let s = Session::new();
        let mut all_participants = Vec::new();
        for i in 0..n {
            let new_user = BasicUser::new();
            let u = Participant::new(&new_user,
                                     s.session_id.clone(),
                                     Nickname::new(format!("bob_{}", i)));
            memdal.add_participant(u.clone()).unwrap();
            all_participants.push(u);
        }

        let same_participants = memdal.get_participants(&s.session_id).unwrap();
        assert_eq!(same_participants, all_participants);
    }


    #[test]
    fn save_and_get_admins() {
        for i in 0..32 {
            add_n_admins_and_check(i);
        }
    }
    #[test]
    fn save_and_get_participant() {
        for i in 0..32 {
            add_n_participants_and_check(i);
        }
    }

    #[test]
    fn remove_participant() {
        let mut memdal = MemoryDB::new();
        let s = Session::new();
        let new_user = BasicUser::new();
        let u = Participant::new(&new_user, s.session_id.clone(), Nickname::new("bob"));
        memdal.add_participant(u.clone()).unwrap();

        let new_user2 = BasicUser::new();
        let u2 = Participant::new(&new_user2, s.session_id.clone(), Nickname::new("bill"));
        memdal.add_participant(u2.clone()).unwrap();

        memdal.del_participant(&u.user_id, &s.session_id).unwrap();

        let all_participants = memdal.get_participants(&s.session_id).unwrap();

        assert_eq!(all_participants.len(), 1);
        assert_eq!(all_participants[0].user_id, u2.user_id);

    }

    #[test]
    fn remove_nonexistant_participant() {
        let mut memdal = MemoryDB::new();
        let s = Session::new();
        let new_user = BasicUser::new();
        let outcome = memdal.del_participant(&new_user.user_id, &s.session_id);
        assert!(outcome.is_err())
    }

    #[test]
    fn remove_admin() {
        let mut memdal = MemoryDB::new();
        let s = Session::new();
        let new_user = BasicUser::new();
        memdal.add_admin(new_user.user_id.clone(), s.session_id.clone())
            .unwrap();

        let new_user2 = BasicUser::new();
        memdal.add_admin(new_user2.user_id.clone(), s.session_id.clone())
            .unwrap();

        memdal.del_admin(&new_user.user_id, &s.session_id).unwrap();

        let all_admins = memdal.get_admins(&s.session_id).unwrap();

        assert_eq!(all_admins.len(), 1);
        assert_eq!(all_admins[0], new_user2.user_id);

    }

    #[test]
    fn remove_nonexistant_admin() {
        let mut memdal = MemoryDB::new();
        let s = Session::new();
        let new_user = BasicUser::new();
        let outcome = memdal.del_admin(&new_user.user_id, &s.session_id);
        assert!(outcome.is_err())
    }

    #[test]
    fn test_is_admin() {
        let mut memdal = MemoryDB::new();
        let s = Session::new();
        let real_admin = BasicUser::new();
        memdal.add_admin(real_admin.user_id.clone(), s.session_id.clone())
            .unwrap();

        let fake_admin = BasicUser::new();

        assert!(memdal.is_admin(&s.session_id, &real_admin.user_id).unwrap());
        assert!(!memdal.is_admin(&s.session_id, &fake_admin.user_id).unwrap());
    }

    #[test]
    fn create_and_delete_session() {
        let mut memdal = MemoryDB::new();
        let my_session = Session::new();
        memdal.add_session(my_session.clone()).unwrap();
        memdal.del_session(&my_session.session_id).unwrap();
        let outcome = memdal.get_session(&my_session.session_id).unwrap();
        assert_eq!(outcome, None);
    }

    #[test]
    fn delete_non_existent_session() {
        let mut memdal = MemoryDB::new();
        let my_session = Session::new();
        let outcome = memdal.del_session(&my_session.session_id);
        assert!(match outcome.unwrap_err() {
            Error(ErrorKind::ObjectNotFound(_), _) => true,
            _ => false,
        });
    }
    #[test]
    fn create_and_update_session() {
        let mut memdal = MemoryDB::new();
        let my_session = Session::new();
        memdal.add_session(my_session.clone()).unwrap();
        let magic_number = 1234.5f32;
        let add_magic = |s: &mut Session, _: &mut Vec<Participant>| -> Result<()> {
            s.average = Some(magic_number);
            Ok(())
        };

        memdal.update_session(&my_session.session_id, add_magic)
            .unwrap();
        let same_session = memdal.get_session(&my_session.session_id).unwrap().unwrap();
        assert_eq!(same_session.average, Some(magic_number));

    }

    #[test]
    fn create_and_update_participant() {
        let mut memdal = MemoryDB::new();
        let my_session = Session::new();
        memdal.add_session(my_session.clone()).unwrap();
        let new_user = BasicUser::new();
        let u = Participant::new(&new_user,
                                 my_session.session_id.clone(),
                                 Nickname::new("bob"));
        memdal.add_participant(u.clone()).unwrap();
        let new_name = Nickname::new("bill");
        let update_username = |mut p: &mut Participant| {
            p.nickname = new_name.clone();
            Ok(())
        };
        memdal.update_participant(&my_session.session_id, &new_user.user_id, update_username)
            .unwrap();
        let all_participants = memdal.get_participants(&my_session.session_id).unwrap();
        assert_eq!(all_participants[0].nickname, new_name);

    }
}
