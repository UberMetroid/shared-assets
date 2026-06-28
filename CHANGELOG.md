# Changelog

All notable changes to `shared-assets` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.0.0] - 2026-06-26

### ⚠ BREAKING CHANGES

This release adds shared backend primitives. Consumers must update their
`Cargo.toml` to use the v3.0.0 tag and update field references as described
below.

#### Migration guide

**Before (2.x):**
```rust
use shared_assets::print_unauthorized_console_message;
use shared_assets::header::Header;

shared_assets::print_unauthorized_console_message();
```

**After (3.x):**
```rust
use shared_backend::security::print_unauthorized_console_message;
use shared_frontend::components::Header;

shared_backend::security::print_unauthorized_console_message();
```

**Cargo dependency:**
```toml
# Before (2.x):
shared-assets = { path = "...", features = ["frontend"] }

# After (3.x): same syntax, but pin to the v3.0.0 tag:
shared-assets = { git = "https://github.com/UberMetroid/shared-assets", tag = "v3.0.0" }
# Or for local development:
shared-assets = { path = "..." }
```

#### New modules

- **`server`** — Backend server primitives
  - `ServerConfig` — common env-driven config (port, pin, attempts, cookie age, CORS, enable_*, show_*, trust_proxy)
  - `server::serve` — bind + graceful shutdown on SIGINT/SIGTERM
  - `server::ServerError` — `IntoResponse` error type with HTTP status mapping
  - `server::ip::get_client_ip` — trusted-proxy-aware client IP extraction
  - `server::version::CARGO_PKG_VERSION` — re-export of the consuming crate's version

- **`auth`** — PIN authentication
  - `auth::pin_auth_layer` — axum middleware that gates routes behind a PIN
  - `auth::attempts::{is_locked_out, record_attempt, reset_attempts, lockout_remaining_secs}`
  - `auth::session::issue_cookie` — session cookie helpers

- **`middleware`** — Shared axum middleware factories
  - `middleware::cors_layer` — CORS layer from `ALLOWED_ORIGINS`
  - `middleware::security_headers_layer` — CSP, X-Frame-Options, etc.
  - `middleware::title_injection_layer` — `{{SITE_TITLE}}` → config
  - `middleware::hsts_layer` — HSTS when HTTPS

#### Removed

- Top-level `print_unauthorized_console_message` re-export. Use
  `shared_assets::security::print_unauthorized_console_message` instead.

#### Changed

- Bumped edition 2021 → 2024 (let-chains used throughout)
- `web-sys` pinned to `=0.3.98` (matches the Yew 0.23 expected version)
- `ipnet`, `tokio`, `tower-http`, `axum`, `thiserror`, `anyhow`, `dotenvy`,
  `constant_time_eq`, `tracing`, `http-body-util` are now direct dependencies
  of the shared-assets crate (consumers don't need to declare them just to use
  the new shared modules)

### Added

- 62 unit tests + 1 doctest (was 22)
- `cargo clippy` clean, `cargo fmt` clean
- 27 `.rs` files all ≤ 250 lines

## [2.1.1] - 2026-06-25

Last 2.x release. Provides Yew components, theme management, and i18n only.

- `components::Header`, `components::Footer` — Yew UI chrome
- `theme::Theme` — Super Metroid theme enum (Crateria, Brinstar, Norfair, WreckedShip, Maridia, Tourian)
- `theme::mapping::Scheme` — User-facing scheme (light/sepia/dracula/nord) → Theme mapping
- `i18n::Language` — 8-language enum (en/zh/es/de/ja/fr/pt/ru)
- `i18n::strings::lookup` — Centralized UI string lookup
- `security::print_unauthorized_console_message` — anti-shell alert (also re-exported at crate root for 2.x compat)