# UberMetroid Shared Assets

Shared styles, templates, and Rust utility logic for the UberMetroid companion applications (`beam`, `grid`, `pad`, `todo`, `trace`).

---

## Architectural Guidelines

To ensure a cohesive user experience and clean deployment in containerized environments (like Unraid and Cloudflare tunnels), all companion applications must adhere to the following standards:

### 1. Browser Tab & Page Title Standard
* **Frontend Template**: The `index.html` must define the title tag exactly as:
  ```html
  <title>{{SITE_TITLE}}</title>
  ```
* **Backend Substitution**: The backend server must intercept requests for `/` and `/index.html`, dynamically replacing the `{{SITE_TITLE}}` placeholder with the user-configured title before serving the HTML.
* **Dynamic Updates**: The frontend WebAssembly (Yew) application must update the document title using *only* the site title (e.g., `document.set_title(&self.site_title)`). **No prefix or suffix flavor text** (such as board names, active queries, or tool descriptions) should be added to the tab title during normal usage.

### 2. Environment Configurations
All applications should support:
* `SITE_TITLE`: General environment variable to configure the application name.
* `<APP_NAME>_SITE_TITLE`: App-specific override (e.g., `GRID_SITE_TITLE`, `TODO_SITE_TITLE`).

### 3. Shared Styling Assets
The following stylesheet files are maintained in this repository and linked as Trunk resources across all applications:
* `themes.css`: Global design systems, color tokens, and light/dark themes.
* `header.css`: Common navigation and user profile styles.
* `footer.css`: Unified footer layouts.
* `print.css`: Print media optimizations.
* `login.css`: Standardized PIN login screen styles.

### 4. Shared Rust Utility (shared-rust)
A lightweight Rust module containing helper functions and utilities shared across the companion app backends:
* **Crate Dependency**: Declared in backends as `shared-assets` pointing to `shared-rust`.
* **Unauthorized Console Access**: 
  - Function: `shared_assets::print_unauthorized_console_message()`
  - Purpose: Clears the terminal screen and outputs a stylized security warning alert (cowsay format) notifying users that interactive shell access is unauthorized and disabled inside secure, read-only Nix containers. Useful for overriding custom debug or shell binaries in Nix builds.

