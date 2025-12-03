use crate:: poller::Poller;
use dioxus::prelude::*;

#[component]
fn LastUpdate() -> Element {
    
    let last_poll_time = {
        let poller_signal = consume_context::<Signal<Poller>>();
        let poller = poller_signal.read();
        poller.clone().last_poll_time()
    };
    rsx! {
        "last poll time: {last_poll_time}"
    }
}

#[component]
pub fn Header() -> Element {
    rsx! {
        LastUpdate { }
    }
}
