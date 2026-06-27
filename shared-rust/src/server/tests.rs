//! Unit tests for the `server` module.
//!
//! Tests touching process env must serialize (env is process-global).
//! `cargo test` runs tests in parallel threads of the same process, which
//! races `env::set_var` across tests. The `ENV_LOCK` mutex serializes them.

use super::*;
use serial_test::serial;
use std::env;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_clean_env<F: FnOnce()>(vars: &[&str], f: F) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let originals: Vec<Option<String>> = vars.iter().map(|v| env::var(v).ok()).collect();
    for v in vars {
        unsafe { env::remove_var(v) };
    }
    f();
    for (v, original) in vars.iter().zip(originals) {
        match original {
            Some(val) => unsafe { env::set_var(v, val) },
            None => unsafe { env::remove_var(v) },
        }
    }
}

fn with_env<F: FnOnce()>(vars: &[(&str, &str)], f: F) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let originals: Vec<(&str, Option<String>)> =
        vars.iter().map(|(k, _)| (*k, env::var(k).ok())).collect();
    let all_keys = [
        "PORT",
        "SITE_TITLE",
        "BASE_URL",
        "ALLOWED_ORIGINS",
        "BEAM_PIN",
        "BEAM_SITE_TITLE",
        "BEAM_TITLE",
        "PIN",
        "ENABLE_TRANSLATION",
        "ENABLE_THEMES",
        "ENABLE_PRINT",
        "SHOW_VERSION",
        "SHOW_GITHUB",
        "MAX_ATTEMPTS",
        "LOCKOUT_TIME_MINUTES",
        "COOKIE_MAX_AGE_HOURS",
        "TRUST_PROXY",
        "TRUSTED_PROXY_IPS",
    ];
    let originals_all: Vec<(&str, Option<String>)> = all_keys
        .iter()
        .map(|k| (*k, env::var(k).ok()))
        .filter(|(k, _)| !vars.iter().any(|(kk, _)| kk == k))
        .collect();
    for k in all_keys {
        unsafe { env::remove_var(k) };
    }
    for (k, v) in vars {
        unsafe { env::set_var(k, *v) };
    }
    f();
    for k in all_keys {
        unsafe { env::remove_var(k) };
    }
    for (k, val) in originals_all.iter().chain(originals.iter()) {
        if let Some(v) = val {
            unsafe { env::set_var(k, v) };
        }
    }
}

fn minimal_config() -> ServerConfig {
    ServerConfig {
        port: 4401,
        site_title: "X".into(),
        base_url: "http://localhost:4401".into(),
        allowed_origins: "*".into(),
        pin: None,
        enable_translation: false,
        enable_themes: false,
        enable_print: false,
        show_version: true,
        show_github: true,
        trust_proxy: false,
        trusted_proxies: vec![],
        max_attempts: 5,
        lockout_time_minutes: 15,
        cookie_max_age_hours: 24,
    }
}

#[test]
fn defaults_when_no_env_set() {
    with_clean_env(
        &[
            "PORT",
            "SITE_TITLE",
            "BASE_URL",
            "ALLOWED_ORIGINS",
            "BEAM_PIN",
            "ENABLE_TRANSLATION",
            "ENABLE_THEMES",
            "ENABLE_PRINT",
            "MAX_ATTEMPTS",
            "LOCKOUT_TIME_MINUTES",
            "COOKIE_MAX_AGE_HOURS",
            "TRUST_PROXY",
            "TRUSTED_PROXY_IPS",
            "BEAM_SITE_TITLE",
            "BEAM_TITLE",
        ],
        || {
            let cfg = ServerConfig::from_env("BEAM");
            assert_eq!(cfg.port, 4401);
            assert_eq!(cfg.site_title, "BEAM");
            assert_eq!(cfg.base_url, "http://localhost:4401");
            assert_eq!(cfg.allowed_origins, "*");
            assert!(cfg.pin.is_none());
            assert!(!cfg.enable_translation);
            assert!(!cfg.enable_themes);
            assert!(!cfg.enable_print);
            assert!(cfg.show_version);
            assert!(cfg.show_github);
            assert_eq!(cfg.max_attempts, 5);
            assert_eq!(cfg.lockout_time_minutes, 15);
            assert_eq!(cfg.cookie_max_age_hours, 24);
        },
    );
}

#[test]
fn pin_prefix_lookup_order() {
    with_clean_env(
        &[
            "PIN",
            "BEAM_PIN",
            "SITE_TITLE",
            "BEAM_SITE_TITLE",
            "BEAM_TITLE",
        ],
        || {
            unsafe { env::set_var("PIN", "12345678") };
            assert_eq!(
                ServerConfig::from_env("BEAM").pin.as_deref(),
                Some("12345678")
            );

            unsafe { env::set_var("BEAM_PIN", "app_pin_12") };
            assert_eq!(
                ServerConfig::from_env("BEAM").pin.as_deref(),
                Some("app_pin_12"),
                "prefix wins"
            );
        },
    );
}

#[test]
fn pin_rejected_when_too_short() {
    with_clean_env(&["BEAM_PIN"], || {
        unsafe { env::set_var("BEAM_PIN", "abc") };
        assert!(ServerConfig::from_env("BEAM").pin.is_none());
    });
}

#[test]
fn site_title_prefix_lookup_order() {
    with_clean_env(&["SITE_TITLE", "BEAM_SITE_TITLE", "BEAM_TITLE"], || {
        unsafe { env::set_var("SITE_TITLE", "FromGeneric") };
        assert_eq!(ServerConfig::from_env("BEAM").site_title, "FromGeneric");

        unsafe { env::set_var("BEAM_TITLE", "FromTitle") };
        assert_eq!(
            ServerConfig::from_env("BEAM").site_title,
            "FromTitle",
            "_TITLE beats generic"
        );

        unsafe { env::set_var("BEAM_SITE_TITLE", "FromSiteTitle") };
        assert_eq!(
            ServerConfig::from_env("BEAM").site_title,
            "FromSiteTitle",
            "_SITE_TITLE beats _TITLE"
        );
    });
}

#[test]
fn booleans_truthy_values() {
    with_env(
        &[("ENABLE_TRANSLATION", "true"), ("ENABLE_THEMES", "on")],
        || {
            let cfg = ServerConfig::from_env("X");
            assert!(cfg.enable_translation);
            assert!(cfg.enable_themes);
        },
    );
}

#[test]
fn opt_out_booleans_default_true() {
    with_env(&[], || {
        let cfg = ServerConfig::from_env("X");
        assert!(cfg.show_version);
        assert!(cfg.show_github);
    });
    with_env(&[("SHOW_VERSION", "false")], || {
        assert!(!ServerConfig::from_env("X").show_version);
    });
}

#[test]
fn pin_enabled_reflects_config() {
    let mut cfg = minimal_config();
    cfg.pin = Some("12345678".into());
    assert!(cfg.pin_enabled());
    cfg.pin = None;
    assert!(!cfg.pin_enabled());
}

#[test]
fn lockout_duration_scales_with_minutes() {
    let mut cfg = minimal_config();
    cfg.lockout_time_minutes = 30;
    assert_eq!(cfg.lockout_duration().as_secs(), 30 * 60);
}

#[test]
fn parse_trusted_proxies_handles_cidrs() {
    with_clean_env(&["TRUSTED_PROXY_IPS"], || {
        unsafe { env::set_var("TRUSTED_PROXY_IPS", "10.0.0.0/8, 192.168.1.0/24, garbage") };
        let cfg = ServerConfig::from_env("X");
        assert_eq!(cfg.trusted_proxies.len(), 2, "garbage should be skipped");
    });
}

#[test]
fn port_parses_correctly() {
    with_env(&[("PORT", "8080")], || {
        assert_eq!(ServerConfig::from_env("X").port, 8080);
    });
}

#[test]
fn port_falls_back_on_invalid() {
    with_env(&[("PORT", "not_a_number")], || {
        assert_eq!(ServerConfig::from_env("X").port, 4401);
    });
}
