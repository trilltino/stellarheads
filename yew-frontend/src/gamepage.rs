use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use web_sys::console;
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

    // ===== Component mount log and game initialization =====
    let game_initialized = use_state(|| false);

    {
        let game_initialized = game_initialized.clone();
        let wallet_address = wallet_address.clone();

        use_effect_with(wallet_address.clone(), move |wallet_addr| {
            // Only initialize game when wallet is connected and game hasn't been initialized yet
            if wallet_addr.is_some() && !*game_initialized {
                console::log_1(&"GamePage component mounted".into());

                let game_initialized = game_initialized.clone();

                // Load and initialize the game WASM module
                wasm_bindgen_futures::spawn_local(async move {
                    // Small delay to ensure canvas is in DOM
                    gloo::timers::future::sleep(std::time::Duration::from_millis(500)).await;

                    console::log_1(&"Loading Stellar Heads game WASM...".into());

                    // Load the game WASM from the backend
                    match load_game_wasm().await {
                        Ok(_) => {
                            game_initialized.set(true);
                            console::log_1(&"Game initialization completed successfully".into());
                        },
                        Err(e) => {
                            console::log_1(&format!("Failed to load game: {}", e).into());
                        }
                    }
                });
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
                <canvas id="stellar-heads-canvas"></canvas>
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

                #stellar-heads-canvas {
                    width: 1366px;
                    height: 768px;
                    max-width: 100%;
                    max-height: 100%;
                    border: 2px solid rgba(255, 255, 255, 0.2);
                    border-radius: 12px;
                    background: #000;
                    display: block;
                    cursor: crosshair;
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

async fn load_game_wasm() -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    use web_sys::{HtmlScriptElement, Document, Window};

    let window: Window = web_sys::window().ok_or("No window object")?;
    let document: Document = window.document().ok_or("No document object")?;

    // Check if canvas exists
    if document.get_element_by_id("stellar-heads-canvas").is_none() {
        return Err("Canvas element not found".to_string());
    }

    console::log_1(&"Canvas found, loading game WASM...".into());

    // Create script element to load the game WASM
    let script: HtmlScriptElement = document
        .create_element("script")
        .map_err(|e| format!("Failed to create element: {:?}", e))?
        .dyn_into()
        .map_err(|_| "Failed to create script element")?;

    script.set_type("module");
    script.set_text_content(Some(r#"
        // Try to load the game using a simple fetch approach instead of ES modules
        async function loadGame() {
            // Prevent multiple initialization attempts
            if (window.stellarHeadsGameInitialized) {
                console.log('🎮 Game already initialized, skipping...');
                return;
            }

            // Mark as initializing to prevent concurrent attempts
            if (window.stellarHeadsGameInitializing) {
                console.log('🎮 Game initialization already in progress, skipping...');
                return;
            }

            window.stellarHeadsGameInitializing = true;

            try {
                console.log('🎮 Initializing Stellar Heads WASM...');

                // Clean up any previous instances
                if (window.stellarHeadsGameInstance) {
                    console.log('🧹 Cleaning up previous game instance...');
                    try {
                        window.stellarHeadsGameInstance = null;
                    } catch (e) {
                        console.warn('Warning: Could not clean up previous instance:', e);
                    }
                }

                // Load the game script directly
                const gameScript = document.createElement('script');
                gameScript.type = 'module';
                gameScript.innerHTML = `
                    import init, { main_js } from 'http://localhost:3000/game/stellar_heads_game.js';

                    try {
                        await init('http://localhost:3000/game/stellar_heads_game_bg.wasm');
                        console.log('✅ WASM loaded, starting game...');
                        window.stellarHeadsGameInstance = main_js();
                        window.stellarHeadsGameInitialized = true;
                        window.stellarHeadsGameInitializing = false;
                        console.log('✅ Bevy game started successfully!');

                        // Auto-start game by simulating key press after a delay
                        setTimeout(() => {
                            console.log('🚀 Auto-starting game...');
                            // Simulate Space key press to start game
                            const canvas = document.getElementById('stellar-heads-canvas');
                            if (canvas) {
                                canvas.focus();
                                const event = new KeyboardEvent('keydown', { code: 'Space', key: ' ' });
                                canvas.dispatchEvent(event);

                                // Also try Enter key
                                setTimeout(() => {
                                    const enterEvent = new KeyboardEvent('keydown', { code: 'Enter', key: 'Enter' });
                                    canvas.dispatchEvent(enterEvent);
                                }, 500);
                            }
                        }, 2000);
                    } catch (error) {
                        console.error('❌ Failed to load game:', error);
                        window.stellarHeadsGameInitialized = false;
                        window.stellarHeadsGameInitializing = false;
                        window.stellarHeadsGameInstance = null;
                    }
                `;
                document.head.appendChild(gameScript);

            } catch (error) {
                console.error('❌ Failed to load game:', error);
                // Reset flags on error to allow retry
                window.stellarHeadsGameInitialized = false;
                window.stellarHeadsGameInitializing = false;
                window.stellarHeadsGameInstance = null;
            }
        }

        loadGame();
    "#));

    document
        .head()
        .ok_or("No head element")?
        .append_child(&script)
        .map_err(|e| format!("Failed to append script: {:?}", e))?;

    Ok(())
}