use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use web_sys::{console, MessageEvent};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use gloo::events::EventListener;
use crate::freighter::{connect_wallet, is_freighter_available};
// Removed unused import: use crate::soroban::complete_join_flow;

#[derive(Debug, Deserialize)]
struct GameResultMessage {
    #[serde(rename = "type")]
    message_type: String,
    timestamp: i64,
    data: GameResultData,
}

#[derive(Debug, Deserialize)]
struct GameResultData {
    player_address: String,
    player_username: String,
    won: Option<bool>,
    score_left: i32,
    score_right: i32,
    match_duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
struct StoreGameResultRequest {
    game_session_id: String,
    player_username: String,
    player_wallet_address: String,
    player_result: String,
    player_score: i32,
    opponent_score: i32,
    duration_seconds: f32,
    game_mode: String,
}

#[function_component(GamePage)]
pub fn game_page() -> Html {
    let username = use_state(|| LocalStorage::get::<String>("username").unwrap_or_else(|_| "Unknown".to_string()));
    let wallet_address = use_state(|| None::<String>);
    let loading = use_state(|| true);
    let error_message = use_state(|| None::<String>);
    // Removed joining_contract and join_result state variables - were unused

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
    {
        let wallet_address = wallet_address.clone();

        use_effect_with(wallet_address.clone(), move |wallet_addr| {
            if wallet_addr.is_some() {
                console::log_1(&"GamePage component mounted".into());
                console::log_1(&"Game will load in iframe".into());
            }
        });
    }

    // ===== PostMessage listener for game results =====
    {
        let wallet_address = wallet_address.clone();
        let username = username.clone();

        use_effect_with(wallet_address.clone(), move |wallet_addr| {
            if let Some(wallet) = &**wallet_addr {
                console::log_1(&"🎧 Setting up PostMessage listener for game results".into());

                let wallet_clone = wallet.clone();
                let username_clone = (*username).clone();

                let window = web_sys::window().unwrap();
                let listener = EventListener::new(&window, "message", move |event| {
                    let message_event = event.dyn_ref::<MessageEvent>().unwrap();
                    let data = message_event.data();

                    // Try to parse as string first, then as JSON
                    if let Some(message_str) = data.as_string() {
                        match serde_json::from_str::<GameResultMessage>(&message_str) {
                            Ok(game_message) => {
                                if game_message.message_type == "game_result" {
                                    console::log_1(&"🎮 Received game result from iframe!".into());

                                    // Transform data for backend API
                                    let player_result = match game_message.data.won {
                                        Some(true) => "Win".to_string(),
                                        Some(false) => "Loss".to_string(),
                                        None => "Draw".to_string(),
                                    };

                                    let api_request = StoreGameResultRequest {
                                        game_session_id: format!("session_{}", game_message.timestamp),
                                        player_username: username_clone.clone(),
                                        player_wallet_address: wallet_clone.clone(),
                                        player_result,
                                        player_score: game_message.data.score_left,
                                        opponent_score: game_message.data.score_right,
                                        duration_seconds: game_message.data.match_duration_seconds as f32,
                                        game_mode: "single_player_vs_ai".to_string(),
                                    };

                                    // Send to backend
                                    let api_request_clone = api_request.clone();
                                    spawn_local(async move {
                                        match send_game_result_to_backend(api_request_clone).await {
                                            Ok(_) => console::log_1(&"✅ Game result sent to backend successfully!".into()),
                                            Err(e) => console::log_1(&format!("❌ Failed to send game result to backend: {:?}", e).into()),
                                        }
                                    });
                                }
                            }
                            Err(_) => {
                                // Not a game result message, ignore
                            }
                        }
                    }
                });

                // Store the listener to keep it alive without explicit leak
                std::mem::forget(listener);
            }
        });
    }

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

    // Contract join functionality removed - was unused dead code

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

    // Wallet connected - show game
    html! {
        <div class="game-container">
            <div class="game-header">
                <h1 class="game-title">{"Stellar Heads"}</h1>
                <div class="player-info">
                    <span class="player-name">{"Player: "}{(*username).clone()}</span>
                    <span class="wallet-info">{
                        if let Some(addr) = wallet_address.as_ref() {
                            format!("Wallet: {}...{}", &addr[0..4], &addr[addr.len()-4..])
                        } else {
                            "Wallet: Unknown".to_string()
                        }
                    }</span>
                </div>
            </div>

            <div class="game-area">
                <iframe
                    src="/game/index.html"
                    id="stellar-heads-frame"
                    title="Stellar Heads Game">
                </iframe>
            </div>

            <div class="game-controls">
                <div class="controls-info">
                    <span>{"A/D - Move"}</span>
                    <span>{"Space - Jump"}</span>
                    <span>{"X - Kick"}</span>
                </div>
            </div>
            <style>
                {r#"
                .game-container {
                    display: flex;
                    flex-direction: column;
                    min-height: 100vh;
                    background: linear-gradient(135deg, #0d1117 0%, #1a1a2e 50%, #16213e 100%);
                    color: white;
                    font-family: 'Arial', sans-serif;
                }

                .game-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 1rem 2rem;
                    background: rgba(255, 255, 255, 0.05);
                    backdrop-filter: blur(10px);
                    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
                }

                .game-title {
                    font-size: 2rem;
                    font-weight: 700;
                    background: linear-gradient(135deg, #4f46e5, #06b6d4);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    margin: 0;
                }

                .player-info {
                    display: flex;
                    gap: 2rem;
                    font-size: 0.9rem;
                }

                .player-name, .wallet-info {
                    color: rgba(255, 255, 255, 0.8);
                }

                .game-area {
                    flex: 1;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    padding: 2rem;
                    min-height: 600px;
                }

                #stellar-heads-frame {
                    width: 1366px;
                    height: 768px;
                    max-width: 100%;
                    max-height: 100%;
                    border-radius: 12px;
                    background: #000;
                    display: block;
                    border: none;
                }

                .game-controls {
                    padding: 1rem 2rem;
                    background: rgba(255, 255, 255, 0.05);
                    border-top: 1px solid rgba(255, 255, 255, 0.1);
                }

                .controls-info {
                    display: flex;
                    justify-content: center;
                    gap: 2rem;
                    font-size: 0.9rem;
                    color: rgba(255, 255, 255, 0.7);
                }

                .controls-info span {
                    padding: 0.5rem 1rem;
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 6px;
                }
                "#}
            </style>
        </div>
    }
}

async fn send_game_result_to_backend(request: StoreGameResultRequest) -> Result<(), Box<dyn std::error::Error>> {
    use gloo::net::http::Request;

    console::log_1(&format!("📡 Sending game result to backend: {:?}", request).into());

    let response = Request::post("/api/games/store")
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if response.ok() {
        console::log_1(&"✅ Backend confirmed game result received".into());
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or("Unknown error".to_string());
        console::log_1(&format!("❌ Backend error ({}): {}", status, error_text).into());
        Err(format!("Backend error: {} - {}", status, error_text).into())
    }
}

