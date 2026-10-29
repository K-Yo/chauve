use dioxus::prelude::*;
use tokio::time::Duration;

/// Bottom-center popover that fades out on its own after 5s, or on click.
#[component]
pub fn Toast(message: String) -> Element {
    let mut visible: Signal<bool> = use_signal(|| true);

    use_future(move || async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        visible.set(false);
    });

    let opacity_class = if visible() {
        "opacity-100"
    } else {
        "opacity-0 pointer-events-none"
    };

    rsx! {
        div {
            class: "fixed bottom-8 left-1/2 -translate-x-1/2 z-50 bg-gray-800 text-white text-sm px-4 py-2 rounded shadow-lg transition-opacity duration-500 {opacity_class}",
            onclick: move |_| visible.set(false),
            "{message}"
        }
    }
}
