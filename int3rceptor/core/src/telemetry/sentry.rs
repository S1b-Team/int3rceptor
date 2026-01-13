use sentry::types::Dsn;
use sentry::{ClientInitGuard, ClientOptions, Level};
use std::borrow::Cow;
use std::sync::Arc;

const DEFAULT_TIER: u8 = 0;
const MAX_TRACING_RATE: f32 = 0.05;

pub fn init_sentry() -> Option<ClientInitGuard> {
    let dsn = std::env::var("SENTRY_DSN").ok()?;
    let dsn = dsn.trim();
    if dsn.is_empty() {
        return None;
    }

    let parsed_dsn: Dsn = match dsn.parse() {
        Ok(dsn) => dsn,
        Err(_) => return None,
    };

    let environment =
        std::env::var("SENTRY_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    let release = std::env::var("SENTRY_RELEASE")
        .ok()
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| {
            if let Some(sha) = option_env!("GIT_SHA") {
                format!("int3rceptor-core@{}+{}", env!("CARGO_PKG_VERSION"), sha)
            } else {
                format!("int3rceptor-core@{}", env!("CARGO_PKG_VERSION"))
            }
        });

    let tier = telemetry_tier();
    let traces_sample_rate = if tier == 2 {
        clamp_rate(
            std::env::var("SENTRY_TRACES_SAMPLE_RATE")
                .ok()
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(MAX_TRACING_RATE),
        )
    } else {
        0.0
    };

    let options = ClientOptions {
        release: Some(Cow::Owned(release)),
        environment: Some(Cow::Owned(environment)),
        send_default_pii: false,
        sample_rate: 1.0,
        traces_sample_rate,
        before_send: Some(Arc::new(|mut event| {
            if is_noisy_network(&event) {
                return None;
            }

            if let Some(request) = event.request.as_mut() {
                request.url = None;
                request.headers.clear();
                request.cookies = None;
                request.data = None;
            }

            event.extra.retain(|key, _| {
                !matches!(
                    key.as_str(),
                    "request"
                        | "payload"
                        | "body"
                        | "headers"
                        | "cookies"
                        | "authorization"
                        | "token"
                        | "url"
                )
            });

            event.tags.retain(|key, _| {
                !matches!(
                    key.as_str(),
                    "authorization"
                        | "cookie"
                        | "set-cookie"
                        | "x-api-key"
                        | "token"
                        | "url"
                        | "payload"
                )
            });

            Some(event)
        })),
        ..Default::default()
    };

    let guard = sentry::init((parsed_dsn, options));

    sentry::configure_scope(|scope| {
        scope.set_tag("component", "core");
        scope.set_tag("tier", tier.to_string());
        scope.set_tag("os", std::env::consts::OS);
        scope.set_tag("arch", std::env::consts::ARCH);
    });

    if std::env::var("SENTRY_SMOKE_TEST")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        sentry::capture_message("Int3rceptor core smoke test", Level::Info);
    }

    Some(guard)
}

/// Capture an anyhow error with safe tagging and noise filtering.
pub fn capture_anyhow(err: &anyhow::Error, module: &str, tags: &[(&str, &str)]) {
    if sentry::Hub::current().client().is_none() || is_noisy_io(err) {
        return;
    }

    let tier = telemetry_tier();
    sentry::configure_scope(|scope| {
        scope.set_tag("component", "core");
        scope.set_tag("module", module);
        scope.set_tag("tier", tier.to_string());
        scope.set_tag("os", std::env::consts::OS);
        scope.set_tag("arch", std::env::consts::ARCH);

        for (k, v) in tags {
            if is_safe_tag_key(k) {
                scope.set_tag(*k, *v);
            }
        }
    });

    sentry::integrations::anyhow::capture_anyhow(err);
}

fn is_noisy_network(event: &sentry::protocol::Event) -> bool {
    let mut texts: Vec<String> = Vec::new();

    if let Some(msg) = event.message.as_ref() {
        texts.push(msg.to_string());
    }

    let mut networkish_type = false;
    if !event.exception.values.is_empty() {
        for ex in &event.exception.values {
            let ty_lower = ex.ty.to_lowercase();
            if ty_lower.contains("io::error")
                || ty_lower.contains("std::io::error")
                || ty_lower.contains("reqwest")
                || ty_lower.contains("hyper")
                || ty_lower.contains("h2")
            {
                networkish_type = true;
            }

            texts.push(ex.ty.clone());
            if let Some(value) = ex.value.as_deref() {
                texts.push(value.to_string());
            }
        }
    }

    let has_network_phrase = texts.iter().any(|text| is_network_phrase(text));
    has_network_phrase && networkish_type
}

fn is_network_phrase(text: &str) -> bool {
    let lower = text.to_lowercase();
    const PATTERNS: [&str; 10] = [
        "timed out",
        "timeout",
        "connection reset",
        "broken pipe",
        "connection refused",
        "connection aborted",
        "network error",
        "dns error",
        "no route to host",
        "host unreachable",
    ];

    PATTERNS.iter().any(|pat| lower.contains(pat))
}

fn is_noisy_io(err: &anyhow::Error) -> bool {
    use std::io::ErrorKind;

    for cause in err.chain() {
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            match io.kind() {
                ErrorKind::TimedOut
                | ErrorKind::WouldBlock
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::ConnectionRefused
                | ErrorKind::BrokenPipe
                | ErrorKind::NotConnected
                | ErrorKind::AddrNotAvailable
                | ErrorKind::HostUnreachable
                | ErrorKind::NetworkUnreachable => return true,
                _ => {}
            }
        }
    }

    false
}

fn telemetry_tier() -> u8 {
    std::env::var("SENTRY_TELEMETRY_TIER")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .filter(|v| *v <= 2)
        .unwrap_or(DEFAULT_TIER)
}

fn clamp_rate(rate: f32) -> f32 {
    if rate < 0.0 {
        0.0
    } else if rate > MAX_TRACING_RATE {
        MAX_TRACING_RATE
    } else {
        rate
    }
}

fn is_safe_tag_key(key: &str) -> bool {
    !matches!(
        key.to_ascii_lowercase().as_str(),
        "authorization"
            | "cookie"
            | "set-cookie"
            | "token"
            | "x-api-key"
            | "bearer"
            | "payload"
            | "body"
            | "url"
            | "headers"
    )
}
