use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::{OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension};
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
             ON sessions(user_id);"
    ).map_err(|e| e.to_string())
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
}
