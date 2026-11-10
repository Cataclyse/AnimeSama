pub mod auth;
pub mod data;
pub mod paths;

#[allow(unused_imports)]
pub use auth::AuthClient;
#[allow(unused_imports)]
pub use data::{HistoryEntry, UserData, UserManager};
#[allow(unused_imports)]
pub use paths::*;

pub use self::data::PlaybackPosition;

pub mod accounts;
