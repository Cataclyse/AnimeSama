mod blocker;
mod bridge;
mod styles;

pub use blocker::get_blocker_script;
pub use bridge::get_bridge_script;
pub use styles::get_shield_styles;

pub fn generate_shield_script(resume_seconds: f64, episode_num: u32) -> String {
    format!(
        r#"<meta name="referrer" content="no-referrer">
{}
<script>
(function() {{
    'use strict';
{}
{}
}})();
</script>"#,
        get_shield_styles(),
        get_blocker_script(),
        get_bridge_script(resume_seconds, episode_num)
    )
}

pub fn generate_resume_only_script(resume_seconds: f64, episode_num: u32) -> String {
    format!(
        r#"<meta name="referrer" content="no-referrer">
<script>
(function() {{
    'use strict';
{}
}})();
</script>"#,
        get_bridge_script(resume_seconds, episode_num)
    )
}
