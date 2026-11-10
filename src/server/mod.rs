pub mod handlers;
pub mod video_bridge;
pub mod routes;
pub mod static_files;
pub mod translate;

use crate::catalogue::CatalogData;
use crate::scraper::Scraper;
use crate::user::{AuthClient, UserManager};
use crate::user::accounts::AccountStore;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

pub struct AppState {
    pub catalog: CatalogData,
    pub catalog_error: Option<String>,
    pub scraper: Scraper,
    pub user_mgr: UserManager,
    pub auth: AuthClient,
    pub accounts: AccountStore,
    pub is_syncing: AtomicBool,
    pub translation_cache: translate::TranslationCache,
}


const MAX_ACTIVE_HTTP_REQUESTS: usize = 32;

struct RequestPermit(Arc<AtomicUsize>);

impl RequestPermit {
    fn try_acquire(active: &Arc<AtomicUsize>) -> Option<Self> {
        active
            .try_update(
                Ordering::AcqRel,
                Ordering::Acquire,
                |count| {
                    (count < MAX_ACTIVE_HTTP_REQUESTS)
                        .then_some(count + 1)
                },
            )
            .ok()?;

        Some(Self(Arc::clone(active)))
    }
}

impl Drop for RequestPermit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

pub fn start_server(port: u16) -> (Arc<AppState>, thread::JoinHandle<()>) {
    let server = tiny_http::Server::http(format!("127.0.0.1:{}", port))
        .expect("Impossible de démarrer le serveur HTTP");

    println!("[Serveur] Écoute sur http://127.0.0.1:{}", port);

    let (catalog, catalog_error) = match CatalogData::load() {
        Ok(c) => (c, None),
        Err(e) => {
            eprintln!("[Erreur] Chargement catalogue: {}", e);
            (CatalogData::empty(), Some(e))
        }
    };

    crate::user::paths::init_app_dirs();
    let userdata_file = crate::user::paths::userdata_path();
    let user_mgr = UserManager::new(userdata_file.to_str().unwrap_or("userdata.json"));
    let auth = AuthClient::new();

    // Stockage local Cataclyse, distinct de la session AnimeSama.
    let accounts_dir = userdata_file.with_file_name("accounts");

    std::fs::create_dir_all(&accounts_dir)
        .expect("Impossible de créer le répertoire des comptes");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        assert!(
            !accounts_dir.is_symlink(),
            "Répertoire des comptes non autorisé : lien symbolique"
        );

        std::fs::set_permissions(
            &accounts_dir,
            std::fs::Permissions::from_mode(0o700),
        ).expect("Impossible de protéger le répertoire des comptes");
    }

    let accounts = AccountStore::open(
        &accounts_dir.join("accounts.sqlite3")
    ).expect("Impossible d'initialiser les comptes Cataclyse");

    let state = Arc::new(AppState {
        catalog,
        catalog_error,
        scraper: Scraper::new(),
        user_mgr,
        auth,
        accounts,
        is_syncing: AtomicBool::new(false),
        translation_cache: translate::new_translation_cache(),
    });

    let s = state.clone();
    let active_requests = Arc::new(AtomicUsize::new(0));

    let handle = thread::spawn(move || {
        for incoming in server.incoming_requests() {
            let req = match incoming {
                Ok(req) => req,
                Err(error) => {
                    eprintln!(
                        "[Serveur] Requête HTTP rejetée: {:?}",
                        error
                    );
                    continue;
                }
            };
            let Some(permit) = RequestPermit::try_acquire(&active_requests)
            else {
                let response = tiny_http::Response::from_string(
                    "Trop de requêtes simultanées",
                )
                .with_status_code(tiny_http::StatusCode(503));

                let _ = req.respond(response);
                continue;
            };

            let st = s.clone();

            if let Err(error) = thread::Builder::new()
                .name("cataclyse-http".to_string())
                .spawn(move || {
                    let _permit = permit;
                    routes::handle_request(req, st);
                })
            {
                eprintln!(
                    "[Serveur] Création du thread HTTP impossible: {}",
                    error
                );
            }
        }
    });

    (state, handle)
}

#[cfg(test)]
mod tests;


#[cfg(test)]
mod concurrency_guard_tests {
    use super::*;

    #[test]
    fn limits_active_requests_and_releases_slots() {
        let active = Arc::new(AtomicUsize::new(0));
        let mut permits = Vec::new();

        for _ in 0..MAX_ACTIVE_HTTP_REQUESTS {
            permits.push(
                RequestPermit::try_acquire(&active)
                    .expect("Place disponible"),
            );
        }

        assert_eq!(
            active.load(Ordering::Acquire),
            MAX_ACTIVE_HTTP_REQUESTS
        );

        assert!(RequestPermit::try_acquire(&active).is_none());

        permits.pop();

        assert_eq!(
            active.load(Ordering::Acquire),
            MAX_ACTIVE_HTTP_REQUESTS - 1
        );

        let permit = RequestPermit::try_acquire(&active);
        assert!(permit.is_some());

        drop(permit);
        drop(permits);

        assert_eq!(active.load(Ordering::Acquire), 0);
    }
}
