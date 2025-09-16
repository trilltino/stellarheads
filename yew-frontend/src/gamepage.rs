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

    // ===== Auto-set ready state after wallet connection =====
    {
        let wallet_address = wallet_address.clone();

        use_effect_with(wallet_address.clone(), move |wallet_addr| {
            if let Some(_) = wallet_addr.as_ref() {
                console::log_1(&"🎮 Wallet connected, ready for game...".into());
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

    // Wallet connected - show game ready screen
    html! {
        <div class="setup-container">
            <div class="game-ready-card">
                <div class="success-animation">
                    <div class="success-circle">
                        <div class="checkmark">{"✓"}</div>
                    </div>
                </div>

                <h1 class="game-title">{"🌟 Stellar Heads"}</h1>
                <h2 class="ready-title">{"Ready to Play!"}</h2>

                <div class="player-info">
                    <div class="info-row">
                        <span class="label">{"Player:"}</span>
                        <span class="value">{(*username).clone()}</span>
                    </div>
                    <div class="info-row">
                        <span class="label">{"Wallet:"}</span>
                        <span class="value">{
                            if let Some(addr) = wallet_address.as_ref() {
                                format!("{}...{}", &addr[0..4], &addr[addr.len()-4..])
                            } else {
                                "Unknown".to_string()
                            }
                        }</span>
                    </div>
                </div>

                <div class="launch-options">
                    <div class="launch-method">
                        <h3>{"Launch Native Game"}</h3>
                        <p>{"Run the game locally for the best performance"}</p>
                        <div class="code-block">
                            <code>{"cargo run --bin stellar_heads"}</code>
                        </div>
                        <small>{"Run this command in your game directory"}</small>
                    </div>
                </div>

                <div class="instructions">
                    <h4>{"Game Controls:"}</h4>
                    <div class="controls">
                        <span>{"A/D or ←/→ - Move"}</span>
                        <span>{"Space - Jump"}</span>
                        <span>{"X - Kick Ball"}</span>
                        <span>{"R - Reset Game"}</span>
                    </div>
                </div>
            </div>

            <style>
                {r#"
                .setup-container {
                    min-height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    padding: 2rem;
                    background: linear-gradient(135deg, #0d1117 0%, #1a1a2e 50%, #16213e 100%);
                }

                .game-ready-card {
                    background: rgba(255, 255, 255, 0.05);
                    backdrop-filter: blur(20px);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 24px;
                    padding: 3rem;
                    max-width: 600px;
                    text-align: center;
                    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
                }

                .success-animation {
                    margin-bottom: 2rem;
                }

                .success-circle {
                    width: 80px;
                    height: 80px;
                    border-radius: 50%;
                    background: linear-gradient(135deg, #10b981, #059669);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    margin: 0 auto;
                    animation: success-bounce 0.6s cubic-bezier(0.68, -0.55, 0.265, 1.55);
                }

                @keyframes success-bounce {
                    0% { transform: scale(0); }
                    50% { transform: scale(1.2); }
                    100% { transform: scale(1); }
                }

                .checkmark {
                    font-size: 2rem;
                    color: white;
                    font-weight: bold;
                }

                .game-title {
                    font-size: 2.5rem;
                    font-weight: 700;
                    background: linear-gradient(135deg, #4f46e5, #06b6d4, #8b5cf6);
                    background-size: 200% 200%;
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    animation: gradient-shift 3s ease-in-out infinite;
                    margin-bottom: 0.5rem;
                }

                @keyframes gradient-shift {
                    0%, 100% { background-position: 0% 50%; }
                    50% { background-position: 100% 50%; }
                }

                .ready-title {
                    color: #10b981;
                    font-size: 1.8rem;
                    margin-bottom: 2rem;
                }

                .player-info {
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 16px;
                    padding: 1.5rem;
                    margin-bottom: 2rem;
                }

                .info-row {
                    display: flex;
                    justify-content: space-between;
                    margin-bottom: 0.5rem;
                }

                .info-row:last-child {
                    margin-bottom: 0;
                }

                .label {
                    color: rgba(255, 255, 255, 0.7);
                    font-weight: 500;
                }

                .value {
                    color: #06b6d4;
                    font-weight: 600;
                }

                .launch-options {
                    margin-bottom: 2rem;
                }

                .launch-method {
                    background: rgba(255, 255, 255, 0.05);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 16px;
                    padding: 2rem;
                    text-align: center;
                }

                .launch-method h3 {
                    color: #4f46e5;
                    margin-bottom: 0.5rem;
                }

                .launch-method p {
                    color: rgba(255, 255, 255, 0.7);
                    margin-bottom: 1rem;
                }

                .code-block {
                    background: rgba(0, 0, 0, 0.3);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    border-radius: 8px;
                    padding: 1rem;
                    margin: 1rem 0;
                    font-family: 'Monaco', 'Consolas', monospace;
                }

                .code-block code {
                    color: #06b6d4;
                    font-size: 1.1rem;
                    font-weight: 600;
                }

                .instructions {
                    border-top: 1px solid rgba(255, 255, 255, 0.1);
                    padding-top: 1.5rem;
                }

                .instructions h4 {
                    color: rgba(255, 255, 255, 0.9);
                    margin-bottom: 1rem;
                }

                .controls {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 0.5rem;
                    font-family: 'Monaco', 'Consolas', monospace;
                    font-size: 0.9rem;
                }

                .controls span {
                    background: rgba(255, 255, 255, 0.05);
                    padding: 0.5rem 1rem;
                    border-radius: 8px;
                    color: rgba(255, 255, 255, 0.8);
                }
                "#}
            </style>
        </div>
    }
}