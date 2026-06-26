use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FooterProps {
    #[prop_or_default]
    pub show_version: bool,
    #[prop_or_default]
    pub version: String,
    #[prop_or_default]
    pub show_github: bool,
    #[prop_or_default]
    pub github_url: Option<String>,
    
    #[prop_or_default]
    pub children: Html,
}

#[function_component(Footer)]
pub fn footer(props: &FooterProps) -> Html {
    let github_link = props.github_url.clone().unwrap_or_else(|| "https://github.com/UberMetroid".to_string());
    
    html! {
        <footer class="layout-footer">
            <div class="footer-left">
                {if props.show_version {
                    html! {
                        <span class="footer-version">{format!("v{}", props.version)}</span>
                    }
                } else {
                    html! {}
                }}
            </div>
            
            <div class="footer-center">
                {props.children.clone()}
            </div>
            
            <div class="footer-right">
                {if props.show_github {
                    html! {
                        <a class="footer-github-link" href={github_link} target="_blank" rel="noopener noreferrer" aria-label="GitHub Profile">
                            <svg class="github-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
                                <path d="M9 18c-4.51 2-5-2-7-2" />
                            </svg>
                            <span>{"GitHub"}</span>
                        </a>
                    }
                } else {
                    html! {}
                }}
            </div>
        </footer>
    }
}
