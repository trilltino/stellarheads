use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use web_sys::console;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures;
use crate::game_score::GameScore;
use crate::contract_test::ContractTest;

#[function_component(GamePage)]
pub fn game_page() -> Html {
    // Debug: Check if we have user data
    let username = LocalStorage::get::<String>("username").unwrap_or_else(|_| "Unknown".to_string());
    let wallet_address = LocalStorage::get::<String>("wallet_address").unwrap_or_else(|_| "None".to_string());
    
    // Log debug info
    console::log_1(&format!("GamePage loaded - Username: {}, Wallet: {}", username, wallet_address).into());
    
    use_effect(|| {
        console::log_1(&"GamePage component mounted".into());
        || console::log_1(&"GamePage component unmounted".into())
    });

    let on_load_game = Callback::from(|_| {
        console::log_1(&"Loading Stellar Heads game in iframe...".into());
        
        // Show/reload the iframe
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        if let Some(iframe) = document.get_element_by_id("game-iframe") {
            // Reload the iframe with a cache-busting parameter
            let iframe: web_sys::HtmlElement = iframe.dyn_into().unwrap();
            let timestamp = js_sys::Date::now();
            iframe.set_attribute("src", &format!("http://127.0.0.1:1334?t={}", timestamp)).unwrap();
        }
        
        // Hide the game status overlay
        if let Some(status) = document.get_element_by_id("game-status") {
            let status: web_sys::HtmlElement = status.dyn_into().unwrap();
            let _ = status.set_attribute("style", "display: none;");
        }
    });
    html! {
        <div class="game-container">
            <div class="header">
                <h1>{"🌟 Stellar Heads"}</h1>
                <div class="user-info">
                    <span>{format!("Player: {}", username)}</span>
                    <span class="wallet-addr">{format!("Wallet: {}...{}", 
                        &wallet_address[0..std::cmp::min(6, wallet_address.len())],
                        if wallet_address.len() > 6 { &wallet_address[wallet_address.len()-4..] } else { "" }
                    )}</span>
                </div>
                <nav>
                    <a href="/" class="nav-link">{"← Back to Login"}</a>
                </nav>
            </div>
            <div class="game-area">
                <div class="game-status" id="game-status">
                    <p>{"🎮 Stellar Heads Game"}</p>
                    <p class="status-text">{"Click 'Load Game' to start playing"}</p>
                    <div class="loading-spinner">
                        <div class="spinner"></div>
                    </div>
                </div>
                <iframe 
                    id="game-iframe"
                    src=""
                    width="100%"
                    height="100%"
                    frameborder="0"
                    title="Stellar Heads Game"
                    style="border: none; background: #1a1a2e;"
                />
            </div>
            <div class="controls">
                <p class="instructions">{"Game Status: "}<span style="color: #10b981;">{"✅ Running"}</span></p>
                <button onclick={on_load_game}>
                    {"🎮 Load Game"}
                </button>
                <br/>
                <p class="debug-info">{format!("Player: {} | Wallet: {}...{}", 
                    username, 
                    &wallet_address[0..std::cmp::min(4, wallet_address.len())],
                    if wallet_address.len() > 8 { &wallet_address[wallet_address.len()-4..] } else { "" }
                )}</p>
                <p class="server-info">{"Game server: http://127.0.0.1:1334"}</p>
                
                <div class="blockchain-demo">
                    <ContractTest />
                    <br/>
                    <h3>{"🌟 Score Demo"}</h3>
                    <p>{"Test score submission to Soroban contract:"}</p>
                    <GameScore 
                        score={12500}
                        game_mode={"demo".to_string()}
                        duration={180}
                        achievements={vec!["high_score".to_string(), "speed_demon".to_string()]}
                    />
                </div>
            </div>
            <style>
                {r#"
                .game-container {
                    display: flex;
                    flex-direction: column;
                    height: 100vh;
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                }
                
                .header {
                    background: #667eea;
                    color: white;
                    padding: 1rem;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                }
                
                .user-info {
                    display: flex;
                    flex-direction: column;
                    font-size: 0.9rem;
                }
                
                .wallet-addr {
                    font-family: 'Monaco', 'Consolas', monospace;
                    opacity: 0.8;
                }
                
                .nav-link {
                    color: white;
                    text-decoration: none;
                    padding: 0.5rem 1rem;
                    border-radius: 4px;
                    transition: background-color 0.3s;
                }
                
                .nav-link:hover {
                    background-color: rgba(255, 255, 255, 0.1);
                }
                
                .game-area {
                    flex: 1;
                    background: #000;
                    position: relative;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    min-height: 500px;
                }
                
                #stellar-heads-canvas {
                    max-width: 100%;
                    max-height: 100%;
                    width: 1280px;
                    height: 720px;
                    border: 2px solid #667eea;
                    border-radius: 8px;
                    background: #1a1a2e;
                    display: block;
                }
                
                .game-status {
                    position: absolute;
                    top: 50%;
                    left: 50%;
                    transform: translate(-50%, -50%);
                    text-align: center;
                    z-index: 10;
                    background: rgba(26, 26, 46, 0.95);
                    color: white;
                    padding: 2rem;
                    border-radius: 12px;
                    pointer-events: none;
                    border: 1px solid #667eea;
                    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.6);
                }
                
                .status-text {
                    font-size: 0.9rem;
                    color: #ccc;
                    margin-top: 0.5rem;
                }
                
                .loading-spinner {
                    margin-top: 1rem;
                }
                
                .spinner {
                    width: 40px;
                    height: 40px;
                    border: 3px solid rgba(255, 255, 255, 0.3);
                    border-top: 3px solid #667eea;
                    border-radius: 50%;
                    animation: spin 1s linear infinite;
                    margin: 0 auto;
                }
                
                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }
                
                #game-iframe {
                    position: absolute;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100%;
                    z-index: 1;
                }
                
                .controls {
                    background: #e9ecef;
                    padding: 1rem;
                    text-align: center;
                }
                
                .instructions {
                    margin: 1rem 0 0.5rem 0;
                    font-weight: 600;
                }
                
                .debug-info {
                    font-size: 0.8rem;
                    color: #666;
                    margin-top: 1rem;
                }
                
                .server-info {
                    font-size: 0.7rem;
                    color: #888;
                    font-family: 'Monaco', 'Consolas', monospace;
                    margin-top: 0.5rem;
                }

                .blockchain-demo {
                    margin-top: 2rem;
                    padding: 1.5rem;
                    background: rgba(79, 70, 229, 0.1);
                    border: 1px solid rgba(79, 70, 229, 0.3);
                    border-radius: 12px;
                }

                .blockchain-demo h3 {
                    margin: 0 0 1rem 0;
                    color: #4f46e5;
                    font-size: 1.2rem;
                }

                .blockchain-demo p {
                    margin: 0 0 1rem 0;
                    color: #666;
                    font-size: 0.9rem;
                }
                
                code {
                    background: #495057;
                    color: #f8f9fa;
                    padding: 0.5rem;
                    border-radius: 4px;
                    font-family: 'Monaco', 'Consolas', monospace;
                }
                "#}
            </style>
        </div>
    }
}