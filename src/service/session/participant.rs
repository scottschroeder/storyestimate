use dal;
use errors::*;
use estimates::participant::Participant;
use estimates::session::SessionID;
use user::{AuthenticatedUser, Nickname, UserID};

pub fn join_session<D>(
    dal: &mut D,
    session_id: &SessionID,
    user_id: &UserID,
    user: &AuthenticatedUser,
    nickname: &Nickname
) -> Result<()>
    where D: dal::StoryData
{
    if *user_id != user.user_id {
        bail!(ErrorKind::UserUnauthorized);
    }
    if dal.get_session(session_id)?.is_none() {
        bail!(ErrorKind::ObjectNotFound(format!("Can not participate in non-existent session \
                                                 ID {:?}",
                                                session_id)));
    }

    let update_nickname = |mut p: &mut Participant| {
        p.nickname = nickname.clone();
        Ok(())
    };
    match dal.update_participant(session_id, &user.user_id, update_nickname) {
        Err(Error(ErrorKind::ObjectNotFound(_), _)) => {
            let member = Participant::new(user, session_id.clone(), nickname.clone());
            dal.add_participant(member)
        },
        x => x,
    }
}

pub fn place_vote<D>(
    dal: &mut D,
    session_id: &SessionID,
    user_id: &UserID,
    user: &AuthenticatedUser,
    vote: u32
) -> Result<()>
    where D: dal::StoryData
{
    if *user_id != user.user_id {
        bail!(ErrorKind::UserUnauthorized);
    }
    let place_vote = |mut p: &mut Participant| {
        p.vote(vote);
        Ok(())
    };
    dal.update_participant(session_id, &user.user_id, place_vote)
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use super::super::super::user;
    use dal::StoryData;
    use estimates::vote::VoteState;

    #[test]
    fn change_nickname() {
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
        let new_nickname = Nickname::new("bill");
        join_session(&mut dal,
                     &new_session_id,
                     &member_user.user_id,
                     &member_user,
                     &new_nickname)
            .unwrap();
        let all_participants = dal.get_participants(&new_session_id).unwrap();
        assert_eq!(all_participants[0].user_id, member_user.user_id);
        assert_eq!(all_participants[0].nickname, new_nickname);
    }

    #[test]
    fn join_empty_session() {
        let mut dal = dal::MemoryDB::new();
        let admin_user = user::get_authenticated_user(&mut dal).unwrap();
        let bad_session_id = SessionID("foo".to_string());
        let member_user = user::get_authenticated_user(&mut dal).unwrap();
        let nickname = Nickname::new("bob");
        let result = join_session(&mut dal,
                                  &bad_session_id,
                                  &member_user.user_id,
                                  &member_user,
                                  &nickname);

        match result {
            Err(Error(ErrorKind::ObjectNotFound(_), _)) => (),
            _ => panic!("Did not raise object not found error for missing session"),
        }
    }

    #[test]
    fn update_vote() {
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

        place_vote(&mut dal,
                   &new_session_id,
                   &member_user.user_id,
                   &member_user,
                   5)
            .unwrap();
        let all_participants = dal.get_participants(&new_session_id).unwrap();
        assert_eq!(all_participants[0].user_id, member_user.user_id);
        assert_eq!(all_participants[0].vote, VoteState::Hidden(5));
    }
}
