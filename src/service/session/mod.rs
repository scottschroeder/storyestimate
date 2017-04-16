use dal;
use errors::*;
use estimates::participant::Participant;
use estimates::session::{PublicSession, Session, SessionID, SessionState};
use user::{AuthenticatedUser, UserID};

mod participant;
pub use self::participant::*;

pub fn create_session<D>(dal: &mut D, user: &AuthenticatedUser) -> Result<SessionID>
    where D: dal::StoryData
{
    let new_session = Session::new();
    let session_id = new_session.session_id.clone();
    dal.add_session(new_session)?;
    dal.add_admin(user.user_id.clone(), session_id.clone())?;
    Ok(session_id)
}

pub fn kick_user<D>(
    dal: &mut D,
    session_id: &SessionID,
    user_id: &UserID,
    requester: &AuthenticatedUser
) -> Result<()>
    where D: dal::StoryData
{
    if requester.user_id == *user_id || dal.is_admin(session_id, &requester.user_id)? {
        dal.del_participant(user_id, session_id)
    } else {
        bail!(ErrorKind::UserForbidden(format!("User {:?} is not an admin of the session {:?}",
                                               requester,
                                               session_id)))
    }
}

pub fn grant_admin<D>(
    dal: &mut D,
    session_id: &SessionID,
    user_id: &UserID,
    requester: &AuthenticatedUser
) -> Result<()>
    where D: dal::StoryData
{
    if let Some(_) = dal.get_session(session_id)? {
        if dal.is_admin(session_id, &requester.user_id)? {
            dal.add_admin(user_id.clone(), session_id.clone())?;
        } else {
            bail!(ErrorKind::UserForbidden(format!("User {:?} is not an admin of the session \
                                                    {:?}",
                                                   requester,
                                                   session_id)))
        }
    } else {
        bail!(ErrorKind::ObjectNotFound(format!("Can not make an admin for a non-existent \
                                                 session ID {:?}",
                                                session_id)));
    }
    Ok(())
}

pub fn revoke_admin<D>(
    dal: &mut D,
    session_id: &SessionID,
    user_id: &UserID,
    requester: &AuthenticatedUser
) -> Result<()>
    where D: dal::StoryData
{
    if dal.is_admin(session_id, &requester.user_id)? {
        dal.del_admin(user_id, session_id)
    } else {
        bail!(ErrorKind::UserForbidden(format!("User {:?} is not an admin of the session {:?}",
                                               requester,
                                               session_id)))
    }
}

fn do_session_reset(s: &mut Session, mut participants: &mut Vec<Participant>) -> Result<()> {
    s.reset(&mut participants);
    Ok(())
}
fn do_session_clear(s: &mut Session, mut participants: &mut Vec<Participant>) -> Result<()> {
    s.clear(&mut participants);
    Ok(())
}
fn do_session_vote(s: &mut Session, mut participants: &mut Vec<Participant>) -> Result<()> {
    s.take_votes(&mut participants);
    Ok(())
}

pub fn lookup_session<D>(dal: &D, session_id: &SessionID) -> Result<Option<PublicSession>>
    where D: dal::StoryData
{
    let participants = dal.get_participants(&session_id)?;
    let admins = dal.get_admins(&session_id)?;
    Ok(dal.get_session(&session_id)?
        .map(|s| PublicSession::new(s, participants, admins)))
}

pub fn update_session<D>(
    dal: &mut D,
    session_id: &SessionID,
    into_state: &SessionState,
    requester: &AuthenticatedUser
) -> Result<()>
    where D: dal::StoryData
{

    if !dal.is_admin(session_id, &requester.user_id)? {
        bail!(ErrorKind::UserForbidden(format!("User {:?} is not an admin of the session {:?}",
                                               requester,
                                               session_id)))
    }

    let session_action = match *into_state {
        SessionState::Clean => do_session_reset,
        SessionState::Voting => do_session_clear,
        SessionState::Visible => do_session_vote,
        SessionState::Dirty => {
            bail!(ErrorKind::UserError("Can not set session to be 'Dirty'".to_string()))
        },
    };
    dal.update_session(&session_id, session_action)
}


pub fn delete_session<D>(
    dal: &mut D,
    session_id: &SessionID,
    requester: &AuthenticatedUser
) -> Result<()>
    where D: dal::StoryData
{
    // Verify that the session exists first

    if let Some(_) = dal.get_session(session_id)? {
        if dal.is_admin(session_id, &requester.user_id)? {
            dal.del_session(session_id)
        } else {
            bail!(ErrorKind::UserForbidden(format!("User {} is not an admin of the session {}",
                                                   requester,
                                                   session_id)))
        }
    } else {
        bail!(ErrorKind::ObjectNotFound(format!("Can not delete non-existent session ID {:?}",
                                                session_id)));

    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::user;
    use dal::StoryData;
    use user::Nickname;


    #[test]
    fn create_session_and_check() {
        let mut dal = dal::MemoryDB::new();
        let auth_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &auth_user).unwrap();
        let saved_session = dal.get_session(&new_session_id).unwrap().unwrap();
        assert_eq!(new_session_id, saved_session.session_id);
    }

    #[test]
    fn join_session_and_check() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let member_user = user::get_authenticated_user(&mut dal).unwrap();
        let nickname = Nickname::new("bob");
        join_session(&mut dal,
                     &new_session_id,
                     &member_user.user_id,
                     &member_user,
                     &nickname)
            .unwrap();
        let all_participants = dal.get_participants(&new_session_id).unwrap();
        assert_eq!(member_user.user_id, all_participants[0].user_id);
    }

    #[test]
    fn leave_session_willingly() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let member_user = user::get_authenticated_user(&mut dal).unwrap();
        let nickname = Nickname::new("bob");
        join_session(&mut dal,
                     &new_session_id,
                     &member_user.user_id,
                     &member_user,
                     &nickname)
            .unwrap();
        kick_user(&mut dal,
                  &new_session_id,
                  &member_user.user_id,
                  &member_user)
            .unwrap();
        let all_participants = dal.get_participants(&new_session_id).unwrap();
        assert_eq!(all_participants.len(), 0);
    }

    #[test]
    fn kick_user_from_session() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let member_user = user::get_authenticated_user(&mut dal).unwrap();
        let nickname = Nickname::new("bob");
        join_session(&mut dal,
                     &new_session_id,
                     &member_user.user_id,
                     &member_user,
                     &nickname)
            .unwrap();
        kick_user(&mut dal, &new_session_id, &member_user.user_id, &admin_user).unwrap();
        let all_participants = dal.get_participants(&new_session_id).unwrap();
        assert_eq!(all_participants.len(), 0);
    }

    #[test]
    fn kick_user_without_admin_session() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let member_user = user::get_authenticated_user(&mut dal).unwrap();
        let non_admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let nickname = Nickname::new("bob");
        join_session(&mut dal,
                     &new_session_id,
                     &member_user.user_id,
                     &member_user,
                     &nickname)
            .unwrap();
        let outcome = kick_user(&mut dal,
                                &new_session_id,
                                &member_user.user_id,
                                &non_admin_user);
        assert!(outcome.is_err())
    }

    #[test]
    fn kick_non_existant_user() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let member_user = user::get_authenticated_user(&mut dal).unwrap();
        let non_member_user = user::get_authenticated_user(&mut dal).unwrap();
        let nickname = Nickname::new("bob");
        join_session(&mut dal,
                     &new_session_id,
                     &member_user.user_id,
                     &member_user,
                     &nickname)
            .unwrap();
        let outcome = kick_user(&mut dal,
                                &new_session_id,
                                &non_member_user.user_id,
                                &admin_user);
        assert!(outcome.is_err())
    }

    #[test]
    fn add_admin_to_session() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let new_admin_user = user::get_authenticated_user(&mut dal).unwrap();
        grant_admin(&mut dal,
                    &new_session_id,
                    &new_admin_user.user_id,
                    &admin_user)
            .unwrap();
        assert!(dal.is_admin(&new_session_id, &new_admin_user.user_id)
            .unwrap());
        assert_eq!(dal.get_admins(&new_session_id).unwrap().len(), 2);
    }

    #[test]
    fn add_admin_to_session_wout_creds() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let new_admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let outcome = grant_admin(&mut dal,
                                  &new_session_id,
                                  &new_admin_user.user_id,
                                  &new_admin_user);
        assert!(outcome.is_err());
        assert!(!dal.is_admin(&new_session_id, &new_admin_user.user_id)
            .unwrap());
    }

    #[test]
    fn remove_admin_from_session() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let new_admin_user = user::get_authenticated_user(&mut dal).unwrap();
        grant_admin(&mut dal,
                    &new_session_id,
                    &new_admin_user.user_id,
                    &admin_user)
            .unwrap();
        assert!(dal.is_admin(&new_session_id, &new_admin_user.user_id)
            .unwrap());
        revoke_admin(&mut dal,
                     &new_session_id,
                     &new_admin_user.user_id,
                     &admin_user)
            .unwrap();
        assert!(!dal.is_admin(&new_session_id, &new_admin_user.user_id)
            .unwrap());
    }

    #[test]
    fn remove_admin_from_session_wout_creds() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let new_admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_non_admin_user = user::get_authenticated_user(&mut dal).unwrap();
        grant_admin(&mut dal,
                    &new_session_id,
                    &new_admin_user.user_id,
                    &admin_user)
            .unwrap();
        assert!(dal.is_admin(&new_session_id, &new_admin_user.user_id)
            .unwrap());
        let outcome = revoke_admin(&mut dal,
                                   &new_session_id,
                                   &new_admin_user.user_id,
                                   &new_non_admin_user);
        assert!(outcome.is_err());
        assert!(dal.is_admin(&new_session_id, &new_admin_user.user_id)
            .unwrap());
    }

    #[test]
    fn get_session() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let pub_session: PublicSession = lookup_session(&dal, &new_session_id).unwrap().unwrap();
        assert_eq!(pub_session.session_id, new_session_id);
        assert_eq!(pub_session.users, vec![]);
        assert_eq!(pub_session.average, None);
        assert_eq!(pub_session.state, SessionState::Clean);
    }

    #[test]
    fn get_session_with_participants() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let new_session_id = create_session(&mut dal, &admin_user).unwrap();
        let member_user = user::get_authenticated_user(&mut dal).unwrap();
        let nickname = Nickname::new("bob");
        join_session(&mut dal,
                     &new_session_id,
                     &member_user.user_id,
                     &member_user,
                     &nickname)
            .unwrap();
        let pub_session: PublicSession = lookup_session(&dal, &new_session_id).unwrap().unwrap();
        assert_eq!(pub_session.session_id, new_session_id);
        assert_eq!(pub_session.users[0].user_id, member_user.user_id);
        assert_eq!(pub_session.average, None);
        assert_eq!(pub_session.state, SessionState::Voting);
    }

}
