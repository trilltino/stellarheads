use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use web_sys::console;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures;
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

    // On component mount, get wallet address from localStorage or try Freighter as fallback
    {
        let wallet_address = wallet_address.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        
        use_effect_with(
            (),
            move |_| {
                // First try to get wallet from localStorage (from LoginPage)
                if let Ok(stored_wallet) = LocalStorage::get::<String>("wallet_address") {
                    console::log_1(&format!("✅ Using stored wallet address: {}", stored_wallet).into());
                    wallet_address.set(Some(stored_wallet));
                    loading.set(false);
                    return;
                }

                // Fallback: check if Freighter is available but don't auto-connect
                if !is_freighter_available() {
                    error_message.set(Some("Freighter wallet not found. Please install Freighter and connect via Login page.".to_string()));
                    loading.set(false);
                    return;
                }

                // If no stored wallet and Freighter is available, don't set error - let the no-wallet screen show
                loading.set(false);
                
                ()
            },
        );
    }

    // Show loading state
    if *loading {
        return html! {
            <div class="loading-wallet">
                <div class="spinner"></div>
                <h2>{"🔗 Connecting to Freighter..."}</h2>
                <p>{"Getting your wallet address..."}</p>
                <style>
                    {".loading-wallet {
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        justify-content: center;
                        height: 100vh;
                        padding: 2rem;
                        text-align: center;
                        background: #1a1a2e;
                        color: white;
                    }
                    .spinner {
                        width: 40px;
                        height: 40px;
                        border: 3px solid rgba(255, 255, 255, 0.3);
                        border-top: 3px solid #4f46e5;
                        border-radius: 50%;
                        animation: spin 1s linear infinite;
                        margin-bottom: 1rem;
                    }
                    @keyframes spin {
                        0% { transform: rotate(0deg); }
                        100% { transform: rotate(360deg); }
                    }"}
                </style>
            </div>
        };
    }

    // Show error state  
    if let Some(error) = (*error_message).clone() {
        return html! {
            <div class="no-wallet-message">
                <h2>{"❌ Wallet Connection Failed"}</h2>
                <p>{error}</p>
                <p>{"Please make sure Freighter is installed and connected."}</p>
                <a href="/" class="login-btn">{"← Back to Login"}</a>
                <style>
                    {".no-wallet-message {
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        justify-content: center;
                        height: 100vh;
                        padding: 2rem;
                        text-align: center;
                        background: #1a1a2e;
                        color: white;
                    }
                    .no-wallet-message h2 {
                        color: #f59e0b;
                        margin-bottom: 1rem;
                    }
                    .login-btn {
                        display: inline-block;
                        padding: 1rem 2rem;
                        background: linear-gradient(135deg, #4f46e5, #06b6d4);
                        color: white;
                        text-decoration: none;
                        border-radius: 12px;
                        margin-top: 2rem;
                        font-weight: 600;
                    }
                    .login-btn:hover {
                        transform: translateY(-2px);
                        box-shadow: 0 10px 25px rgba(79, 70, 229, 0.4);
                    }"}
                </style>
            </div>
        };
    }

    // Manual connect wallet callback
    let on_manual_connect = {
        let wallet_address = wallet_address.clone();
        let error_message = error_message.clone();
        let loading = loading.clone();
        
        Callback::from(move |_| {
            let wallet_address = wallet_address.clone();
            let error_message = error_message.clone();
            let loading = loading.clone();
            
            loading.set(true);
            error_message.set(None);
            
            wasm_bindgen_futures::spawn_local(async move {
                match connect_wallet().await {
                    Ok(address) => {
                        console::log_1(&format!("✅ Manually connected wallet: {}", address).into());
                        wallet_address.set(Some(address.clone()));
                        let _ = LocalStorage::set("wallet_address", &address);
                        error_message.set(None);
                    },
                    Err(e) => {
                        console::log_1(&format!("❌ Manual wallet connection failed: {}", e).into());
                        error_message.set(Some(format!("Failed to connect: {}", e)));
                    }
                }
                loading.set(false);
            });
        })
    };

    // Show no wallet state
    if wallet_address.is_none() {
        return html! {
            <div class="no-wallet-message">
                <h2>{"No Wallet Connected"}</h2>
                <p>{"Connect your Freighter wallet to continue."}</p>
                <div class="wallet-options">
                    <button 
                        class="connect-btn"
                        onclick={on_manual_connect}
                        disabled={*loading}
                    >
                        {
                            if *loading {
                                html! {
                                    <>
                                        <div class="spinner-small"></div>
                                        <span>{"Connecting..."}</span>
                                    </>
                                }
                            } else {
                                html! {
                                    <>
                                        <span>{"🔗 Connect Freighter"}</span>
                                    </>
                                }
                            }
                        }
                    </button>
                    <a href="/" class="login-btn">{"← Back to Login"}</a>
                </div>
                <style>
                    {".no-wallet-message {
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        justify-content: center;
                        height: 100vh;
                        padding: 2rem;
                        text-align: center;
                        background: #1a1a2e;
                        color: white;
                    }
                    .no-wallet-message h2 {
                        color: #f59e0b;
                        margin-bottom: 1rem;
                    }
                    .wallet-options {
                        display: flex;
                        flex-direction: column;
                        gap: 1rem;
                        margin-top: 2rem;
                    }
                    .connect-btn {
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        gap: 0.5rem;
                        padding: 1rem 2rem;
                        background: linear-gradient(135deg, #8b5cf6, #a855f7);
                        color: white;
                        border: none;
                        border-radius: 12px;
                        font-weight: 600;
                        cursor: pointer;
                        transition: all 0.3s ease;
                    }
                    .connect-btn:hover:not(:disabled) {
                        transform: translateY(-2px);
                        box-shadow: 0 10px 25px rgba(139, 92, 246, 0.4);
                    }
                    .connect-btn:disabled {
                        opacity: 0.6;
                        cursor: not-allowed;
                    }
                    .login-btn {
                        display: inline-block;
                        padding: 1rem 2rem;
                        background: linear-gradient(135deg, #4f46e5, #06b6d4);
                        color: white;
                        text-decoration: none;
                        border-radius: 12px;
                        font-weight: 600;
                    }
                    .login-btn:hover {
                        transform: translateY(-2px);
                        box-shadow: 0 10px 25px rgba(79, 70, 229, 0.4);
                    }
                    .spinner-small {
                        width: 16px;
                        height: 16px;
                        border: 2px solid rgba(255, 255, 255, 0.3);
                        border-top: 2px solid white;
                        border-radius: 50%;
                        animation: spin 1s linear infinite;
                    }"}
                </style>
            </div>
        };
    }

    let wallet_addr = wallet_address.as_ref().unwrap();
    
    // Log debug info
    console::log_1(&format!("GamePage loaded - Username: {}, Wallet: {}", username.as_str(), wallet_addr).into());
    
    use_effect(|| {
        console::log_1(&"GamePage component mounted".into());
        || console::log_1(&"GamePage component unmounted".into())
    });


    let on_join_contract = {
        let wallet_addr = wallet_addr.clone();
        let username = username.clone();
        let joining_contract = joining_contract.clone();
        let join_result = join_result.clone();
        
        Callback::from(move |_| {
            let wallet_addr = wallet_addr.clone();
            let username_val = username.as_str().to_string();
            let joining_contract = joining_contract.clone();
            let join_result = join_result.clone();
            
            joining_contract.set(true);
            join_result.set(None);
            
            wasm_bindgen_futures::spawn_local(async move {
                console::log_1(&format!("🚀 Starting contract join for wallet: {}", wallet_addr).into());
                
                match complete_join_flow(&wallet_addr, &username_val).await {
                    Ok(result) => {
                        console::log_1(&format!("✅ Successfully joined contract! Hash: {}", result.hash).into());
                        join_result.set(Some(format!("✅ Successfully joined! Tx: {}...{}", &result.hash[0..8], &result.hash[result.hash.len()-8..])));
                    },
                    Err(e) => {
                        console::log_1(&format!("❌ Failed to join contract: {}", e).into());
                        join_result.set(Some(format!("❌ Failed to join: {}", e)));
                    }
                }
                joining_contract.set(false);
            });
        })
    };

    html! {
        <div class="game-container">
            <div class="header">
                <h1>{"🌟 Stellar Heads"}</h1>
                <div class="user-info">
                    <span>{format!("Player: {}", username.as_str())}</span>
                    <span class="wallet-addr">{format!("Wallet: {}...{}", 
                        &wallet_addr[0..std::cmp::min(6, wallet_addr.len())],
                        if wallet_addr.len() > 6 { &wallet_addr[wallet_addr.len()-4..] } else { "" }
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
                <div class="main-section">
                    <h2>{"🌟 Stellar Heads Game"}</h2>
                    <p class="player-info">{format!("Player: {} | Wallet: {}...{}", 
                        username.as_str(), 
                        &wallet_addr[0..std::cmp::min(6, wallet_addr.len())],
                        if wallet_addr.len() > 8 { &wallet_addr[wallet_addr.len()-4..] } else { "" }
                    )}</p>
                    
                    {
                        if let Some(result) = (*join_result).clone() {
                            html! {
                                <div class="join-result">
                                    <p>{result}</p>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                    
                    <button 
                        class="join-contract-btn"
                        onclick={on_join_contract}
                        disabled={*joining_contract}
                    >
                        {
                            if *joining_contract {
                                html! {
                                    <>
                                        <div class="spinner-small"></div>
                                        <span>{"Joining Contract..."}</span>
                                    </>
                                }
                            } else {
                                html! {
                                    <>
                                        <span>{"🚀 Join Leaderboard Contract"}</span>
                                    </>
                                }
                            }
                        }
                    </button>
                    
                    <p class="contract-info">{"Join the Stellar contract to compete on the leaderboard!"}</p>
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
                    padding: 2rem;
                    text-align: center;
                }

                .main-section {
                    background: linear-gradient(135deg, #4f46e5 0%, #06b6d4 100%);
                    color: white;
                    padding: 2rem;
                    border-radius: 16px;
                    text-align: center;
                    max-width: 500px;
                    margin: 0 auto;
                }

                .main-section h2 {
                    margin: 0 0 1rem 0;
                    font-size: 1.8rem;
                    font-weight: 700;
                }

                .player-info {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 8px;
                    padding: 1rem;
                    margin-bottom: 1.5rem;
                    font-family: 'Monaco', 'Consolas', monospace;
                    font-size: 0.9rem;
                }

                .contract-info {
                    margin-top: 1rem;
                    opacity: 0.9;
                    font-size: 0.9rem;
                }

                .join-contract-btn {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 0.5rem;
                    width: 100%;
                    padding: 1.25rem 2rem;
                    background: rgba(255, 255, 255, 0.2);
                    color: white;
                    border: 2px solid rgba(255, 255, 255, 0.3);
                    border-radius: 12px;
                    font-weight: 600;
                    font-size: 1.1rem;
                    cursor: pointer;
                    transition: all 0.3s ease;
                }

                .join-contract-btn:hover:not(:disabled) {
                    background: rgba(255, 255, 255, 0.3);
                    border-color: rgba(255, 255, 255, 0.5);
                    transform: translateY(-2px);
                    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.3);
                }

                .join-contract-btn:disabled {
                    opacity: 0.6;
                    cursor: not-allowed;
                }

                .spinner-small {
                    width: 16px;
                    height: 16px;
                    border: 2px solid rgba(255, 255, 255, 0.3);
                    border-top: 2px solid white;
                    border-radius: 50%;
                    animation: spin 1s linear infinite;
                }

                .join-result {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 8px;
                    padding: 1rem;
                    margin-bottom: 1rem;
                    font-family: 'Monaco', 'Consolas', monospace;
                    font-size: 0.9rem;
                }
                "#}
            </style>
        </div>
    }
}