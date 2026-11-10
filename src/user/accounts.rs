use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::{OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use crate::user::PlaybackPosition;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SESSION_TTL_SECONDS: i64 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalUser {
    pub id: i64,
    pub username: String,
}

pub struct AccountStore {
    db: Mutex<Connection>,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64
}

fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn valid_username(name: &str) -> bool {
    (3..=32).contains(&name.len())
        && name.bytes().all(|b| {
            b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
        })
}

fn init_database(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS users (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             username TEXT NOT NULL COLLATE NOCASE UNIQUE,
             password_hash TEXT NOT NULL,
             created_at INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS sessions (
             token_hash TEXT PRIMARY KEY,
             user_id INTEGER NOT NULL,
             created_at INTEGER NOT NULL,
             expires_at INTEGER NOT NULL,
             FOREIGN KEY(user_id) REFERENCES users(id)
                 ON DELETE CASCADE
         );

         CREATE INDEX IF NOT EXISTS idx_sessions_user
             ON sessions(user_id);

         CREATE TABLE IF NOT EXISTS playback_positions (
             user_id INTEGER NOT NULL,
             slug TEXT NOT NULL,
             season TEXT NOT NULL,
             episode INTEGER NOT NULL CHECK(episode > 0),
             current_time REAL NOT NULL,
             duration REAL NOT NULL,
             timestamp INTEGER NOT NULL,
             revision INTEGER NOT NULL,
             PRIMARY KEY(user_id, slug, season, episode),
             FOREIGN KEY(user_id) REFERENCES users(id)
                 ON DELETE CASCADE
         );"
    ).map_err(|e| e.to_string())
}


#[derive(Debug)]
pub enum PositionStoreError {
    Unauthorized,
    Invalid,
    Conflict,
    Storage(String),
}

impl From<rusqlite::Error> for PositionStoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(error.to_string())
    }
}

fn session_owner(
    conn: &Connection,
    token: &str,
) -> Result<Option<i64>, PositionStoreError> {
    if token.len() != 64
        || !token.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Ok(None);
    }

    conn.query_row(
        "SELECT user_id FROM sessions
         WHERE token_hash = ?1 AND expires_at > ?2",
        params![token_hash(token), now()],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}

fn valid_position_key(
    slug: &str,
    season: &str,
    episode: usize,
) -> bool {
    !slug.is_empty()
        && slug.len() <= 200
        && !season.is_empty()
        && season.len() <= 100
        && (1..=10000).contains(&episode)
}

impl AccountStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| e.to_string())?;
        }

        let conn = Connection::open(path)
            .map_err(|e| e.to_string())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                path,
                std::fs::Permissions::from_mode(0o600),
            ).map_err(|e| e.to_string())?;
        }

        conn.busy_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;

        init_database(&conn)?;

        Ok(Self {
            db: Mutex::new(conn),
        })
    }

    pub fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LocalUser, String> {
        let name = username.trim().to_ascii_lowercase();

        if !valid_username(&name) {
            return Err("Identifiant invalide".into());
        }

        if !(12..=1024).contains(&password.len()) {
            return Err("Mot de passe de longueur invalide".into());
        }

        let salt = SaltString::generate(&mut OsRng);

        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| e.to_string())?
            .to_string();

        let conn = self.db.lock()
            .map_err(|_| "Verrou de base indisponible")?;

        conn.execute(
            "INSERT INTO users(username, password_hash, created_at)
             VALUES (?1, ?2, ?3)",
            params![name, hash, now()],
        ).map_err(|_| "Création de compte impossible")?;

        Ok(LocalUser {
            id: conn.last_insert_rowid(),
            username: name,
        })
    }

    pub fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<(LocalUser, String)>, String> {
        if password.len() > 1024 {
            return Ok(None);
        }

        let name = username.trim().to_ascii_lowercase();

        if !valid_username(&name) {
            return Ok(None);
        }

        let found: Option<(i64, String, String)> = {
            let conn = self.db.lock()
                .map_err(|_| "Verrou de base indisponible")?;

            conn.query_row(
                "SELECT id, username, password_hash
                 FROM users WHERE username = ?1",
                params![name],
                |row| Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                )),
            )
            .optional()
            .map_err(|e| e.to_string())?
        };

        let Some((id, username, hash)) = found else {
            return Ok(None);
        };

        let parsed = PasswordHash::new(&hash)
            .map_err(|e| e.to_string())?;

        if Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_err()
        {
            return Ok(None);
        }

        let mut random = [0u8; 32];
        OsRng.fill_bytes(&mut random);

        let token = random.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        let conn = self.db.lock()
            .map_err(|_| "Verrou de base indisponible")?;

        let issued_at = now();

        conn.execute(
            "INSERT INTO sessions
             (token_hash, user_id, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                token_hash(&token),
                id,
                issued_at,
                issued_at + SESSION_TTL_SECONDS
            ],
        ).map_err(|e| e.to_string())?;

        Ok(Some((LocalUser { id, username }, token)))
    }

    pub fn resolve(
        &self,
        token: &str,
    ) -> Result<Option<LocalUser>, String> {
        if token.len() != 64
            || !token.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Ok(None);
        }

        let conn = self.db.lock()
            .map_err(|_| "Verrou de base indisponible")?;

        conn.query_row(
            "SELECT u.id, u.username
             FROM sessions s
             JOIN users u ON u.id = s.user_id
             WHERE s.token_hash = ?1
               AND s.expires_at > ?2",
            params![token_hash(token), now()],
            |row| Ok(LocalUser {
                id: row.get(0)?,
                username: row.get(1)?,
            }),
        )
        .optional()
        .map_err(|e| e.to_string())
    }


    pub fn position_for_session(
        &self,
        token: &str,
        slug: &str,
        season: &str,
        episode: usize,
    ) -> Result<Option<PlaybackPosition>, PositionStoreError> {
        if !valid_position_key(slug, season, episode) {
            return Err(PositionStoreError::Invalid);
        }

        let conn = self.db.lock().map_err(|_| {
            PositionStoreError::Storage("Verrou indisponible".into())
        })?;

        let user_id = session_owner(&conn, token)?
            .ok_or(PositionStoreError::Unauthorized)?;

        let row: Option<(f64, f64, i64, i64)> = conn.query_row(
            "SELECT \"current_time\", duration, timestamp, revision
             FROM playback_positions
             WHERE user_id = ?1
               AND slug = ?2
               AND season = ?3
               AND episode = ?4",
            params![user_id, slug, season, episode as i64],
            |row| Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            )),
        ).optional()?;

        let Some((current_time, duration, timestamp, revision)) = row
        else {
            return Ok(None);
        };

        if timestamp < 0 || revision < 0
            || !current_time.is_finite()
            || !duration.is_finite()
        {
            return Err(PositionStoreError::Storage(
                "Position enregistrée invalide".into()
            ));
        }

        Ok(Some(PlaybackPosition {
            slug: slug.to_owned(),
            season: season.to_owned(),
            episode,
            current_time,
            duration,
            timestamp: timestamp as u64,
            revision: revision as u64,
        }))
    }

    pub fn save_position_for_session(
        &self,
        token: &str,
        mut position: PlaybackPosition,
    ) -> Result<PlaybackPosition, PositionStoreError> {
        if !valid_position_key(
            &position.slug, &position.season, position.episode
        )
            || !position.current_time.is_finite()
            || !position.duration.is_finite()
            || position.current_time < 0.0
            || position.duration < 0.0
            || position.current_time > 864000.0
            || position.duration > 864000.0
            || position.timestamp == 0
            || position.timestamp > i64::MAX as u64
            || position.revision > i64::MAX as u64
        {
            return Err(PositionStoreError::Invalid);
        }

        let mut conn = self.db.lock().map_err(|_| {
            PositionStoreError::Storage("Verrou indisponible".into())
        })?;

        // Une transaction IMMEDIATE protège la comparaison
        // et la mise à jour des révisions.
        let tx = conn.transaction_with_behavior(
            TransactionBehavior::Immediate
        )?;

        let user_id = session_owner(&tx, token)?
            .ok_or(PositionStoreError::Unauthorized)?;

        let old: Option<i64> = tx.query_row(
            "SELECT revision FROM playback_positions
             WHERE user_id = ?1
               AND slug = ?2
               AND season = ?3
               AND episode = ?4",
            params![
                user_id,
                &position.slug,
                &position.season,
                position.episode as i64
            ],
            |row| row.get(0),
        ).optional()?;

        let next_revision = match old {
            None if position.revision == 0 => 1,
            Some(previous)
                if previous >= 0
                && previous as u64 == position.revision =>
            {
                previous.checked_add(1)
                    .ok_or(PositionStoreError::Conflict)?
            }
            _ => return Err(PositionStoreError::Conflict),
        };

        tx.execute(
            "INSERT INTO playback_positions
             (user_id, slug, season, episode,
              current_time, duration, timestamp, revision)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(user_id, slug, season, episode)
             DO UPDATE SET
                current_time = excluded.current_time,
                duration = excluded.duration,
                timestamp = excluded.timestamp,
                revision = excluded.revision",
            params![
                user_id,
                &position.slug,
                &position.season,
                position.episode as i64,
                position.current_time,
                position.duration,
                position.timestamp as i64,
                next_revision
            ],
        )?;

        tx.commit()?;

        position.revision = next_revision as u64;
        Ok(position)
    }

    pub fn logout(&self, token: &str) -> Result<(), String> {
        let conn = self.db.lock()
            .map_err(|_| "Verrou de base indisponible")?;

        conn.execute(
            "DELETE FROM sessions WHERE token_hash = ?1",
            params![token_hash(token)],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> AccountStore {
        let conn = Connection::open_in_memory().unwrap();
        init_database(&conn).unwrap();
        AccountStore {
            db: Mutex::new(conn),
        }
    }

    #[test]
    fn two_accounts_have_distinct_sessions() {
        let db = store();

        let a = db.register("alice", "Alice-secret-2026").unwrap();
        let b = db.register("bob", "Bob-secret-2026").unwrap();

        assert_ne!(a.id, b.id);

        let (_, token_a) =
            db.login("alice", "Alice-secret-2026").unwrap().unwrap();
        let (_, token_b) =
            db.login("bob", "Bob-secret-2026").unwrap().unwrap();

        assert_ne!(token_a, token_b);
        assert_eq!(db.resolve(&token_a).unwrap(), Some(a));
        assert_eq!(db.resolve(&token_b).unwrap(), Some(b));
    }

    #[test]
    fn logout_only_revokes_current_session() {
        let db = store();
        db.register("alice", "Alice-secret-2026").unwrap();

        let (_, first) =
            db.login("alice", "Alice-secret-2026").unwrap().unwrap();
        let (user, second) =
            db.login("alice", "Alice-secret-2026").unwrap().unwrap();

        db.logout(&first).unwrap();

        assert_eq!(db.resolve(&first).unwrap(), None);
        assert_eq!(db.resolve(&second).unwrap(), Some(user));
    }

    #[test]
    fn invalid_password_is_rejected() {
        let db = store();
        db.register("alice", "Alice-secret-2026").unwrap();

        assert!(db.login("alice", "wrong-password").unwrap().is_none());
        assert!(db.login("unknown", "wrong-password").unwrap().is_none());
    }

    #[test]
    fn duplicate_username_is_rejected() {
        let db = store();
        db.register("alice", "Alice-secret-2026").unwrap();

        assert!(db.register("ALICE", "Another-secret-2026").is_err());
    }


    fn sample_position(time: f64, revision: u64) -> PlaybackPosition {
        PlaybackPosition {
            slug: "frieren".into(),
            season: "saison1/vostfr".into(),
            episode: 2,
            current_time: time,
            duration: 1560.0,
            timestamp: 1794311492377,
            revision,
        }
    }

    #[test]
    fn positions_are_isolated_between_accounts() {
        let db = store();

        db.register("alice", "Alice-secret-2026").unwrap();
        db.register("bob", "Bob-secret-2026").unwrap();

        let (_, alice) =
            db.login("alice", "Alice-secret-2026").unwrap().unwrap();
        let (_, bob) =
            db.login("bob", "Bob-secret-2026").unwrap().unwrap();

        db.save_position_for_session(
            &alice, sample_position(600.0, 0)
        ).unwrap();

        db.save_position_for_session(
            &bob, sample_position(300.0, 0)
        ).unwrap();

        let a = db.position_for_session(
            &alice, "frieren", "saison1/vostfr", 2
        ).unwrap().unwrap();

        let b = db.position_for_session(
            &bob, "frieren", "saison1/vostfr", 2
        ).unwrap().unwrap();

        assert_eq!(a.current_time, 600.0);
        assert_eq!(b.current_time, 300.0);
        assert_eq!(a.revision, 1);
        assert_eq!(b.revision, 1);
    }

    #[test]
    fn stale_position_is_rejected_per_account() {
        let db = store();

        db.register("alice", "Alice-secret-2026").unwrap();

        let (_, token) =
            db.login("alice", "Alice-secret-2026").unwrap().unwrap();

        let first = db.save_position_for_session(
            &token, sample_position(600.0, 0)
        ).unwrap();

        assert_eq!(first.revision, 1);

        assert!(matches!(
            db.save_position_for_session(
                &token, sample_position(300.0, 0)
            ),
            Err(PositionStoreError::Conflict)
        ));

        let rewind = db.save_position_for_session(
            &token, sample_position(300.0, 1)
        ).unwrap();

        assert_eq!(rewind.current_time, 300.0);
        assert_eq!(rewind.revision, 2);
    }

    #[test]
    fn revoked_session_cannot_access_positions() {
        let db = store();

        db.register("alice", "Alice-secret-2026").unwrap();

        let (_, token) =
            db.login("alice", "Alice-secret-2026").unwrap().unwrap();

        db.save_position_for_session(
            &token, sample_position(600.0, 0)
        ).unwrap();

        db.logout(&token).unwrap();

        assert!(matches!(
            db.position_for_session(
                &token, "frieren", "saison1/vostfr", 2
            ),
            Err(PositionStoreError::Unauthorized)
        ));

        assert!(matches!(
            db.save_position_for_session(
                &token, sample_position(200.0, 1)
            ),
            Err(PositionStoreError::Unauthorized)
        ));
    }

    #[test]
    fn position_survives_sqlite_reopen() {
        let mut random = [0u8; 16];
        OsRng.fill_bytes(&mut random);

        let id = u128::from_le_bytes(random);

        let directory = std::env::temp_dir().join(
            format!(
                "cataclyse-position-test-{}-{}",
                std::process::id(), id
            )
        );

        let path = directory.join("accounts.sqlite3");

        let first = AccountStore::open(&path).unwrap();

        first.register(
            "alice", "Alice-secret-2026"
        ).unwrap();

        let (_, token) = first.login(
            "alice", "Alice-secret-2026"
        ).unwrap().unwrap();

        first.save_position_for_session(
            &token, sample_position(777.0, 0)
        ).unwrap();

        drop(first);

        let second = AccountStore::open(&path).unwrap();

        let saved = second.position_for_session(
            &token, "frieren", "saison1/vostfr", 2
        ).unwrap().unwrap();

        assert_eq!(saved.current_time, 777.0);
        assert_eq!(saved.revision, 1);

        drop(second);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn expired_session_is_rejected() {
        let db = store();
        db.register("alice", "Alice-secret-2026").unwrap();

        let (_, token) =
            db.login("alice", "Alice-secret-2026").unwrap().unwrap();

        {
            let conn = db.db.lock().unwrap();
            conn.execute(
                "UPDATE sessions SET expires_at = 0",
                [],
            ).unwrap();
        }

        assert!(db.resolve(&token).unwrap().is_none());
    }

    #[test]
    fn sqlite_persistence_survives_reopen() {
        let mut nonce = [0u8; 16];
        OsRng.fill_bytes(&mut nonce);

        let id = u128::from_le_bytes(nonce);

        let directory = std::env::temp_dir().join(
            format!("cataclyse-account-test-{}-{}", std::process::id(), id)
        );

        let path = directory.join("accounts.sqlite3");

        let first = AccountStore::open(&path).unwrap();

        let alice = first.register(
            "alice", "Alice-persistence-2026"
        ).unwrap();

        let (_, token) = first.login(
            "alice", "Alice-persistence-2026"
        ).unwrap().unwrap();

        drop(first);

        let reopened = AccountStore::open(&path).unwrap();

        assert_eq!(
            reopened.resolve(&token).unwrap(),
            Some(alice.clone())
        );

        let (authenticated, _) = reopened.login(
            "alice", "Alice-persistence-2026"
        ).unwrap().unwrap();

        assert_eq!(authenticated, alice);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = std::fs::metadata(&path)
                .unwrap()
                .permissions()
                .mode() & 0o777;

            assert_eq!(mode, 0o600);
        }

        drop(reopened);
        std::fs::remove_dir_all(directory).unwrap();
    }

}
