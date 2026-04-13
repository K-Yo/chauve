use crate::entities::settings::{ProviderGrafanaSetting, Settings};
use crate::poller::Poller;
use crate::settings::settings_path;
use dioxus::prelude::*;

#[component]
pub fn SettingsView() -> Element {
    let mut settings_signal: Signal<Settings> = use_context();
    let mut poller_signal: Signal<Poller> = use_context();

    // Local draft — edits don't apply until Save is clicked.
    let mut draft: Signal<Settings> = use_signal(|| settings_signal.read().clone());

    rsx! {
        div { class: "overflow-y-scroll pt-12 p-4",
            // Poll frequency
            div { class: "mb-4",
                label { class: "block mb-1 text-sm", "Poll frequency (seconds)" }
                input {
                    r#type: "number",
                    class: "border px-2 py-1 w-24",
                    min: "1",
                    value: "{draft.read().poll_frequency}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<u64>() {
                            draft.write().poll_frequency = v;
                        }
                    },
                }
            }

            // Grafana providers
            div { class: "mb-6",
                h3 { class: "font-bold mb-2", "Grafana providers" }
                for (i , provider) in draft.read().providers.grafana.clone().into_iter().enumerate() {
                    div { key: "{i}", class: "flex gap-2 mb-2",
                        input {
                            class: "border px-2 py-1 flex-1",
                            placeholder: "URL",
                            value: "{provider.url}",
                            oninput: move |e| {
                                draft.write().providers.grafana[i].url = e.value();
                            },
                        }
                        input {
                            class: "border px-2 py-1 flex-1",
                            placeholder: "Token",
                            value: "{provider.token}",
                            oninput: move |e| {
                                draft.write().providers.grafana[i].token = e.value();
                            },
                        }
                        button {
                            class: "px-2 py-1 text-red-600",
                            onclick: move |_| {
                                draft.write().providers.grafana.remove(i);
                            },
                            "✕"
                        }
                    }
                }
                button {
                    class: "text-sm mt-1",
                    onclick: move |_| {
                        draft.write().providers.grafana.push(ProviderGrafanaSetting {
                            url: String::new(),
                            token: String::new(),
                        });
                    },
                    "+ Add Grafana"
                }
            }

            button {
                class: "bg-orange-500 hover:bg-orange-700 text-black py-2 px-4 rounded",
                onclick: move |_| {
                    let new_settings = draft.read().clone();
                    match toml::to_string(&new_settings) {
                        Ok(toml_str) => {
                            let path = settings_path();
                            if let Err(e) = std::fs::write(&path, &toml_str) {
                                tracing::error!("Failed to write settings to {}: {}", path.display(), e);
                                return;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to serialize settings: {}", e);
                            return;
                        }
                    }
                    // Rebuild the poller with the new provider list.
                    *poller_signal.write() = Poller::new(new_settings.clone());
                    // Update the global settings signal — the coroutine reads
                    // poll_frequency from this on the next sleep iteration.
                    *settings_signal.write() = new_settings;
                },
                "Save"
            }
        }
    }
}
