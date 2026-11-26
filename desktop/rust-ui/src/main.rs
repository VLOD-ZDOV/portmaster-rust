//! Main entrypoint for Portmaster Native Leptos CSR.

use leptos::prelude::*;

mod app;
mod client;
mod components;
mod i18n;
mod state;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App/> });
}
