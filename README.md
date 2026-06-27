# UberMetroid Shared Assets

Shared styles, Rust components, and backend helpers for the UberMetroid companion
applications (`beam`, `grid`, `pad`, `todo`, `trace`).

---

## Repository Layout

```
shared-assets/
├── styles/                       Shared CSS, organized by concern
│   ├── themes/
│   │   └── themes.css            Super Metroid color tokens & themes
│   ├── layout/
│   │   ├── header.css            Top navigation bar
│   │   └── footer.css            Bottom footer
│   ├── components/
│   │   └── body.css              Page body & containers
│   └── pages/
│       ├── login.css             PIN entry screen
│       └── print.css             Print media rules
└── shared-rust/                  Rust crate (path: `shared-rust/`)
    ├── Cargo.toml
    ├── rust-toolchain.toml       Pinned to 1.96.0
    ├── rustfmt.toml              100-col, reorder imports
    ├── clippy.toml               Moderate strictness
    ├── deny.toml                 Supply-chain license policy
    └── src/
        ├── lib.rs                Entry point + public re-exports
        ├── components/           Yew UI chrome
        │   ├── mod.rs
        │   ├── header.rs         Top bar (theme/lang/print/logout)
        │   └── footer.rs         Bottom bar (version/github/children)
        ├── theme/                Super Metroid theme management
        │   ├── mod.rs            Theme enum + name/from_name
        │   ├── icons.rs          SVG icons per theme
        │   └── mapping.rs        Scheme (light/sepia/dracula/nord) → Theme
        ├── i18n/                 Internationalization
        │   ├── mod.rs            Language enum
        │   └── strings.rs        Centralized UI string lookup
        └── security/             Backend utilities
            └── mod.rs            print_unauthorized_console_message
```

---

## Architectural Guidelines

To ensure a cohesive user experience and clean deployment in containerized
environments (like Unraid and Cloudflare tunnels), all companion applications
must adhere to the following standards.

### 1. Browser Tab & Page Title Standard
* **Frontend Template**: The `index.html` must define the title tag exactly as:
  ```html
  <title>{{SITE_TITLE}}</title>
  ```
* **Backend Substitution**: The backend server must intercept requests for `/`
  and `/index.html`, dynamically replacing the `{{SITE_TITLE}}` placeholder with
  the user-configured title before serving the HTML.
* **Dynamic Updates**: The frontend WebAssembly (Yew) application must update
  the document title using *only* the site title (e.g.,
  `document.set_title(&self.site_title)`). **No prefix or suffix flavor text**
  (such as board names, active queries, or tool descriptions) should be added
  to the tab title during normal usage.

### 2. Environment Configurations

All applications should support:
* `SITE_TITLE`: General environment variable to configure the application name.
* `<APP_NAME>_SITE_TITLE`: App-specific override (e.g., `GRID_SITE_TITLE`,
  `TODO_SITE_TITLE`).

### 3. Shared Styling Assets

Stylesheets are organized by concern under `styles/`. Each app wires them via
Trunk's `<link data-trunk rel="css" ...>` in `frontend/index.html`:

```html
<link data-trunk rel="css" href="Assets/shared-assets/styles/themes/themes.css" />
<link data-trunk rel="css" href="Assets/shared-assets/styles/layout/header.css" />
<link data-trunk rel="css" href="Assets/shared-assets/styles/layout/footer.css" />
<link data-trunk rel="css" href="Assets/shared-assets/styles/components/body.css" />
<link data-trunk rel="css" href="Assets/shared-assets/styles/pages/login.css" />
<link data-trunk rel="css" href="Assets/shared-assets/styles/pages/print.css" />
```

The CSS file purposes:
* `styles/themes/themes.css` — design tokens and 5 Super Metroid themes
  (`crateria`, `brinstar`, `norfair`, `wrecked_ship`, `maridia`, `tourian`).
  Names are referenced by [`crate::theme::Theme`].
* `styles/layout/header.css` — top navigation bar (used with
  [`crate::components::header::Header`]).
* `styles/layout/footer.css` — bottom footer (used with
  [`crate::components::footer::Footer`]).
* `styles/components/body.css` — page body, containers, common components.
* `styles/pages/login.css` — PIN entry screen.
* `styles/pages/print.css` — print media rules.

### 4. Shared Rust Crate (`shared-rust`)

A small Rust crate consumed by every companion app's backend and frontend.

#### Cargo Dependency

**Backend:**
```toml
# backend/Cargo.toml
shared-assets = { path = "../frontend/Assets/shared-assets/shared-rust" }
```

**Frontend:**
```toml
# frontend/Cargo.toml
shared-assets = { path = "Assets/shared-assets/shared-rust" }
```

#### Public API

| Module | Purpose | When to use |
| :--- | :--- | :--- |
| [`components::Header`] | Top navigation bar Yew component | Always |
| [`components::Footer`] | Bottom footer Yew component | Always |
| [`theme::Theme`] | Super Metroid theme enum (replaces string literals) | When storing/loading the active theme |
| [`theme::mapping::Scheme`] | User-facing scheme name → `Theme` mapping | When rendering the theme picker |
| [`i18n::Language`] | Supported UI language enum | When storing/loading the active language |
| [`i18n::strings::lookup`] | Centralized UI string translation | When displaying any translated text |
| [`security::print_unauthorized_console_message`] | Anti-shell alert | In `src/bin/sh.rs` stub only |

#### Feature Flags

```toml
[features]
default = ["frontend"]   # Pulls in Yew + web-sys + serde
```

Backend consumers that don't need the frontend stack can disable defaults:
```toml
shared-assets = { path = "...", default-features = false }
```

#### Example: Adding a New Translated String

1. Add a variant to `i18n::strings::StringKey`.
2. Add translations for every language in the `lookup` match.
3. Call `lookup(StringKey::YourNewKey, language)` from the component.

#### Example: Switching a Component to Use the Theme Enum

Replace string literals like `"brinstar"` with:
```rust
use shared_assets::theme::Theme;
Theme::Brinstar.name()  // returns "brinstar" for CSS / localStorage
```

---

## Development

### Build & Test

```bash
cd shared-rust
cargo build
cargo test         # 22 unit tests + 1 doctest
cargo clippy       # 0 warnings
cargo fmt --check
```

### Coding Standards

* Files limited to 250 lines
* Subdirectories for each major concern
* 100-column line width (`rustfmt.toml`)
* All public API documented with `///` doc comments
* Defensive parsing (`Option`/`Result` over panic)
* Tests for round-trip parsing, uniqueness, and coverage