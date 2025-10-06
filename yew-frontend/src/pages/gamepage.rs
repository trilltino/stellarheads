use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use web_sys::{console, MessageEvent};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use gloo::events::EventListener;
use crate::wallet::{connect_wallet, is_freighter_available, sign_transaction};
use shared::dto::contract::{ContractSubmitRequest, ContractSubmitResponse, LeaderboardFunction};
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
    #[allow(dead_code)]
    player_address: String,
    #[allow(dead_code)]
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

#[derive(Debug, Clone, Deserialize)]
struct StoreGameResultResponse {
    #[allow(dead_code)]
    game_id: i32,
    contract_xdr: Option<ContractXdrInfo>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContractXdrInfo {
    xdr: String,
    function_name: String,
    description: String,
    network_passphrase: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    #[allow(dead_code)]
    message: Option<String>,
}

#[function_component(GamePage)]
pub fn game_page() -> Html {
    let username = use_state(|| LocalStorage::get::<String>("username").unwrap_or_else(|_| "Unknown".to_string()));
    let wallet_address = use_state(|| None::<String>);
    let loading = use_state(|| true);
    let error_message = use_state(|| None::<String>);
    let pending_xdr = use_state(|| None::<ContractXdrInfo>);
    let signing_transaction = use_state(|| false);
    let join_status_checked = use_state(|| false);
    let needs_to_join = use_state(|| false);
    let checking_join_status = use_state(|| false);
    let has_joined_contract = use_state(|| false);
    let show_game = use_state(|| false);
    let contract_functions_visible = use_state(|| false);
    let auto_loading_game = use_state(|| false);

    // ===== On mount: load wallet =====
    {
        let wallet_address = wallet_address.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();

        use_effect_with((), move |_| {
            console::log_1(&"🔍 GamePage: Checking for stored wallet data...".into());

            if let Ok(stored_wallet) = LocalStorage::get::<String>("wallet_address") {
                wallet_address.set(Some(stored_wallet));
                loading.set(false);
            } else {
                spawn_local(async move {
                    if !is_freighter_available().await {
                        error_message.set(Some("Freighter wallet not found. Please install Freighter.".to_string()));
                        loading.set(false);
                        return;
                    }

                    error_message.set(Some("No wallet connection found. Connect via the Login page.".to_string()));
                    loading.set(false);
                });
            }

            || ()
      });
    }

    // ===== Check join status after wallet connection =====
    {
        let wallet_address = wallet_address.clone();
        let join_status_checked = join_status_checked.clone();
        let needs_to_join = needs_to_join.clone();
        let checking_join_status = checking_join_status.clone();

        use_effect_with(wallet_address.clone(), move |wallet_addr| {
            if let Some(wallet) = wallet_addr.as_ref() {
                if !*join_status_checked {
                    console::log_1(&"🔍 Checking if player has joined leaderboard...".into());
                    checking_join_status.set(true);

                    let wallet_clone = wallet.clone();
                    let join_status_checked_clone = join_status_checked.clone();
                    let needs_to_join_clone = needs_to_join.clone();
                    let checking_join_status_clone = checking_join_status.clone();

                    spawn_local(async move {
                        match check_join_status(&wallet_clone).await {
                            Ok(response) => {
                                console::log_1(&format!("📊 Join status result: needs_join={}", response.needs_join_xdr).into());
                                needs_to_join_clone.set(response.needs_join_xdr);
                                join_status_checked_clone.set(true);
                                if response.needs_join_xdr {
                                    console::log_1(&"⚠️ Player needs to join leaderboard first".into());
                                } else {
                                    console::log_1(&"✅ Player already joined, ready for game".into());
                                }
                            },
                            Err(e) => {
                                console::log_1(&format!("❌ Failed to check join status: {e}").into());
                                // Assume they need to join on error
                                needs_to_join_clone.set(true);
                                join_status_checked_clone.set(true);
                            }
                        }
                        checking_join_status_clone.set(false);
                    });
                }
            }
        });
    }

    // ===== Auto-load game after successful join =====
    {
        let auto_loading_game = auto_loading_game.clone();
        let show_game = show_game.clone();

        use_effect_with(auto_loading_game.clone(), move |auto_loading| {
            if **auto_loading {
                console::log_1(&"⏱️ Starting 3-second countdown to load game...".into());

                let show_game_clone = show_game.clone();
                let auto_loading_game_clone = auto_loading_game.clone();

                spawn_local(async move {
                    // Wait 3 seconds
                    gloo::timers::future::sleep(std::time::Duration::from_secs(3)).await;

                    console::log_1(&"🎮 Auto-loading game now!".into());
                    show_game_clone.set(true);
                    auto_loading_game_clone.set(false);  // Reset the auto-loading state
                });
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
        let pending_xdr = pending_xdr.clone();

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
                                    let pending_xdr_clone = pending_xdr.clone();
                                    spawn_local(async move {
                                        match send_game_result_to_backend(api_request_clone).await {
                                            Ok(Some(xdr_info)) => {
                                                console::log_1(&"✅ Game result sent to backend successfully!".into());
                                                console::log_1(&format!("🏆 Win detected! XDR generated for {}", xdr_info.function_name).into());
                                                pending_xdr_clone.set(Some(xdr_info));
                                            },
                                            Ok(None) => {
                                                console::log_1(&"✅ Game result sent to backend successfully! (No contract action needed)".into());
                                            },
                                            Err(e) => console::log_1(&format!("❌ Failed to send game result to backend: {e:?}").into()),
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
                        error_message.set(Some(format!("Failed to connect: {e}")));
                    }
                }
                loading.set(false);
            });
        })
    };

    // Sign transaction callback for wins
    let on_sign_transaction = {
        let pending_xdr = pending_xdr.clone();
        let signing_transaction = signing_transaction.clone();
        let wallet_address = wallet_address.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let pending_xdr = pending_xdr.clone();
            let signing_transaction = signing_transaction.clone();
            let wallet_address = wallet_address.clone();

            if let Some(xdr_info) = (*pending_xdr).clone() {
                signing_transaction.set(true);

                spawn_local(async move {
                    console::log_1(&format!("🖊️ Signing transaction for {}", xdr_info.function_name).into());

                    match sign_transaction(&xdr_info.xdr, &xdr_info.network_passphrase).await {
                        Ok(signed_xdr) => {
                            console::log_1(&"✅ Transaction signed successfully!".into());

                            // Submit signed transaction to backend
                            // Create the LeaderboardFunction for add_win
                            let wallet = (*wallet_address).clone().unwrap_or_default();
                            let function = LeaderboardFunction::AddWin {
                                player: wallet.clone()
                            };

                            let submit_request = ContractSubmitRequest {
                                signed_xdr,
                                function,
                                wallet_type: Some("Freighter".to_string()),
                            };

                            match submit_signed_transaction(submit_request).await {
                                Ok(response) => {
                                    if let Some(hash) = response.transaction_hash {
                                        console::log_1(&format!("🎉 Transaction submitted! Hash: {}", hash).into());
                                    } else {
                                        console::log_1(&"🎉 Transaction submitted successfully!".into());
                                    }
                                    pending_xdr.set(None);
                                },
                                Err(e) => {
                                    console::log_1(&format!("❌ Failed to submit transaction: {e}").into());
                                }
                            }
                        },
                        Err(e) => {
                            console::log_1(&format!("❌ Failed to sign transaction: {e}").into());
                        }
                    }
                    signing_transaction.set(false);
                });
            }
        })
    };

    let on_dismiss_xdr = {
        let pending_xdr = pending_xdr.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            pending_xdr.set(None);
        })
    };

    // Join leaderboard callback
    let on_join_leaderboard = {
        let wallet_address = wallet_address.clone();
        let signing_transaction = signing_transaction.clone();
        let needs_to_join = needs_to_join.clone();
        let has_joined_contract = has_joined_contract.clone();
        let auto_loading_game = auto_loading_game.clone();
        let contract_functions_visible = contract_functions_visible.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let wallet_address = wallet_address.clone();
            let signing_transaction = signing_transaction.clone();
            let needs_to_join = needs_to_join.clone();
            let has_joined_contract = has_joined_contract.clone();
            let auto_loading_game = auto_loading_game.clone();
            let contract_functions_visible = contract_functions_visible.clone();

            if let Some(wallet) = (*wallet_address).clone() {
                console::log_1(&"🎯 Starting join leaderboard process...".into());
                signing_transaction.set(true);

                spawn_local(async move {
                    match generate_join_xdr(&wallet).await {
                        Ok(xdr_info) => {
                            console::log_1(&"✅ Join XDR generated successfully!".into());
                            console::log_1(&format!("🔍 Join XDR preview: {}...", &xdr_info.xdr[0..50.min(xdr_info.xdr.len())]).into());

                            match sign_transaction(&xdr_info.xdr, &xdr_info.network_passphrase).await {
                                Ok(signed_xdr) => {
                                    console::log_1(&"✅ Join transaction signed successfully!".into());

                                    let function = LeaderboardFunction::Join {
                                        player: wallet.clone()
                                    };

                                    let submit_request = ContractSubmitRequest {
                                        signed_xdr,
                                        function,
                                        wallet_type: Some("Freighter".to_string()),
                                    };

                                    match submit_signed_transaction(submit_request).await {
                                        Ok(response) => {
                                            if let Some(hash) = response.transaction_hash {
                                                console::log_1(&format!("🎉 Join transaction submitted! Hash: {}", hash).into());
                                            } else {
                                                console::log_1(&"🎉 Join transaction submitted successfully!".into());
                                            }
                                            needs_to_join.set(false);
                                            has_joined_contract.set(true);
                                            auto_loading_game.set(true);
                                            contract_functions_visible.set(true);
                                            console::log_1(&"🎮 Player joined! Starting auto-load sequence...".into());
                                        },
                                        Err(e) => {
                                            console::log_1(&format!("❌ Failed to submit join transaction: {e}").into());
                                        }
                                    }
                                },
                                Err(e) => {
                                    console::log_1(&format!("❌ Failed to sign join transaction: {e}").into());
                                }
                            }
                        },
                        Err(e) => {
                            console::log_1(&format!("❌ Failed to generate join XDR: {e}").into());
                        }
                    }
                    signing_transaction.set(false);
                });
            }
        })
    };

    // Generic contract function callback
    let create_contract_function_callback = {
        let wallet_address = wallet_address.clone();
        let signing_transaction = signing_transaction.clone();
        let pending_xdr = pending_xdr.clone();

        move |function: LeaderboardFunction, _description: String| {
            let wallet_address = wallet_address.clone();
            let signing_transaction = signing_transaction.clone();
            let pending_xdr = pending_xdr.clone();

            Callback::from(move |_: web_sys::MouseEvent| {
                let wallet_address = wallet_address.clone();
                let signing_transaction = signing_transaction.clone();
                let pending_xdr = pending_xdr.clone();
                let function = function.clone();

                if let Some(wallet) = (*wallet_address).clone() {
                    console::log_1(&format!("🎯 Starting {} process...", function.display_name()).into());
                    signing_transaction.set(true);

                    spawn_local(async move {
                        match generate_contract_xdr(&wallet, function.clone()).await {
                            Ok(xdr_info) => {
                                console::log_1(&format!("✅ {} XDR generated successfully!", function.display_name()).into());
                                pending_xdr.set(Some(xdr_info));
                            },
                            Err(e) => {
                                console::log_1(&format!("❌ Failed to generate {} XDR: {e}", function.display_name()).into());
                            }
                        }
                        signing_transaction.set(false);
                    });
                }
            })
        }
    };

    // Contract function callbacks
    let on_add_win = create_contract_function_callback(
        LeaderboardFunction::AddWin { player: wallet_address.as_ref().unwrap_or(&"".to_string()).clone() },
        "Add a win to your record".to_string()
    );

    let on_get_wins = create_contract_function_callback(
        LeaderboardFunction::GetWins { player: wallet_address.as_ref().unwrap_or(&"".to_string()).clone() },
        "Get your total wins".to_string()
    );

    let on_get_my_wins = create_contract_function_callback(
        LeaderboardFunction::GetMyWins { player: wallet_address.as_ref().unwrap_or(&"".to_string()).clone() },
        "Get your wins (explicit call)".to_string()
    );

    let on_get_player = create_contract_function_callback(
        LeaderboardFunction::GetPlayer { player: wallet_address.as_ref().unwrap_or(&"".to_string()).clone() },
        "Get your player information".to_string()
    );

    let on_get_all_players = create_contract_function_callback(
        LeaderboardFunction::GetAllPlayers,
        "Get all players on the leaderboard".to_string()
    );

    let on_get_leaderboard = create_contract_function_callback(
        LeaderboardFunction::GetLeaderboard { limit: 10 },
        "Get top 10 players".to_string()
    );

    let on_get_player_count = create_contract_function_callback(
        LeaderboardFunction::GetPlayerCount,
        "Get total player count".to_string()
    );

    let on_check_joined = create_contract_function_callback(
        LeaderboardFunction::HasJoined { player: wallet_address.as_ref().unwrap_or(&"".to_string()).clone() },
        "Check if you've joined the leaderboard".to_string()
    );

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

            // Show join leaderboard banner if needed
            {if *checking_join_status {
                html! {
                    <div class="join-banner checking">
                        <div class="join-content">
                            <div class="join-icon">{"🔍"}</div>
                            <div class="join-text">
                                <h3>{"Checking Leaderboard Status..."}</h3>
                                <p>{"Verifying if you've joined the Stellar Heads leaderboard"}</p>
                            </div>
                        </div>
                    </div>
                }
            } else if *needs_to_join && *join_status_checked {
                html! {
                    <div class="join-banner">
                        <div class="join-content">
                            <div class="join-icon">{"🎯"}</div>
                            <div class="join-text">
                                <h3>{"Join the Leaderboard"}</h3>
                                <p>{"To record your wins on the Stellar blockchain, you need to join the leaderboard first."}</p>
                            </div>
                            <button
                                class="join-button"
                                onclick={on_join_leaderboard}
                                disabled={*signing_transaction}
                            >
                                {if *signing_transaction {
                                    "Joining..."
                                } else {
                                    "Join with Freighter"
                                }}
                            </button>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}

            // Contract Functions Panel (show after joining)
            {if *contract_functions_visible {
                html! {
                    <div class="contract-panel">
                        <div class="contract-header">
                            <h2>{"🔧 Smart Contract Functions"}</h2>
                            <p>{"Interact with the Stellar Heads leaderboard contract"}</p>
                        </div>
                        <div class="contract-functions">
                            <div class="function-group">
                                <h3>{"📊 Query Functions"}</h3>
                                <div class="function-buttons">
                                    <button class="contract-btn query" onclick={on_check_joined}>
                                        {"❓"} {"Check Join Status"}
                                    </button>
                                    <button class="contract-btn query" onclick={on_get_wins}>
                                        {"📊"} {"Get My Wins"}
                                    </button>
                                    <button class="contract-btn query" onclick={on_get_my_wins}>
                                        {"🏅"} {"Get My Wins (Alt)"}
                                    </button>
                                    <button class="contract-btn query" onclick={on_get_player}>
                                        {"👤"} {"Get Player Info"}
                                    </button>
                                    <button class="contract-btn query" onclick={on_get_all_players}>
                                        {"👥"} {"Get All Players"}
                                    </button>
                                    <button class="contract-btn query" onclick={on_get_leaderboard}>
                                        {"🏆"} {"Get Leaderboard"}
                                    </button>
                                    <button class="contract-btn query" onclick={on_get_player_count}>
                                        {"🔢"} {"Get Player Count"}
                                    </button>
                                </div>
                            </div>
                            <div class="function-group">
                                <h3>{"🏆 Action Functions"}</h3>
                                <div class="function-buttons">
                                    <button class="contract-btn action" onclick={on_add_win}>
                                        {"🏆"} {"Add Win"}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}

            // Game area (only show after joining)
            {if *show_game {
                html! {
                    <div class="game-section">
                        <div class="game-area">
                            <iframe
                                src="http://localhost:3000/game/index.html"
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
                    </div>
                }
            } else if *auto_loading_game {
                html! {
                    <div class="welcome-section">
                        <div class="welcome-content">
                            <h2>{"🎉 Welcome to Stellar Heads!"}</h2>
                            <p>{"You've successfully joined the leaderboard. The game will load shortly..."}</p>
                            <div class="loading-game">
                                <div class="loading-spinner-small"></div>
                                <span>{"Loading game..."}</span>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}

            // Show transaction signing modal if there's pending XDR
            {if let Some(xdr_info) = (*pending_xdr).clone() {
                html! {
                    <div class="modal-overlay">
                        <div class="modal-content">
                            <h2 class="modal-title">{"🏆 Victory Recorded!"}</h2>
                            <p class="modal-description">
                                {"Congratulations! You won the match. Sign the transaction to record your win on the Stellar blockchain."}
                            </p>
                            <div class="modal-info">
                                <div class="info-row">
                                    <span class="info-label">{"Action:"}</span>
                                    <span class="info-value">{&xdr_info.description}</span>
                                </div>
                                <div class="info-row">
                                    <span class="info-label">{"Function:"}</span>
                                    <span class="info-value">{&xdr_info.function_name}</span>
                                </div>
                            </div>
                            <div class="modal-buttons">
                                <button
                                    class="btn-sign"
                                    onclick={on_sign_transaction.clone()}
                                    disabled={*signing_transaction}
                                >
                                    {if *signing_transaction {
                                        "Signing..."
                                    } else {
                                        "Sign with Freighter"
                                    }}
                                </button>
                                <button
                                    class="btn-dismiss"
                                    onclick={on_dismiss_xdr}
                                    disabled={*signing_transaction}
                                >
                                    {"Skip"}
                                </button>
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}

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

                .modal-overlay {
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: rgba(0, 0, 0, 0.8);
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    z-index: 1000;
                    backdrop-filter: blur(5px);
                }

                .modal-content {
                    background: linear-gradient(135deg, #1a1a2e, #16213e);
                    border-radius: 16px;
                    padding: 2rem;
                    max-width: 500px;
                    width: 90%;
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
                }

                .modal-title {
                    font-size: 1.8rem;
                    margin-bottom: 1rem;
                    background: linear-gradient(135deg, #ffd700, #ffed4e);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    text-align: center;
                }

                .modal-description {
                    color: rgba(255, 255, 255, 0.8);
                    margin-bottom: 1.5rem;
                    text-align: center;
                    line-height: 1.5;
                }

                .modal-info {
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 8px;
                    padding: 1rem;
                    margin-bottom: 1.5rem;
                }

                .info-row {
                    display: flex;
                    justify-content: space-between;
                    margin-bottom: 0.5rem;
                }

                .info-row:last-child {
                    margin-bottom: 0;
                }

                .info-label {
                    color: rgba(255, 255, 255, 0.6);
                    font-size: 0.9rem;
                }

                .info-value {
                    color: white;
                    font-weight: 500;
                }

                .modal-buttons {
                    display: flex;
                    gap: 1rem;
                    justify-content: center;
                }

                .btn-sign, .btn-dismiss {
                    padding: 0.75rem 2rem;
                    border-radius: 8px;
                    font-size: 1rem;
                    font-weight: 600;
                    border: none;
                    cursor: pointer;
                    transition: all 0.3s ease;
                }

                .btn-sign {
                    background: linear-gradient(135deg, #4f46e5, #06b6d4);
                    color: white;
                }

                .btn-sign:hover:not(:disabled) {
                    transform: translateY(-2px);
                    box-shadow: 0 5px 15px rgba(79, 70, 229, 0.4);
                }

                .btn-sign:disabled {
                    opacity: 0.6;
                    cursor: not-allowed;
                }

                .btn-dismiss {
                    background: rgba(255, 255, 255, 0.1);
                    color: rgba(255, 255, 255, 0.7);
                }

                .btn-dismiss:hover:not(:disabled) {
                    background: rgba(255, 255, 255, 0.15);
                }

                .join-banner {
                    margin: 1rem 2rem;
                    background: linear-gradient(135deg, #4f46e5, #06b6d4);
                    border-radius: 12px;
                    padding: 0;
                    overflow: hidden;
                    box-shadow: 0 4px 12px rgba(79, 70, 229, 0.3);
                }

                .join-banner.checking {
                    background: linear-gradient(135deg, #6b7280, #9ca3af);
                }

                .join-content {
                    display: flex;
                    align-items: center;
                    padding: 1.5rem;
                    gap: 1.5rem;
                }

                .join-icon {
                    font-size: 3rem;
                    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
                }

                .join-text {
                    flex: 1;
                }

                .join-text h3 {
                    margin: 0 0 0.5rem 0;
                    font-size: 1.25rem;
                    font-weight: 600;
                    color: white;
                }

                .join-text p {
                    margin: 0;
                    color: rgba(255, 255, 255, 0.9);
                    line-height: 1.4;
                }

                .join-button {
                    padding: 0.75rem 2rem;
                    background: rgba(255, 255, 255, 0.2);
                    color: white;
                    border: 2px solid rgba(255, 255, 255, 0.3);
                    border-radius: 8px;
                    font-size: 1rem;
                    font-weight: 600;
                    cursor: pointer;
                    transition: all 0.3s ease;
                    backdrop-filter: blur(10px);
                }

                .join-button:hover:not(:disabled) {
                    background: rgba(255, 255, 255, 0.3);
                    border-color: rgba(255, 255, 255, 0.5);
                    transform: translateY(-2px);
                    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
                }

                .join-button:disabled {
                    opacity: 0.6;
                    cursor: not-allowed;
                }

                /* Contract Functions Panel */
                .contract-panel {
                    margin: 1rem 2rem;
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 12px;
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    overflow: hidden;
                    backdrop-filter: blur(10px);
                }

                .contract-header {
                    padding: 1.5rem 2rem;
                    background: rgba(79, 70, 229, 0.1);
                    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
                }

                .contract-header h2 {
                    margin: 0 0 0.5rem 0;
                    font-size: 1.5rem;
                    background: linear-gradient(135deg, #4f46e5, #06b6d4);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                }

                .contract-header p {
                    margin: 0;
                    color: rgba(255, 255, 255, 0.7);
                    font-size: 0.95rem;
                }

                .contract-functions {
                    padding: 2rem;
                }

                .function-group {
                    margin-bottom: 2rem;
                }

                .function-group:last-child {
                    margin-bottom: 0;
                }

                .function-group h3 {
                    margin: 0 0 1rem 0;
                    font-size: 1.1rem;
                    color: rgba(255, 255, 255, 0.9);
                    display: flex;
                    align-items: center;
                    gap: 0.5rem;
                }

                .function-buttons {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 0.75rem;
                }

                .contract-btn {
                    padding: 0.75rem 1rem;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    border-radius: 8px;
                    background: rgba(255, 255, 255, 0.05);
                    color: white;
                    font-size: 0.9rem;
                    font-weight: 500;
                    cursor: pointer;
                    transition: all 0.3s ease;
                    display: flex;
                    align-items: center;
                    gap: 0.5rem;
                    backdrop-filter: blur(10px);
                }

                .contract-btn:hover {
                    background: rgba(255, 255, 255, 0.1);
                    border-color: rgba(255, 255, 255, 0.3);
                    transform: translateY(-2px);
                    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
                }

                .contract-btn.query {
                    border-color: rgba(6, 182, 212, 0.4);
                }

                .contract-btn.query:hover {
                    background: rgba(6, 182, 212, 0.1);
                    border-color: rgba(6, 182, 212, 0.6);
                    box-shadow: 0 4px 12px rgba(6, 182, 212, 0.2);
                }

                .contract-btn.action {
                    border-color: rgba(79, 70, 229, 0.4);
                }

                .contract-btn.action:hover {
                    background: rgba(79, 70, 229, 0.1);
                    border-color: rgba(79, 70, 229, 0.6);
                    box-shadow: 0 4px 12px rgba(79, 70, 229, 0.2);
                }

                /* Welcome Section */
                .welcome-section {
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    padding: 4rem 2rem;
                    min-height: 400px;
                }

                .welcome-content {
                    text-align: center;
                    max-width: 500px;
                }

                .welcome-content h2 {
                    font-size: 2rem;
                    margin-bottom: 1rem;
                    background: linear-gradient(135deg, #ffd700, #ffed4e);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                }

                .welcome-content p {
                    color: rgba(255, 255, 255, 0.8);
                    font-size: 1.1rem;
                    line-height: 1.6;
                    margin-bottom: 2rem;
                }

                .loading-game {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 1rem;
                    color: rgba(255, 255, 255, 0.7);
                    font-size: 1rem;
                }

                .loading-spinner-small {
                    width: 20px;
                    height: 20px;
                    border: 2px solid rgba(255, 255, 255, 0.3);
                    border-top: 2px solid #4f46e5;
                    border-radius: 50%;
                    animation: spin 1s linear infinite;
                }

                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }

                /* Game Section */
                .game-section {
                    flex: 1;
                    display: flex;
                    flex-direction: column;
                }
                "#}
            </style>
        </div>
    }
}

async fn send_game_result_to_backend(request: StoreGameResultRequest) -> Result<Option<ContractXdrInfo>, Box<dyn std::error::Error>> {
    use gloo::net::http::Request;

    console::log_1(&format!("📡 Sending game result to backend: {request:?}").into());

    let response = Request::post("http://localhost:3000/api/games/store")
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if response.ok() {
        let response_text = response.text().await?;
        console::log_1(&format!("📥 Backend response: {}", &response_text).into());

        match serde_json::from_str::<ApiResponse<StoreGameResultResponse>>(&response_text) {
            Ok(api_response) => {
                if api_response.success {
                    if let Some(data) = api_response.data {
                        console::log_1(&"✅ Backend confirmed game result received".into());
                        Ok(data.contract_xdr)
                    } else {
                        Ok(None)
                    }
                } else {
                    let error = api_response.error.unwrap_or("Unknown error".to_string());
                    Err(format!("API error: {}", error).into())
                }
            },
            Err(e) => {
                console::log_1(&format!("❌ Failed to parse response: {e}").into());
                Err(format!("Parse error: {e}").into())
            }
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or("Unknown error".to_string());
        console::log_1(&format!("❌ Backend error ({status}): {error_text}").into());
        Err(format!("Backend error: {status} - {error_text}").into())
    }
}

async fn submit_signed_transaction(request: ContractSubmitRequest) -> Result<ContractSubmitResponse, Box<dyn std::error::Error>> {
    use gloo::net::http::Request;

    console::log_1(&format!("📡 Submitting signed transaction to backend").into());

    let response = Request::post("http://localhost:3000/api/contract/submit-transaction")
        .header("Content-Type", "application/json")
        .json(&request)?
        .send()
        .await?;

    if response.ok() {
        let response_text = response.text().await?;
        console::log_1(&format!("📥 Submit response: {}", &response_text).into());

        match serde_json::from_str::<ContractSubmitResponse>(&response_text) {
            Ok(submit_response) => {
                console::log_1(&"✅ Transaction submitted successfully".into());
                Ok(submit_response)
            },
            Err(e) => {
                console::log_1(&format!("❌ Failed to parse submit response: {e}").into());
                Err(format!("Parse error: {e}").into())
            }
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or("Unknown error".to_string());
        console::log_1(&format!("❌ Submit error ({status}): {error_text}").into());
        Err(format!("Submit error: {status} - {error_text}").into())
    }
}

#[derive(Debug, Clone, Deserialize)]
struct JoinStatusResponse {
    #[allow(dead_code)]
    player_address: String,
    #[allow(dead_code)]
    has_joined: bool,
    needs_join_xdr: bool,
}

async fn check_join_status(player_address: &str) -> Result<JoinStatusResponse, Box<dyn std::error::Error>> {
    use gloo::net::http::Request;

    console::log_1(&format!("🔍 Checking join status for player: {}", player_address).into());

    let url = format!("http://localhost:3000/api/contract/join-status?player_address={}", player_address);
    let response = Request::get(&url)
        .send()
        .await?;

    if response.ok() {
        let response_text = response.text().await?;
        console::log_1(&format!("📥 Join status response: {}", &response_text).into());

        match serde_json::from_str::<JoinStatusResponse>(&response_text) {
            Ok(join_response) => {
                console::log_1(&"✅ Join status checked successfully".into());
                Ok(join_response)
            },
            Err(e) => {
                console::log_1(&format!("❌ Failed to parse join status response: {e}").into());
                Err(format!("Parse error: {e}").into())
            }
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or("Unknown error".to_string());
        console::log_1(&format!("❌ Join status error ({status}): {error_text}").into());
        Err(format!("Join status error: {status} - {error_text}").into())
    }
}

// Generic function to generate XDR for any contract function
async fn generate_contract_xdr(
    player_address: &str,
    function: LeaderboardFunction
) -> Result<ContractXdrInfo, Box<dyn std::error::Error>> {
    use gloo::net::http::Request;

    console::log_1(&format!("🎯 Generating XDR for function: {} with player: {}", function.name(), player_address).into());

    #[derive(Serialize)]
    struct ContractXdrRequest {
        source_account: String,
        function: shared::dto::contract::LeaderboardFunction,
        wallet_type: String,
    }

    let xdr_request = ContractXdrRequest {
        source_account: player_address.to_string(),
        function,
        wallet_type: "Freighter".to_string(),
    };

    let response = Request::post("http://localhost:3000/api/contract/generate-xdr")
        .header("Content-Type", "application/json")
        .json(&xdr_request)?
        .send()
        .await?;

    if response.ok() {
        let response_text = response.text().await?;
        console::log_1(&format!("📥 Contract XDR response: {}", &response_text).into());

        match serde_json::from_str::<shared::dto::contract::ContractXdrResponse>(&response_text) {
            Ok(xdr_response) => {
                if xdr_response.success {
                    if let Some(xdr) = xdr_response.xdr {
                        console::log_1(&format!("✅ {} XDR generated successfully", xdr_request.function.display_name()).into());
                        Ok(ContractXdrInfo {
                            xdr,
                            function_name: xdr_request.function.name().to_string(),
                            description: xdr_request.function.description().to_string(),
                            network_passphrase: "Test SDF Network ; September 2015".to_string(),
                        })
                    } else {
                        Err("XDR generation succeeded but no XDR returned".into())
                    }
                } else {
                    Err(format!("XDR generation failed: {}", xdr_response.message).into())
                }
            },
            Err(e) => {
                console::log_1(&format!("❌ Failed to parse XDR response: {e}").into());
                Err(format!("Parse error: {e}").into())
            }
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or("Unknown error".to_string());
        console::log_1(&format!("❌ XDR error ({status}): {error_text}").into());
        Err(format!("XDR error: {status} - {error_text}").into())
    }
}

async fn generate_join_xdr(player_address: &str) -> Result<ContractXdrInfo, Box<dyn std::error::Error>> {
    generate_contract_xdr(
        player_address,
        LeaderboardFunction::Join { player: player_address.to_string() }
    ).await
}

// Keep the old function for backward compatibility, but use the new generic one
async fn _generate_join_xdr_old(player_address: &str) -> Result<ContractXdrInfo, Box<dyn std::error::Error>> {
    use gloo::net::http::Request;

    console::log_1(&format!("🎯 Generating join XDR for player: {}", player_address).into());

    #[derive(Serialize)]
    struct JoinXdrRequest {
        source_account: String,
        function: shared::dto::contract::LeaderboardFunction,
        wallet_type: String,
    }

    let join_request = JoinXdrRequest {
        source_account: player_address.to_string(),
        function: shared::dto::contract::LeaderboardFunction::Join {
            player: player_address.to_string()
        },
        wallet_type: "Freighter".to_string(),
    };

    let response = Request::post("http://localhost:3000/api/contract/generate-xdr")
        .header("Content-Type", "application/json")
        .json(&join_request)?
        .send()
        .await?;

    if response.ok() {
        let response_text = response.text().await?;
        console::log_1(&format!("📥 Join XDR response: {}", &response_text).into());

        match serde_json::from_str::<shared::dto::contract::ContractXdrResponse>(&response_text) {
            Ok(xdr_response) => {
                if xdr_response.success {
                    if let Some(xdr) = xdr_response.xdr {
                        console::log_1(&"✅ Join XDR generated successfully".into());
                        console::log_1(&format!("🔍 Join XDR preview: {}...", &xdr[0..100.min(xdr.len())]).into());
                        Ok(ContractXdrInfo {
                            xdr,
                            function_name: "join".to_string(),
                            description: "Join the Stellar Heads leaderboard".to_string(),
                            network_passphrase: "Test SDF Network ; September 2015".to_string(),
                        })
                    } else {
                        Err("XDR generation succeeded but no XDR returned".into())
                    }
                } else {
                    Err(format!("XDR generation failed: {}", xdr_response.message).into())
                }
            },
            Err(e) => {
                console::log_1(&format!("❌ Failed to parse join XDR response: {e}").into());
                Err(format!("Parse error: {e}").into())
            }
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or("Unknown error".to_string());
        console::log_1(&format!("❌ Join XDR error ({status}): {error_text}").into());
        Err(format!("Join XDR error: {status} - {error_text}").into())
    }
}

