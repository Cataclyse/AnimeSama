use super::domains::*;
use super::scriptlet::*;
use regex::Regex;

pub fn sanitize_embed_html(html: &str, _default_host: &str, resume_seconds: f64, episode_num: u32) -> String {
    
    let script_tag_re = Regex::new(r#"(?is)<script[^>]+src=["']([^"']+)["'][^>]*>\s*</script>"#).unwrap();
    let sanitized_scripts = script_tag_re.replace_all(html, |caps: &regex::Captures| {
        let src = &caps[1];
        if is_blocked_url(src) {
            String::new() 
        } else {
            caps[0].to_string()
        }
    });

    
    
    let iframe_tag_re = Regex::new(r#"(?is)<iframe[^>]*>.*?</iframe>"#).unwrap();
    let sanitized_iframes = iframe_tag_re.replace_all(&sanitized_scripts, "");
    let iframe_self_closing_re = Regex::new(r#"(?is)<iframe[^>]*/>"#).unwrap();
    let sanitized_all_iframes = iframe_self_closing_re.replace_all(&sanitized_iframes, "");

    
    let shield = generate_shield_script(resume_seconds, episode_num);
    let mut result = sanitized_all_iframes.into_owned();

    if let Some(pos) = result.to_lowercase().find("<head>") {
        result.insert_str(pos + 6, &format!("\n{}", shield));
    } else {
        result.insert_str(0, &format!("{}\n", shield));
    }

    result
}

pub fn inject_resume_only_html(html: &str, resume_seconds: f64, episode_num: u32) -> String {
    let script = generate_resume_only_script(resume_seconds, episode_num);
    let mut result = html.to_string();
    if let Some(pos) = result.to_lowercase().find("<head>") {
        result.insert_str(pos + 6, &format!("\n{}", script));
    } else {
        result.insert_str(0, &format!("{}\n", script));
    }
    result
}
