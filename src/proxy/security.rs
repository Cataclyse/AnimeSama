use std::net::{IpAddr, SocketAddr};
use url::Url;

// Liste initiale volontairement restrictive.
// Les domaines supplémentaires seront validés par les tests vidéo.
const TRUSTED_HOSTS: &[&str] = &[
    "anime-sama.to",
    "voe.sx",
    "teresapoliticallearn.com",
    "megankiddifference.com",
    "claireguessthousand.com",
    "uqload.to",
    "uqload.com",
    "uqload.io",
    "uqload.ws",
    "uqload.vc",
    "vidmoly.to",
    "vidmoly.me",
    "vidmoly.net",
    "vidmoly.biz",
    "ansembed.net",
    "staticmoly.me",
    "vmpx.online",
    "vmget.online",
    "vmcld.space",
    "sibnet.ru",
    "sendvid.com",
    "earnvids.com",
    "minochinos.com",
    "callistanise.com",
    "streamtape.com",
    "doodstream.com",
    "filemoon.sx",
    "filemoon.to",
];

pub fn validate_proxy_url(input: &str) -> Result<(), &'static str> {
    let parsed = Url::parse(input).map_err(|_| "URL invalide")?;

    if parsed.scheme() != "https" {
        return Err("HTTPS obligatoire");
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("Identifiants dans URL interdits");
    }

    if parsed.port().is_some() {
        return Err("Ports explicites interdits");
    }

    let host = parsed.host_str().ok_or("Hôte manquant")?;
    let host = host.to_ascii_lowercase();

    if host.parse::<IpAddr>().is_ok() {
        return Err("Adresse IP directe interdite");
    }

    if !TRUSTED_HOSTS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{}", domain)))
    {
        return Err("Domaine non autorisé");
    }

    Ok(())
}

pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();

            !(o[0] == 0
                || o[0] == 10
                || o[0] == 127
                || o[0] >= 224
                || (o[0] == 100 && (64..=127).contains(&o[1]))
                || (o[0] == 169 && o[1] == 254)
                || (o[0] == 172 && (16..=31).contains(&o[1]))
                || (o[0] == 192 && o[1] == 168)
                || (o[0] == 192 && o[1] == 0 && o[2] == 0)
                || (o[0] == 192 && o[1] == 0 && o[2] == 2)
                || (o[0] == 192 && o[1] == 88 && o[2] == 99)
                || (o[0] == 198 && (18..=19).contains(&o[1]))
                || (o[0] == 198 && o[1] == 51 && o[2] == 100)
                || (o[0] == 203 && o[1] == 0 && o[2] == 113))
        }

        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_public_ip(IpAddr::V4(v4));
            }

            let s = v6.segments();

            let global_unicast = (s[0] & 0xe000) == 0x2000;

            global_unicast
                && !v6.is_unspecified()
                && !v6.is_loopback()
                && !v6.is_unique_local()
                && !v6.is_unicast_link_local()
                && !v6.is_multicast()
                && !(s[0] == 0x2001 && s[1] == 0x0000)
                && !(s[0] == 0x2001 && s[1] == 0x0db8)
                && !(s[0] == 0x2001 && (s[1] & 0xfff0) == 0x0010)
                && !(s[0] == 0x2001 && (s[1] & 0xfff0) == 0x0020)
                && s[0] != 0x2002
                && s[0] != 0x3ffe
        }
    }
}

pub fn addresses_allowed(addresses: &[SocketAddr]) -> bool {
    !addresses.is_empty() && addresses.iter().all(|addr| is_public_ip(addr.ip()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_trusted_video_hosts() {
        assert!(validate_proxy_url("https://voe.sx/e/123").is_ok());
        assert!(validate_proxy_url(
            "https://claireguessthousand.com/e/test"
        ).is_ok());

        assert!(validate_proxy_url(
            "https://claireguessthousand.com.evil.example/e/test"
        ).is_err());

        assert!(validate_proxy_url(
            "http://claireguessthousand.com/e/test"
        ).is_err());

        assert!(validate_proxy_url(
            "https://megankiddifference.com/embed/test"
        ).is_ok());

        assert!(validate_proxy_url(
            "https://megankiddifference.com.evil.example/embed/test"
        ).is_err());

        assert!(validate_proxy_url(
            "http://megankiddifference.com/embed/test"
        ).is_err());

        assert!(validate_proxy_url(
            "https://callistanise.com/embed/test"
        ).is_ok());

        assert!(validate_proxy_url(
            "https://callistanise.com.evil.example/embed/test"
        ).is_err());

        assert!(validate_proxy_url(
            "http://callistanise.com/embed/test"
        ).is_err());

        assert!(validate_proxy_url(
            "https://minochinos.com/embed/test"
        ).is_ok());

        assert!(validate_proxy_url(
            "https://minochinos.com.evil.example/embed/test"
        ).is_err());

        assert!(validate_proxy_url(
            "http://minochinos.com/embed/test"
        ).is_err());
        assert!(validate_proxy_url(
            "https://teresapoliticallearn.com/e/test"
        ).is_ok());

        assert!(validate_proxy_url(
            "https://teresapoliticallearn.com.evil.example/e/test"
        ).is_err());

        assert!(validate_proxy_url(
            "http://teresapoliticallearn.com/e/test"
        ).is_err());
        assert!(validate_proxy_url("https://uqload.to/embed/123").is_ok());
        assert!(validate_proxy_url("https://cdn.vidmoly.to/video").is_ok());
    }

    #[test]
    fn rejects_untrusted_or_malformed_urls() {
        for url in [
            "http://voe.sx/test",
            "https://127.0.0.1/",
            "https://192.168.1.1/",
            "https://100.121.133.47/",
            "https://169.254.169.254/",
            "https://voe.sx:8443/",
            "https://evil.example/",
            "https://voe.sx.evil.example/",
            "https://user:password@voe.sx/",
            "file:///etc/passwd",
        ] {
            assert!(validate_proxy_url(url).is_err(), "{url}");
        }
    }

    #[test]
    fn rejects_private_and_reserved_addresses() {
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "192.168.1.1",
            "172.20.0.2",
            "100.121.133.47",
            "169.254.169.254",
            "::1",
            "fc00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
        ] {
            assert!(!is_public_ip(ip.parse().unwrap()), "{ip}");
        }
    }

    #[test]
    fn allows_public_addresses() {
        assert!(is_public_ip("1.1.1.1".parse().unwrap()));
        assert!(is_public_ip("2606:4700:4700::1111".parse().unwrap()));
    }

    #[test]
    fn rejects_additional_ipv4_reserved_ranges() {
        for value in [
            "192.88.99.1",
            "192.0.0.9",
            "224.0.0.1",
            "240.0.0.1",
            "255.255.255.255",
        ] {
            let ip = value.parse().unwrap();
            assert!(!is_public_ip(ip), "{value}");
        }
    }

    #[test]
    fn rejects_ipv6_transition_and_reserved_ranges() {
        for value in [
            "2001::1",
            "2001:db8::1",
            "2001:10::1",
            "2001:20::1",
            "2002:c0a8:0101::1",
            "3ffe::1",
            "64:ff9b::c0a8:101",
        ] {
            let ip = value.parse().unwrap();
            assert!(!is_public_ip(ip), "{value}");
        }

        for value in ["2606:4700:4700::1111", "2a06:98c1:3121::2"] {
            let ip = value.parse().unwrap();
            assert!(is_public_ip(ip), "{value}");
        }
    }

    #[test]
    fn rejects_mixed_dns_answers() {
        let addresses = [
            "1.1.1.1:443".parse().unwrap(),
            "127.0.0.1:443".parse().unwrap(),
        ];

        assert!(!addresses_allowed(&addresses));
        assert!(!addresses_allowed(&[]));
    }
}
