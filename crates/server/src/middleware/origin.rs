use std::{
    net::{IpAddr, Ipv4Addr},
    sync::OnceLock,
};

use axum::{
    body::Body,
    extract::Request,
    http::{StatusCode, header},
    response::Response,
};
use url::Url;

#[derive(Clone, Debug, Eq, PartialEq)]
struct OriginKey {
    https: bool,
    host: String,
    port: u16,
}

impl OriginKey {
    fn from_origin(origin: &str) -> Option<Self> {
        let url = Url::parse(origin).ok()?;
        let https = match url.scheme() {
            "http" => false,
            "https" => true,
            _ => return None,
        };
        let host = normalize_host(url.host_str()?);
        let port = url.port_or_known_default()?;
        Some(Self { https, host, port })
    }

    fn from_host_header(host: &str, https: bool) -> Option<Self> {
        let authority: axum::http::uri::Authority = host.parse().ok()?;
        let host = normalize_host(authority.host());
        let port = authority.port_u16().unwrap_or_else(|| default_port(https));
        Some(Self { https, host, port })
    }
}

#[allow(clippy::result_large_err)]
pub fn validate_origin<B>(req: &mut Request<B>) -> Result<(), Response> {
    let Some(origin) = get_origin_header(req) else {
        return Ok(());
    };

    if origin.eq_ignore_ascii_case("null") {
        tracing::debug!("Rejecting request with null origin");
        return Err(forbidden());
    }

    let host = get_host_header(req);

    // quick short-circuit same-origin check
    if host.is_some_and(|host| origin_matches_host(origin, host)) {
        return Ok(());
    }

    let Some(origin_key) = OriginKey::from_origin(origin) else {
        tracing::debug!(origin, "Rejecting request with unparseable origin");
        return Err(forbidden());
    };

    // Allow requests from private/local network origins.
    // This app is deployed on private LANs (e.g., EC2 instances accessed via
    // private IPs like 10.x.x.x). The origin and host headers may not match
    // exactly when accessed via different network interfaces.
    if is_private_or_local_host(&origin_key.host) {
        return Ok(());
    }

    // Allow requests whose Host header resolves to a private/local address.
    // When behind a reverse proxy (nginx, Cloudflare tunnel, ngrok, etc.) the
    // proxy forwards requests to the backend on localhost or a private IP.
    // The browser's Origin header will carry the public domain while the Host
    // header (as seen by the backend) may be localhost or a private address.
    // Since this app is a local deployment tool (not a public SaaS), trusting
    // requests that arrive on a private interface is safe.
    if let Some(host_val) = host {
        let host_name = host_val.rsplit_once(':').map_or(host_val, |(h, _)| h);
        if is_private_or_local_host(&normalize_host(host_name)) {
            return Ok(());
        }
    }

    if allowed_origins()
        .iter()
        .any(|allowed| allowed == &origin_key)
    {
        return Ok(());
    }

    if let Some(host_key) =
        host.and_then(|host| OriginKey::from_host_header(host, origin_key.https))
        && host_key == origin_key
    {
        return Ok(());
    }

    tracing::warn!(
        origin,
        host = host.unwrap_or("<missing>"),
        "Rejecting cross-origin request"
    );
    Err(forbidden())
}

fn get_origin_header<B>(req: &Request<B>) -> Option<&str> {
    get_header(req, header::ORIGIN)
}

fn get_host_header<B>(req: &Request<B>) -> Option<&str> {
    get_header(req, header::HOST)
}

fn get_header<B>(req: &Request<B>, name: header::HeaderName) -> Option<&str> {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
}

fn forbidden() -> Response {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .body(Body::empty())
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn origin_matches_host(origin: &str, host: &str) -> bool {
    let origin_authority = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"));
    let Some(rest) = origin_authority else {
        return false;
    };
    // Exact match (host header includes port)
    if rest.eq_ignore_ascii_case(host) {
        return true;
    }
    // Handle reverse proxies (e.g. nginx $host) that strip the port from
    // the Host header. In that case "10.0.1.242:3000" (origin) won't match
    // "10.0.1.242" (host). Compare just the hostname portion.
    let origin_host = rest.rsplit_once(':').map_or(rest, |(h, _)| h);
    origin_host.eq_ignore_ascii_case(host)
}

fn normalize_host(host: &str) -> String {
    let trimmed = host.trim().trim_start_matches('[').trim_end_matches(']');
    let lower = trimmed.to_ascii_lowercase();
    if lower == "localhost" {
        return "localhost".to_string();
    }
    if let Ok(ip) = lower.parse::<IpAddr>() {
        if ip.is_loopback() {
            return "localhost".to_string();
        }
        return ip.to_string();
    }
    lower
}

/// Returns true if the host is localhost or a private/link-local network address.
/// Private networks (RFC 1918): 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
/// CGNAT / Tailscale (RFC 6598): 100.64.0.0/10
/// Link-local: 169.254.0.0/16, fe80::/10
fn is_private_or_local_host(host: &str) -> bool {
    if host == "localhost" {
        return true;
    }
    let Ok(ip) = host.parse::<IpAddr>() else {
        return false;
    };
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local() || is_cgnat(v4),
        IpAddr::V6(v6) => {
            v6.is_loopback()
                // fe80::/10 (link-local)
                || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

/// Returns true if the address is in the CGNAT range 100.64.0.0/10 (RFC 6598).
/// This range is used by Tailscale (100.x.x.x) and carrier-grade NAT deployments.
fn is_cgnat(addr: Ipv4Addr) -> bool {
    let octets = addr.octets();
    octets[0] == 100 && (octets[1] & 0xC0) == 64
}

fn default_port(https: bool) -> u16 {
    if https { 443 } else { 80 }
}

fn allowed_origins() -> &'static Vec<OriginKey> {
    static ALLOWED: OnceLock<Vec<OriginKey>> = OnceLock::new();
    ALLOWED.get_or_init(|| {
        let value = match std::env::var("VK_ALLOWED_ORIGINS") {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };

        value
            .split(',')
            .filter_map(|origin| OriginKey::from_origin(origin.trim()))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use axum::http::{Request, header};

    use super::*;

    fn make_request(origin: Option<&str>, host: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().uri("/test").method("GET");
        if let Some(origin) = origin {
            builder = builder.header(header::ORIGIN, origin);
        }
        if let Some(host) = host {
            builder = builder.header(header::HOST, host);
        }
        builder.body(Body::empty()).unwrap()
    }

    fn is_forbidden(result: Result<(), Response>) -> bool {
        matches!(result, Err(resp) if resp.status() == StatusCode::FORBIDDEN)
    }

    #[test]
    fn no_origin_header_allows_request() {
        let mut req = make_request(None, Some("example.com"));
        assert!(validate_origin(&mut req).is_ok());
    }

    #[test]
    fn null_origin_is_forbidden() {
        for null in ["null", "NULL", "Null"] {
            let mut req = make_request(Some(null), Some("example.com"));
            assert!(is_forbidden(validate_origin(&mut req)));
        }
    }

    #[test]
    fn same_origin_allows_request() {
        // HTTP, HTTPS, with port, case-insensitive
        let cases = [
            ("http://example.com", "example.com"),
            ("https://example.com", "example.com"),
            ("http://example.com:8080", "example.com:8080"),
            ("http://EXAMPLE.COM", "example.com"),
        ];
        for (origin, host) in cases {
            let mut req = make_request(Some(origin), Some(host));
            assert!(validate_origin(&mut req).is_ok(), "{origin} vs {host}");
        }
    }

    #[test]
    fn proxy_stripped_port_allows_request() {
        // Reverse proxies like nginx with `proxy_set_header Host $host` strip
        // the port from the Host header. The origin still has the port.
        let cases = [
            ("http://10.0.1.242:3000", "10.0.1.242"),
            ("http://example.com:8080", "example.com"),
            ("https://example.com:443", "example.com"),
        ];
        for (origin, host) in cases {
            let mut req = make_request(Some(origin), Some(host));
            assert!(
                validate_origin(&mut req).is_ok(),
                "{origin} vs {host} should be allowed (proxy port-strip)"
            );
        }
    }

    #[test]
    fn cross_origin_forbidden() {
        let cases = [
            ("http://unknown.com", "example.com"),         // different host
            ("http://example.com:8080", "example.com:80"), // different port
            ("ftp://example.com", "example.com"),          // non-http scheme
            ("not-a-valid-url", "example.com"),            // invalid URL
            ("http://example.com", ""),                    // missing host (invalid)
        ];
        for (origin, host) in cases {
            let host_opt = if host.is_empty() { None } else { Some(host) };
            let mut req = make_request(Some(origin), host_opt);
            assert!(is_forbidden(validate_origin(&mut req)), "{origin}");
        }
    }

    #[test]
    fn loopback_addresses_normalized_and_equivalent() {
        // All loopback forms normalize to "localhost"
        assert_eq!(
            OriginKey::from_origin("http://localhost:3000")
                .unwrap()
                .host,
            "localhost"
        );
        assert_eq!(
            OriginKey::from_origin("http://127.0.0.1:3000")
                .unwrap()
                .host,
            "localhost"
        );
        assert_eq!(
            OriginKey::from_origin("http://[::1]:3000").unwrap().host,
            "localhost"
        );

        // Cross-loopback requests should be allowed
        let mut req = make_request(Some("http://127.0.0.1:3000"), Some("[::1]:3000"));
        assert!(validate_origin(&mut req).is_ok());
    }

    #[test]
    fn default_ports_handled_correctly() {
        assert_eq!(
            OriginKey::from_origin("http://example.com").unwrap().port,
            80
        );
        assert_eq!(
            OriginKey::from_origin("https://example.com").unwrap().port,
            443
        );

        // Explicit default port matches implicit
        let mut req = make_request(Some("http://example.com:80"), Some("example.com"));
        assert!(validate_origin(&mut req).is_ok());
    }

    #[test]
    fn private_network_origins_allowed() {
        // RFC 1918 private addresses should be allowed even if Host header differs
        let private_origins = [
            ("http://10.0.1.242:3000", "some-other-host:3000"),
            ("http://10.0.0.1:3000", "localhost:3001"),
            ("http://172.16.0.1:8080", "different-host:8080"),
            ("http://172.31.255.255:3000", "whatever:3000"),
            ("http://192.168.1.1:3000", "other:3000"),
            ("http://192.168.0.100:5000", "host:5000"),
        ];
        for (origin, host) in private_origins {
            let mut req = make_request(Some(origin), Some(host));
            assert!(
                validate_origin(&mut req).is_ok(),
                "expected {origin} to be allowed"
            );
        }
    }

    #[test]
    fn tailscale_cgnat_origins_allowed() {
        // Tailscale uses 100.64.0.0/10 (CGNAT / RFC 6598)
        let tailscale_origins = [
            ("http://100.64.0.1:3000", "some-host:3000"),
            ("http://100.100.100.100:3000", "other-host:3000"),
            ("http://100.127.255.255:3000", "whatever:3000"),
        ];
        for (origin, host) in tailscale_origins {
            let mut req = make_request(Some(origin), Some(host));
            assert!(
                validate_origin(&mut req).is_ok(),
                "expected Tailscale/CGNAT {origin} to be allowed"
            );
        }

        // 100.128.0.0 is outside the /10 range — should be forbidden
        let mut req = make_request(Some("http://100.128.0.1:3000"), Some("other-host:3000"));
        assert!(
            is_forbidden(validate_origin(&mut req)),
            "100.128.x.x is outside CGNAT range"
        );
    }

    #[test]
    fn cgnat_range_boundary_check() {
        // Verify the CGNAT range boundaries (100.64.0.0 - 100.127.255.255)
        assert!(is_cgnat(Ipv4Addr::new(100, 64, 0, 0)));
        assert!(is_cgnat(Ipv4Addr::new(100, 127, 255, 255)));
        assert!(!is_cgnat(Ipv4Addr::new(100, 63, 255, 255)));
        assert!(!is_cgnat(Ipv4Addr::new(100, 128, 0, 0)));
        assert!(!is_cgnat(Ipv4Addr::new(99, 64, 0, 0)));
        assert!(!is_cgnat(Ipv4Addr::new(101, 64, 0, 0)));
    }

    #[test]
    fn public_ip_cross_origin_still_forbidden() {
        // Public IPs should still be checked normally
        let mut req = make_request(Some("http://8.8.8.8:3000"), Some("other-host:3000"));
        assert!(is_forbidden(validate_origin(&mut req)));
    }

    #[test]
    fn public_origin_with_private_host_allowed() {
        // When behind a reverse proxy (nginx, Cloudflare tunnel, ngrok), the
        // browser Origin header carries the public domain while the backend
        // sees the proxy's internal Host header (localhost or private IP).
        let cases = [
            // Cloudflare tunnel / ngrok → forwarded to localhost
            ("https://my-app.example.com", "localhost:3000"),
            ("https://my-app.example.com", "127.0.0.1:3000"),
            ("https://my-app.example.com", "[::1]:3000"),
            // nginx proxy → forwarded to private IP
            ("https://my-app.example.com", "10.0.1.242:3000"),
            ("https://my-app.example.com", "192.168.1.100:3000"),
            ("https://my-app.example.com", "172.16.0.1:8080"),
            // Host without port (nginx $host strips port)
            ("https://my-app.example.com", "localhost"),
            ("https://my-app.example.com", "127.0.0.1"),
            ("https://my-app.example.com", "10.0.1.242"),
            // Tailscale host
            ("https://my-app.example.com", "100.100.100.100:3000"),
        ];
        for (origin, host) in cases {
            let mut req = make_request(Some(origin), Some(host));
            assert!(
                validate_origin(&mut req).is_ok(),
                "expected {origin} with host {host} to be allowed (proxy to private backend)"
            );
        }
    }

    #[test]
    fn public_origin_with_public_host_still_forbidden() {
        // If both origin and host are public and don't match, reject
        let mut req = make_request(Some("https://evil.com"), Some("my-app.example.com:3000"));
        assert!(is_forbidden(validate_origin(&mut req)));
    }
}
