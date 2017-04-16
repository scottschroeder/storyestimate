use super::StoryData;
use errors::*;
use estimates::participant::Participant;
use estimates::session::{Session, SessionID};

use r2d2::PooledConnection;
use r2d2_redis::RedisConnectionManager;

use redis::{Commands, FromRedisValue, Value};
use serde::{Deserialize, Serialize};

use serde_json;
use std::fmt;

use user::{BasicUser, UserID};

const REDIS_BASE_KEY: &str = "STORYESTIMATES";

#[derive(Debug)]
enum RedisTable {
    User,
    Session,
    Participant,
}

#[derive(Debug)]
enum RedisSet {
    ParticipantUID,
    ParticipantName,
    Admin,
}

pub struct RedisDB {
    conn: PooledConnection<RedisConnectionManager>,
}

fn redis_set_key<K>(key: K, set: &RedisSet) -> String
    where K: fmt::Display
{
    let set_name = match *set {
        RedisSet::ParticipantUID => "PARTICIPANTUID",
        RedisSet::ParticipantName => "PARTICIPANTNAME",
        RedisSet::Admin => "ADMIN",
    };
    format!("{}_set_{}_{}", REDIS_BASE_KEY, set_name, key)
}

fn redis_table_key<K>(key: K, table: &RedisTable) -> String
    where K: fmt::Display
{
    let table_name = match *table {
        RedisTable::User => "USER",
        RedisTable::Session => "SESSION",
        RedisTable::Participant => "PARTICIPANT",
    };
    format!("{}_{}_{}", REDIS_BASE_KEY, table_name, key)
}

fn participant_key(sid: &SessionID, uid: &UserID) -> String {
    format!("{}_{}", sid, uid)
}

impl RedisDB {
    pub fn new(conn: PooledConnection<RedisConnectionManager>) -> Self {
        RedisDB { conn: conn }
    }

    fn get<T, K>(&self, key: K, table: &RedisTable) -> Result<Option<T>>
        where T: Deserialize,
              K: fmt::Display
    {
        let true_key = redis_table_key(key, table);
        match self.conn.get(true_key)? {
            Value::Nil => Ok(None),
            Value::Data(serialized) => {
                serde_json::from_slice(&serialized)
                    .map(|u| Some(u))
                    .map_err(|e| e.into())
            },
            x => bail!(ErrorKind::UnexpectedRedisResponse(x)),
        }
    }

    fn set<T, K>(&self, key: K, value: T, table: &RedisTable) -> Result<()>
        where T: Serialize,
              K: fmt::Display
    {
        let true_key = redis_table_key(key, table);
        let serialized_value = serde_json::to_string(&value).unwrap();
        // TODO Map redis result to our result
        let _: Value = self.conn.set(true_key, serialized_value).unwrap();
        Ok(())
    }

    fn del<K>(&self, key: K, table: &RedisTable) -> Result<bool>
        where K: fmt::Display
    {
        let true_key = redis_table_key(key, table);
        // TODO Map redis result to our result
        let redis_result: Value = self.conn.del(true_key).unwrap();
        i64::from_redis_value(&redis_result)
            .map(|n| n == 1)
            .map_err(|e| e.into())
    }

    fn sadd<T, K>(&self, key: K, value: T, set: &RedisSet) -> Result<()>
        where T: Serialize,
              K: fmt::Display
    {
        let true_key = redis_set_key(key, set);
        let serialized_value = serde_json::to_string(&value).unwrap();
        // TODO Map redis result to our result
        let _: Value = self.conn.sadd(true_key, serialized_value).unwrap();
        Ok(())
    }

    fn sismember<T, K>(&self, key: K, value: T, set: &RedisSet) -> Result<bool>
        where T: Serialize,
              K: fmt::Display
    {
        let true_key = redis_set_key(key, set);
        let serialized_value = serde_json::to_string(&value).unwrap();
        // TODO Map redis result to our result
        let redis_result: Value = self.conn.sismember(true_key, serialized_value).unwrap();
        i64::from_redis_value(&redis_result)
            .map(|n| n == 1)
            .map_err(|e| e.into())
    }
    fn smembers<T, K>(&self, key: K, set: &RedisSet) -> Result<Vec<T>>
        where T: Deserialize,
              K: fmt::Display
    {
        let true_key = redis_set_key(key, set);
        let all_serialized: Vec<String> = self.conn.smembers(true_key)?;
        all_serialized.iter()
            .map(|serialized| serde_json::from_str(&serialized).map_err(|e| e.into()))
            .collect()
    }
    fn srem<T, K>(&self, key: K, value: T, set: &RedisSet) -> Result<bool>
        where T: Serialize,
              K: fmt::Display
    {
        let true_key = redis_set_key(key, set);
        let serialized_value = serde_json::to_string(&value).unwrap();
        // TODO Map redis result to our result
        let redis_result: Value = self.conn.srem(true_key, serialized_value).unwrap();
        i64::from_redis_value(&redis_result)
            .map(|n| n == 1)
            .map_err(|e| e.into())
    }
}

fn strict<T>(redis_result: Result<Option<T>>) -> Result<T> {
    match redis_result {
        Ok(Some(x)) => Ok(x),
        Ok(None) => {
            Err(ErrorKind::ObjectNotFound("Expected key missing from redis".to_string()).into())
        },
        Err(e) => Err(e),
    }
}

impl StoryData for RedisDB {
    fn get_user(&self, user_id: &UserID) -> Result<Option<BasicUser>> {
        self.get(user_id, &RedisTable::User)
    }
    fn add_user(&mut self, user: BasicUser) -> Result<()> {
        self.set(&user.user_id, &user, &RedisTable::User)
    }

    fn get_session(&self, session_id: &SessionID) -> Result<Option<Session>> {
        self.get(session_id, &RedisTable::Session)
    }
    fn add_session(&mut self, session: Session) -> Result<()> {
        self.set(&session.session_id, &session, &RedisTable::Session)
    }
    fn del_session(&mut self, session_id: &SessionID) -> Result<()> {
        self.del(session_id, &RedisTable::Session)?;
        Ok(())
    }
    fn update_session<F>(&mut self, session_id: &SessionID, mut plan: F) -> Result<()>
        where F: FnMut(&mut Session, &mut Vec<Participant>) -> Result<()>
    {
        let mut participants = self.get_participants(session_id)?;
        let mut session = strict(self.get_session(session_id))?;
        let result = plan(&mut session, &mut participants);
        self.set(&session.session_id, &session, &RedisTable::Session)?;
        for p in participants {
            let pkey = participant_key(&p.session_id, &p.user_id);
            self.set(&pkey, &p, &RedisTable::Participant)?;
        }
        result
    }

    fn get_participants(&self, session_id: &SessionID) -> Result<Vec<Participant>> {
        let all_users: Vec<UserID> = self.smembers(&session_id, &RedisSet::ParticipantUID)?;
        all_users.iter()
            .map(|user_id| {
                let pkey = participant_key(&session_id, &user_id);
                let user: Result<Participant> = strict(self.get(pkey, &RedisTable::Participant))
                    .map_err(|_| {
                        ErrorKind::DataIntegrityError(format!("After finding {} in the \
                                                               participants for session {}, the \
                                                               corresponding Participant could \
                                                               not be found!",
                                                              &user_id,
                                                              &session_id))
                            .into()
                    });
                user
            })
            .collect()
    }

    // TODO These should get error codes
    fn add_participant(&mut self, participant: Participant) -> Result<()> {
        if self.sismember(&participant.session_id,
                       &participant.user_id,
                       &RedisSet::ParticipantUID)? {
            bail!(ErrorKind::UserError("That user is already part of this session".to_string()))
        }
        let pkey = participant_key(&participant.session_id, &participant.user_id);
        self.set(&pkey, &participant, &RedisTable::Participant)?;
        self.sadd(&participant.session_id,
                  &participant.nickname,
                  &RedisSet::ParticipantName)?;
        self.sadd(&participant.session_id,
                  &participant.user_id,
                  &RedisSet::ParticipantUID)?;
        Ok(())
    }

    // TODO These should get error codes
    fn del_participant(&mut self, user_id: &UserID, session_id: &SessionID) -> Result<()> {
        if !self.srem(session_id, user_id, &RedisSet::ParticipantUID)? {
            bail!(ErrorKind::ObjectNotFound(format!("User {} was not a member of session {}",
                                                    user_id,
                                                    session_id)));
        }
        self.srem(session_id, user_id, &RedisSet::ParticipantName)?;
        let pkey = participant_key(&session_id, &user_id);
        self.del(&pkey, &RedisTable::Participant)?;
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
        let pkey = participant_key(&session_id, &user_id);
        let mut participant: Participant = strict(self.get(&pkey, &RedisTable::Participant))?;
        let result = plan(&mut participant);
        self.set(&pkey, &participant, &RedisTable::Participant)?;
        result
    }
    fn get_admins(&self, session_id: &SessionID) -> Result<Vec<UserID>> {
        self.smembers(&session_id, &RedisSet::Admin)
    }
    fn add_admin(&mut self, user_id: UserID, session_id: SessionID) -> Result<()> {
        if self.sismember(&session_id, &user_id, &RedisSet::Admin)? {
            bail!(ErrorKind::UserError("That user is already an admin of this session".to_string()))
        }
        self.sadd(&session_id, &user_id, &RedisSet::Admin)?;
        Ok(())
    }
    fn del_admin(&mut self, user_id: &UserID, session_id: &SessionID) -> Result<()> {
        if !self.srem(session_id, user_id, &RedisSet::Admin)? {
            bail!(ErrorKind::UserError(format!("User {} was not an admin of session {}",
                                               user_id,
                                               session_id)));
        } else {
            Ok(())
        }
    }

    fn is_admin(&self, session_id: &SessionID, user_id: &UserID) -> Result<bool> {
        self.sismember(&session_id, &user_id, &RedisSet::Admin)
    }
}
