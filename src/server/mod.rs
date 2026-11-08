pub mod handlers;
pub mod routes;
pub mod static_files;
pub mod translate;

use crate::catalogue::CatalogData;
use crate::scraper::Scraper;
use crate::user::{AuthClient, UserManager};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

pub struct AppState {
    pub catalog: CatalogData,
    pub catalog_error: Option<String>,
    pub scraper: Scraper,
    pub user_mgr: UserManager,
    pub auth: AuthClient,
    pub is_syncing: AtomicBool,
    pub translation_cache: translate::TranslationCache,
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

    let state = Arc::new(AppState {
        catalog,
        catalog_error,
        scraper: Scraper::new(),
        user_mgr,
        auth,
        is_syncing: AtomicBool::new(false),
        translation_cache: translate::new_translation_cache(),
    });

    let s = state.clone();
    let handle = thread::spawn(move || {
        for req in server.incoming_requests() {
            let st = s.clone();
            thread::spawn(move || {
                routes::handle_request(req, st);
            });
        }
    });

    (state, handle)
}

#[cfg(test)]
mod tests;
