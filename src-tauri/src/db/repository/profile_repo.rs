use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension};

use crate::db::models::UserProfile;

use super::SettingsRepo;

pub const CURRENT_PROFILE_SETTING_KEY: &str = "profile.current_id";

pub struct ProfileRepo;

const PROFILE_COLUMNS: &str = "profile_id, profile_kind, display_name,
    avatar_storage_key, avatar_sha256, timezone_mode, timezone_id,
    week_start, created_at, updated_at";

impl ProfileRepo {
    pub fn ensure_default(conn: &Connection) -> Result<UserProfile> {
        let tx = conn.unchecked_transaction()?;

        if let Some(current_id) = SettingsRepo::get(&tx, CURRENT_PROFILE_SETTING_KEY)? {
            if let Some(profile) = Self::find_by_id(&tx, &current_id)? {
                tx.commit()?;
                return Ok(profile);
            }
        }

        if let Some(profile) = Self::find_first_local(&tx)? {
            SettingsRepo::set(&tx, CURRENT_PROFILE_SETTING_KEY, &profile.profile_id)?;
            tx.commit()?;
            return Ok(profile);
        }

        let profile_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO user_profiles (
                profile_id, profile_kind, display_name, timezone_mode, week_start
             ) VALUES (?1, 'local', 'User', 'system', 1)",
            [&profile_id],
        )?;
        SettingsRepo::set(&tx, CURRENT_PROFILE_SETTING_KEY, &profile_id)?;
        let profile = Self::find_by_id(&tx, &profile_id)?
            .context("default profile was not readable after insert")?;
        tx.commit()?;
        Ok(profile)
    }

    pub fn create_default(conn: &Connection) -> Result<UserProfile> {
        Self::ensure_default(conn)
    }

    pub fn get_current(conn: &Connection) -> Result<UserProfile> {
        let profile_id = SettingsRepo::get(conn, CURRENT_PROFILE_SETTING_KEY)?
            .context("current profile setting is missing")?;
        Self::find_by_id(conn, &profile_id)?
            .with_context(|| format!("current profile {profile_id} does not exist"))
    }

    pub fn find_by_id(conn: &Connection, profile_id: &str) -> Result<Option<UserProfile>> {
        conn.query_row(
            &format!("SELECT {PROFILE_COLUMNS} FROM user_profiles WHERE profile_id = ?1"),
            [profile_id],
            Self::map_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn update_display_name(
        conn: &Connection,
        profile_id: &str,
        display_name: &str,
    ) -> Result<UserProfile> {
        let affected = conn.execute(
            "UPDATE user_profiles
             SET display_name = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?2",
            rusqlite::params![display_name, profile_id],
        )?;
        if affected == 0 {
            anyhow::bail!("profile not found");
        }
        Self::find_by_id(conn, profile_id)?.context("profile not found after display name update")
    }

    pub fn update_avatar(
        conn: &Connection,
        profile_id: &str,
        storage_key: &str,
        sha256: &str,
    ) -> Result<UserProfile> {
        let affected = conn.execute(
            "UPDATE user_profiles
             SET avatar_storage_key = ?1, avatar_sha256 = ?2,
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?3",
            rusqlite::params![storage_key, sha256, profile_id],
        )?;
        if affected == 0 {
            anyhow::bail!("profile not found");
        }
        Self::find_by_id(conn, profile_id)?.context("profile not found after avatar update")
    }

    pub fn clear_avatar(conn: &Connection, profile_id: &str) -> Result<UserProfile> {
        let affected = conn.execute(
            "UPDATE user_profiles
             SET avatar_storage_key = NULL, avatar_sha256 = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?1",
            [profile_id],
        )?;
        if affected == 0 {
            anyhow::bail!("profile not found");
        }
        Self::find_by_id(conn, profile_id)?.context("profile not found after avatar clear")
    }

    pub fn update_timezone_snapshot(
        conn: &Connection,
        profile_id: &str,
        timezone_id: Option<&str>,
        week_start: i32,
    ) -> Result<UserProfile> {
        let affected = conn.execute(
            "UPDATE user_profiles
             SET timezone_mode = 'system', timezone_id = ?1, week_start = ?2,
                 updated_at = CURRENT_TIMESTAMP
             WHERE profile_id = ?3",
            rusqlite::params![timezone_id, week_start, profile_id],
        )?;
        if affected == 0 {
            anyhow::bail!("profile not found");
        }
        Self::find_by_id(conn, profile_id)?.context("profile not found after timezone update")
    }

    fn find_first_local(conn: &Connection) -> Result<Option<UserProfile>> {
        conn.query_row(
            &format!(
                "SELECT {PROFILE_COLUMNS} FROM user_profiles
                 WHERE profile_kind = 'local'
                 ORDER BY created_at, profile_id LIMIT 1"
            ),
            [],
            Self::map_row,
        )
        .optional()
        .map_err(Into::into)
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<UserProfile> {
        Ok(UserProfile {
            profile_id: row.get(0)?,
            profile_kind: row.get(1)?,
            display_name: row.get(2)?,
            avatar_storage_key: row.get(3)?,
            avatar_sha256: row.get(4)?,
            timezone_mode: row.get(5)?,
            timezone_id: row.get(6)?,
            week_start: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}
