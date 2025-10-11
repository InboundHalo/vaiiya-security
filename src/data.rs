use rusqlite::{Connection, Result as SqliteResult, params};
use std::path::Path;
use std::sync::Mutex;
use std::{error::Error, rc::Rc};

use twilight_model::id::{
    Id,
    marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker},
};

pub struct GuildSettings {
    pub guild_id: Id<GuildMarker>,
    pub verification_channel: Id<ChannelMarker>,
    pub verified_role: Id<RoleMarker>,
    pub verification_message: Id<MessageMarker>,
}

pub struct User {
    pub discord_user: Id<UserMarker>,
    pub embark_id: EmbarkID,
}

#[derive(Debug, Clone)]
pub struct EmbarkID {
    username: Rc<str>, // min 2 char max 16 char
    numbers: u16,      // up to 9999 min 0001
}

#[derive(Debug)]
pub enum EmbarkIDSterilizationErrors {
    InvalidFormat,
    UsernameMustBeBetween2And16Integers,
    NumbersMustBeValidIntegers,
}

impl EmbarkID {
    pub fn to_string(&self) -> String {
        format!("{}#{:04}", self.username, self.numbers)
    }

    pub fn new(id_string: &str) -> Result<EmbarkID, EmbarkIDSterilizationErrors> {
        let parts: Vec<&str> = id_string.split('#').collect();

        if parts.len() != 2 {
            return Err(EmbarkIDSterilizationErrors::InvalidFormat);
        }

        let username = parts[0].trim();
        let numbers_str = parts[1].trim();

        if username.len() < 2 || username.len() > 16 {
            return Err(EmbarkIDSterilizationErrors::UsernameMustBeBetween2And16Integers);
        }

        let numbers = match numbers_str.parse::<u16>() {
            Ok(n) => n,
            Err(_) => return Err(EmbarkIDSterilizationErrors::NumbersMustBeValidIntegers),
        };

        if numbers < 1 || numbers > 9999 {
            return Err(EmbarkIDSterilizationErrors::NumbersMustBeValidIntegers);
        }

        Ok(EmbarkID {
            username: username.into(),
            numbers,
        })
    }
}
// TODO: Figure out how to make this async
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let conn = Connection::open(path)?;
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS guild_settings (
                guild_id INTEGER PRIMARY KEY,
                verification_channel INTEGER NOT NULL,
                verified_role INTEGER NOT NULL,
                verification_message INTEGER NOT NULL
            );
            "#,
            [],
        )?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                discord_user INTEGER PRIMARY KEY,
                embark_id TEXT NOT NULL,
                UNIQUE(embark_id)
            );
            "#,
            [],
        )?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn get_guild_settings(&self, guild_id: Id<GuildMarker>) -> Option<GuildSettings> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT guild_id, verification_channel, verified_role, verification_message 
             FROM guild_settings WHERE guild_id = ?",
            )
            .ok()?;

        let mut rows = stmt.query(params![guild_id.get() as i64]).ok()?;

        match rows.next().ok()? {
            Some(row) => Some(GuildSettings {
                guild_id: Id::new(row.get(0).ok()?),
                verification_channel: Id::new(row.get(1).ok()?),
                verified_role: Id::new(row.get(2).ok()?),
                verification_message: Id::new(row.get(3).ok()?),
            }),
            None => None,
        }
    }

    pub fn set_guild_settings(&self, settings: &GuildSettings) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO guild_settings 
             (guild_id, verification_channel, verified_role, verification_message) 
             VALUES (?, ?, ?, ?)",
            params![
                settings.guild_id.get() as i64,
                settings.verification_channel.get() as i64,
                settings.verified_role.get() as i64,
                settings.verification_message.get() as i64
            ],
        )?;

        Ok(())
    }

    pub fn get_user_by_discord_id(
        &self,
        discord_user: Id<UserMarker>,
    ) -> SqliteResult<Option<User>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt =
            conn.prepare("SELECT discord_user, embark_id FROM users WHERE discord_user = ?")?;

        let mut rows = stmt.query(params![discord_user.get() as i64])?;

        match rows.next()? {
            Some(row) => {
                let embark_id_str: String = row.get(1)?;
                Ok(Some(User {
                    discord_user: Id::new(row.get(0)?),
                    embark_id: EmbarkID::new(&embark_id_str).expect("Database should be correct"),
                }))
            }
            None => Ok(None),
        }
    }

    pub fn get_user_by_embark_id(&self, embark_id: &EmbarkID) -> SqliteResult<Option<User>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt =
            conn.prepare("SELECT discord_user, embark_id FROM users WHERE embark_id = ?")?;

        let mut rows = stmt.query(params![embark_id.to_string()])?;

        match rows.next()? {
            Some(row) => {
                let embark_id_str: String = row.get(1)?;
                Ok(Some(User {
                    discord_user: Id::new(row.get(0)?),
                    embark_id: EmbarkID::new(&embark_id_str).expect("Database should be correct"),
                }))
            }
            None => Ok(None),
        }
    }

    pub fn add_user(&self, user: &User) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR IGNORE INTO users (discord_user, embark_id) VALUES (?, ?)",
            params![user.discord_user.get() as i64, user.embark_id.to_string(),],
        )?;

        Ok(())
    }

    pub fn update_user_embark_id(
        &self,
        discord_user: Id<UserMarker>,
        embark_id: &EmbarkID,
    ) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE users SET embark_id = ? WHERE discord_user = ?",
            params![embark_id.to_string(), discord_user.get() as i64],
        )?;

        Ok(())
    }

    pub fn remove_user(&self, discord_user: Id<UserMarker>) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "DELETE FROM users WHERE discord_user = ?",
            params![discord_user.get() as i64],
        )?;

        Ok(())
    }
}
