use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use web_sys::{console, window};
use wasm_bindgen_futures::spawn_local;
use crate::freighter::{connect_wallet, is_freighter_available};
use crate::soroban::complete_join_flow;

#[function_component(GamePage)]
pub fn game_page() -> Html {
    let username = use_state(|| LocalStorage::get::<String>("username").unwrap_or_else(|_| "Unknown".to_string()));
    let wallet_address = use_state(|| None::<String>);
    let loading = use_state(|| true);
    let error_message = use_state(|| None::<String>);
    let joining_contract = use_state(|| false);
    let join_result = use_state(|| None::<String>);

    // ===== On mount: load wallet =====
    {
        let wallet_address = wallet_address.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();

        use_effect_with((), move |_| {
            console::log_1(&"🔍 GamePage: Checking for stored wallet data...".into());

            match LocalStorage::get::<String>("wallet_address") {
                Ok(stored_wallet) => {
                    wallet_address.set(Some(stored_wallet));
                    loading.set(false);
                    return;
                }
                Err(_) => {}
            }

            if !is_freighter_available() {
                error_message.set(Some("Freighter wallet not found. Please install Freighter.".to_string()));
                loading.set(false);
                return;
            }

            error_message.set(Some("No wallet connection found. Connect via the Login page.".to_string()));
            loading.set(false);
        });
    }

    // ===== Auto-redirect to game after wallet connection =====
    {
        let wallet_address = wallet_address.clone();

        use_effect_with(wallet_address.clone(), move |wallet_addr| {
            if let Some(_) = wallet_addr.as_ref() {
                console::log_1(&"🎮 Wallet connected, redirecting to game...".into());

                // Redirect to backend-served game (bypasses frontend routing)
                if let Some(window) = window() {
                    let _ = window.location().set_href("/game");
                }
            }
        });
    }

    // ===== Component mount log =====
    use_effect(|| {
        console::log_1(&"GamePage component mounted".into());
        || console::log_1(&"GamePage component unmounted".into())
    });

    // ===== Callbacks =====
    let on_manual_connect = {
        let wallet_address = wallet_address.clone();
        let error_message = error_message.clone();
        let loading = loading.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let wallet_address = wallet_address.clone();
            let error_message = error_message.clone();
            let loading = loading.clone();

            loading.set(true);
            error_message.set(None);

            spawn_local(async move {
                match connect_wallet().await {
                    Ok(address) => {
                        wallet_address.set(Some(address.clone()));
                        let _ = LocalStorage::set("wallet_address", &address);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to connect: {}", e)));
                    }
                }
                loading.set(false);
            });
        })
    };

    let _on_join_contract = {
        let wallet_address = wallet_address.clone();
        let username = username.clone();
        let joining_contract = joining_contract.clone();
        let join_result = join_result.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let wallet_addr = wallet_address.as_ref().unwrap().clone();
            let username_val: String = (*username).clone();

            joining_contract.set(true);
            join_result.set(None);

            let joining_contract = joining_contract.clone();
            let join_result = join_result.clone();

            spawn_local(async move {
                match complete_join_flow(&wallet_addr, &username_val).await {
                    Ok(result) => {
                        join_result.set(Some(format!(
                            "✅ Joined! Tx: {}...{}",
                            &result.hash[0..8],
                            &result.hash[result.hash.len()-8..]
                        )));
                    }
                    Err(e) => {
                        join_result.set(Some(format!("❌ Failed: {}", e)));
                    }
                }
                joining_contract.set(false);
            });
        })
    };

    // ===== Conditional Rendering =====
    if *loading {
        return html! { <div>{"Loading wallet..."}</div> };
    }

    if let Some(error) = (*error_message).clone() {
        return html! { <div>{format!("Wallet error: {}", error)}</div> };
    }

    if wallet_address.is_none() {
        return html! {
            <div>
                <p>{"No wallet connected"}</p>
                <button onclick={on_manual_connect}>{ "Connect Freighter" }</button>
            </div>
        };
    }

    // This will never be reached since we redirect immediately after wallet connection
    // But keeping it for safety
    html! {
        <div class="setup-container">
            <h1>{"🌟 Stellar Heads"}</h1>
            <p>{"Redirecting to game..."}</p>
        </div>
    }
}