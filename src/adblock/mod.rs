pub mod domains;
pub mod sanitizer;
pub mod scriptlet;

#[allow(unused_imports)]
pub use domains::{
    get_ad_domains, is_blocked_host, is_blocked_url, is_whitelisted, DUMMY_SCRIPT,
    TRANSPARENT_1X1_GIF, WHITELIST,
};
pub use sanitizer::{inject_resume_only_html, sanitize_embed_html};
#[allow(unused_imports)]
pub use scriptlet::{generate_resume_only_script, generate_shield_script};

#[cfg(test)]
mod tests;
