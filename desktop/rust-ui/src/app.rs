//! Root application view for Portmaster Native Leptos CSR.
//! Provides a 1:1 recreation of the official Portmaster desktop interface.

use leptos::prelude::*;

use crate::client::PortApiClient;
use crate::components::{AppSwitcher, Header, MonitorPanel, SettingsPanel, Sidebar, XrayPanel};
use crate::state::{AppState, ViewTab};

#[component]
pub fn App() -> impl IntoView {
    // Initialize PortAPI client and app state
    let client = PortApiClient::new(None);
    let state = AppState::new(client);
    let active_view = state.active_view;

    provide_context(state);

    view! {
        <div class="h-screen w-screen flex bg-[#121213] text-[#ffffff] overflow-hidden font-sans select-none selection:bg-[#4e97fa] selection:text-white">
            // Left Portmaster Navigation Sidebar
            <Sidebar/>

            // Right Main Application Shell
            <div class="flex-1 flex flex-col min-w-0 h-full overflow-hidden">
                // Top Header Bar
                <Header/>

                // Scrollable Content Pane
                <main class="flex-1 overflow-y-auto px-6 py-5 select-text">
                    <div class="max-w-6xl w-full mx-auto">
                        {move || match active_view.get() {
                            ViewTab::Apps => view! { <AppSwitcher/> }.into_any(),
                            ViewTab::Vpn => view! { <XrayPanel/> }.into_any(),
                            ViewTab::Settings => view! { <SettingsPanel/> }.into_any(),
                            ViewTab::Monitor => view! { <MonitorPanel/> }.into_any(),
                        }}
                    </div>
                </main>
            </div>
        </div>
    }
}
