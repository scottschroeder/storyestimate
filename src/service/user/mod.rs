use dal;
use errors::*;
use user::{AuthenticatedUser, BasicUser, UserID, UserToken};

pub fn create_user<D>(dal: &mut D) -> Result<BasicUser>
    where D: dal::StoryData
{
    let new_user = BasicUser::new();
    dal.add_user(new_user.clone())?;
    Ok(new_user)
}

pub fn authenticate_user<D>(
    dal: &D,
    user_id: &UserID,
    user_token: &UserToken
) -> Result<Option<AuthenticatedUser>>
    where D: dal::StoryData
{
    if let Some(existing_user) = dal.get_user(user_id)? {
        Ok(existing_user.authenticate(user_token))
    } else {
        Ok(None)
    }
}

pub fn get_authenticated_user<D>(dal: &mut D) -> Result<AuthenticatedUser>
    where D: dal::StoryData
{
    let new_user: BasicUser = create_user(dal)?;
    Ok(authenticate_user(dal, &new_user.user_id, &new_user.user_token)
        ?
        .unwrap())
}


#[cfg(test)]
mod test {
    use super::*;
    use dal::StoryData;

    #[test]
    fn create_user_and_check() {
        let mut dal = dal::MemoryDB::new();
        let new_user = create_user(&mut dal).unwrap();
        let saved_user = dal.get_user(&new_user.user_id).unwrap().unwrap();
        assert_eq!(new_user, saved_user);
    }

    #[test]
    fn auth_valid_user() {
        let mut dal = dal::MemoryDB::new();
        let new_user = create_user(&mut dal).unwrap();
        let auth_user = authenticate_user(&dal, &new_user.user_id, &new_user.user_token)
            .unwrap()
            .unwrap();
        assert_eq!(new_user.user_id, auth_user.user_id);
    }

    #[test]
    fn auth_invalid_user() {
        let mut dal = dal::MemoryDB::new();
        let new_user = create_user(&mut dal).unwrap();
        let bad_token = UserToken::new();
        let auth_user_opt = authenticate_user(&dal, &new_user.user_id, &bad_token).unwrap();
        assert_eq!(auth_user_opt, None);
    }

    #[test]
    fn auth_missing_user() {
        let mut dal = dal::MemoryDB::new();
        create_user(&mut dal).unwrap();
        let bad_user = BasicUser::new();
        let auth_user_opt = authenticate_user(&dal, &bad_user.user_id, &bad_user.user_token)
            .unwrap();
        assert_eq!(auth_user_opt, None);
    }

    #[test]
    fn new_auth_user() {
        let mut dal = dal::MemoryDB::new();
        let auth_user = get_authenticated_user(&mut dal).unwrap();
    }
}
