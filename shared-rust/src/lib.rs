//! Shared code for UberMetroid companion apps.
//!
//! This crate is consumed by both the apps' native server binaries
//! (which use the `backend` feature) and their Yew/WASM frontends
//! (which use the `frontend` feature). See the `Cargo.toml` for the
//! feature list.
//!
//! Provides:
//!
//! - Yew components (frontend): [`components::Header`], [`components::Footer`]
//! - Theme management (frontend): [`theme::Theme`], [`theme::mapping::Scheme`]
//! - Internationalization (both): [`i18n::Language`], [`i18n::strings::lookup`]
//! - Backend primitives (backend): [`server::ServerConfig`], [`server::serve`]
//! - Authentication (backend): [`auth::pin_auth_layer`], [`auth::attempts`]
//! - Shared middleware (backend): [`middleware::cors_layer`],
//!   [`middleware::security_headers_layer`],
//!   [`middleware::title_injection_layer`], [`middleware::hsts_layer`]
//! - Security helpers (both): [`security::print_unauthorized_console_message`]
//!
//! ## Cargo dependency
//!
//! ```toml
//! # backend/Cargo.toml
//! shared-assets = { path = "...", default-features = false, features = ["backend"] }
//!
//! # frontend/Cargo.toml
//! shared-assets = { path = "...", default-features = false, features = ["frontend"] }
//! ```

#[cfg(feature = "backend")]
pub mod auth;
#[cfg(feature = "backend")]
pub mod middleware;
#[cfg(feature = "backend")]
pub mod server;

// Used by both backend and frontend.
pub mod i18n;
#[cfg(feature = "backend")]
pub mod security;

#[cfg(feature = "frontend")]
pub mod components;
#[cfg(feature = "frontend")]
pub mod theme;

// Re-exports for ergonomics.
#[cfg(feature = "frontend")]
pub use components::{footer, footer::Footer, header, header::Header};
