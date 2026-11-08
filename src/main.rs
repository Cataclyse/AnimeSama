mod adblock;
mod catalogue;
mod proxy;
mod scraper;
mod server;
mod user;

use std::env;
use std::net::TcpListener;

#[cfg(feature = "gui")]
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
#[cfg(feature = "gui")]
use wry::WebViewBuilder;

fn find_available_port(start: u16) -> u16 {
    for port in start..start + 50 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    start
}

#[cfg(feature = "gui")]
fn load_window_icon() -> Option<tao::window::Icon> {
    let bytes = include_bytes!("../assets/icon_64.rgba");
    tao::window::Icon::from_rgba(bytes.to_vec(), 64, 64).ok()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_headless = args.iter().any(|a| a == "--headless" || a == "-s");
    let force_browser = args.iter().any(|a| a == "--browser" || a == "-b");

    println!("==================================================");
    println!("  ANIME-SAMA (RUST)");
    println!("==================================================");

    let port = find_available_port(4567);
    let app_url = format!("http://127.0.0.1:{}", port);

    let (_state, _server_thread) = server::start_server(port);

    if is_headless {
        println!("[Info] Mode serveur headless activé sur {}", app_url);
        println!("[Info] Appuyez sur Ctrl+C pour arrêter le serveur.");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }

    if force_browser {
        println!("[Info] Ouverture dans le navigateur par défaut : {}", app_url);
        let _ = opener::open(&app_url);
        println!("[Info] Appuyez sur Ctrl+C pour arrêter l'application.");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }

    #[cfg(not(feature = "gui"))]
    {
        println!("[Info] Mode Web / Serveur : ouverture de {}", app_url);
        let _ = opener::open(&app_url);
        println!("[Info] Serveur actif sur {}. Appuyez sur Ctrl+C pour arrêter.", app_url);
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }

    #[cfg(feature = "gui")]
    {
        println!("[Info] Lancement de la fenêtre native sur {}...", app_url);
        let event_loop = EventLoop::new();

        let mut window_builder = WindowBuilder::new()
            .with_title("Anime-Sama")
            .with_inner_size(tao::dpi::LogicalSize::new(1440.0, 900.0))
            .with_min_inner_size(tao::dpi::LogicalSize::new(1024.0, 700.0));

        if let Some(icon) = load_window_icon() {
            window_builder = window_builder.with_window_icon(Some(icon));
        }

        let window = match window_builder.build(&event_loop) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[Erreur] Impossible de créer la fenêtre native: {}. Ouverture dans le navigateur...", e);
                let _ = opener::open(&app_url);
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
        };

        let window = std::sync::Arc::new(window);
        let window_title = std::sync::Arc::clone(&window);

        let _webview = match WebViewBuilder::new()
            .with_url(&app_url)
            .with_document_title_changed_handler(move |title| {
                window_title.set_title(&title);
            })
            .build(&*window)
        {
            Ok(wv) => wv,
            Err(e) => {
                eprintln!("[Erreur] Impossible d'initialiser le WebView: {}. Ouverture dans le navigateur...", e);
                let _ = opener::open(&app_url);
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
        };

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("[Info] Fermeture de l'application.");
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            }
        });
    }
}
