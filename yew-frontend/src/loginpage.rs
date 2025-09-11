use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::{HtmlInputElement, InputEvent, SubmitEvent};
use gloo::storage::{LocalStorage, Storage};
use crate::routes::Route;
use crate::freighter::{connect_wallet, is_freighter_available, FreighterError};

#[derive(Debug, Clone, PartialEq)]
enum LoginStep {
    Username,
    WalletConnection,
    Ready,
}

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let navigator = use_navigator().unwrap();
    let username = use_state(String::new);
    let wallet_address = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let freighter_available = use_state(|| is_freighter_available());
    let current_step = use_state(|| LoginStep::Username);
    let show_step_animation = use_state(|| false);

    let on_username = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let on_username_continue = {
        let current_step = current_step.clone();
        let show_step_animation = show_step_animation.clone();
        Callback::from(move |_| {
            show_step_animation.set(true);
            gloo::timers::callback::Timeout::new(300, {
                let current_step = current_step.clone();
                let show_step_animation = show_step_animation.clone();
                move || {
                    current_step.set(LoginStep::WalletConnection);
                    show_step_animation.set(false);
                }
            }).forget();
        })
    };

    let on_connect_wallet = {
        let loading = loading.clone();
        let error_message = error_message.clone();
        let wallet_address = wallet_address.clone();
        let current_step = current_step.clone();
        let show_step_animation = show_step_animation.clone();
        
        Callback::from(move |_| {
            let loading = loading.clone();
            let error_message = error_message.clone();
            let wallet_address = wallet_address.clone();
            let current_step = current_step.clone();
            let show_step_animation = show_step_animation.clone();
            
            loading.set(true);
            error_message.set(None);
            
            wasm_bindgen_futures::spawn_local(async move {
                match connect_wallet().await {
                    Ok(address) => {
                        wallet_address.set(Some(address.clone()));
                        web_sys::console::log_1(&format!("Wallet connected: {}", address).into());
                        
                        // Animate to ready step
                        show_step_animation.set(true);
                        gloo::timers::callback::Timeout::new(500, {
                            let current_step = current_step.clone();
                            let show_step_animation = show_step_animation.clone();
                            move || {
                                current_step.set(LoginStep::Ready);
                                show_step_animation.set(false);
                            }
                        }).forget();
                    }
                    Err(err) => {
                        error_message.set(Some(err.to_string()));
                        web_sys::console::log_1(&format!("Wallet connection failed: {}", err).into());
                    }
                }
                loading.set(false);
            });
        })
    };

    let on_enter_game = {
        let username = username.clone();
        let wallet_address = wallet_address.clone();
        let navigator = navigator.clone();
        let loading = loading.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            if !username.is_empty() && wallet_address.is_some() {
                let navigator = navigator.clone();
                let username_val = (*username).clone();
                let wallet_addr = wallet_address.as_ref().unwrap().clone();
                let loading = loading.clone();
                
                loading.set(true);
                
                // Send to backend API first
                wasm_bindgen_futures::spawn_local(async move {
                    let guest_data = serde_json::json!({
                        "username": username_val,
                        "wallet_address": wallet_addr
                    });
                    
                    let client = reqwest::Client::new();
                    match client
                        .post("http://localhost:3000/api/guest")
                        .header("Content-Type", "application/json")
                        .body(guest_data.to_string())
                        .send()
                        .await
                    {
                        Ok(response) => {
                            if response.status().is_success() {
                                web_sys::console::log_1(&"✅ User registered successfully in database!".into());
                                
                                // Store locally for session management
                                LocalStorage::set("username", &username_val).ok();
                                LocalStorage::set("wallet_address", &wallet_addr).ok();
                                
                                // Navigate to game
                                navigator.push(&Route::Game);
                            } else {
                                web_sys::console::log_1(&format!("❌ Backend error: {}", response.status()).into());
                                // Still allow game access but log the error
                                LocalStorage::set("username", &username_val).ok();
                                LocalStorage::set("wallet_address", &wallet_addr).ok();
                                navigator.push(&Route::Game);
                            }
                        },
                        Err(e) => {
                            web_sys::console::log_1(&format!("❌ Failed to register user: {:?}", e).into());
                            // Still allow game access but log the error
                            LocalStorage::set("username", &username_val).ok();
                            LocalStorage::set("wallet_address", &wallet_addr).ok();
                            navigator.push(&Route::Game);
                        }
                    }
                    
                    loading.set(false);
                });
            }
        })
    };

    html! {
        <div class="stellar-login">
            <div class="cosmic-bg"></div>
            
            <div class={format!("login-card {}", if *show_step_animation { "transitioning" } else { "" })}>
                <div class="header">
                    <h1 class="title">{"🌟 Stellar Heads"}</h1>
                    <div class="step-indicator">
                        <div class={format!("step-dot {}", if *current_step == LoginStep::Username { "active" } else if matches!(*current_step, LoginStep::WalletConnection | LoginStep::Ready) { "completed" } else { "" })}>
                            <span>{"1"}</span>
                        </div>
                        <div class="step-line"></div>
                        <div class={format!("step-dot {}", if *current_step == LoginStep::WalletConnection { "active" } else if *current_step == LoginStep::Ready { "completed" } else { "" })}>
                            <span>{"2"}</span>
                        </div>
                        <div class="step-line"></div>
                        <div class={format!("step-dot {}", if *current_step == LoginStep::Ready { "active" } else { "" })}>
                            <span>{"🚀"}</span>
                        </div>
                    </div>
                </div>

                // Error message with slide animation
                {
                    if let Some(error) = (*error_message).clone() {
                        html! {
                            <div class="error-slide">
                                <div class="error-content">
                                    <span class="error-icon">{"⚠️"}</span>
                                    <span>{error}</span>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                <div class="step-content">
                    {
                        match &*current_step {
                            LoginStep::Username => html! {
                                <div class="step-panel fade-in">
                                    <h2 class="step-title">{"Choose Your Identity"}</h2>
                                    <p class="step-subtitle">{"Enter a username to begin your stellar journey"}</p>
                                    
                                    <div class="input-group">
                                        <div class="input-wrapper">
                                            <input
                                                type="text"
                                                placeholder="Your stellar name..."
                                                oninput={on_username}
                                                value={(*username).clone()}
                                                class="stellar-input"
                                                maxlength="20"
                                            />
                                            <div class="input-glow"></div>
                                        </div>
                                    </div>
                                    
                                    <button 
                                        class="primary-btn"
                                        onclick={on_username_continue}
                                        disabled={username.is_empty()}
                                    >
                                        <span class="btn-text">{"Continue"}</span>
                                        <span class="btn-arrow">{"→"}</span>
                                    </button>
                                </div>
                            },
                            LoginStep::WalletConnection => html! {
                                <div class="step-panel fade-in">
                                    <h2 class="step-title">{"Connect Your Wallet"}</h2>
                                    <p class="step-subtitle">{format!("Hello {}, let's connect your Stellar wallet", (*username).clone())}</p>
                                    
                                    {
                                        if !*freighter_available {
                                            html! {
                                                <div class="wallet-not-found">
                                                    <div class="wallet-icon">{"🌌"}</div>
                                                    <h3>{"Freighter Wallet Required"}</h3>
                                                    <p>{"Install the Freighter browser extension to continue"}</p>
                                                    <a href="https://freighter.app/" target="_blank" class="install-btn">
                                                        {"Install Freighter"}
                                                    </a>
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <div class="wallet-connect">
                                                    <div class="wallet-icon">{"🛸"}</div>
                                                    <button 
                                                        class="wallet-btn"
                                                        onclick={on_connect_wallet}
                                                        disabled={*loading}
                                                    >
                                                        {
                                                            if *loading {
                                                                html! {
                                                                    <>
                                                                        <div class="spinner"></div>
                                                                        <span>{"Connecting..."}</span>
                                                                    </>
                                                                }
                                                            } else {
                                                                html! {
                                                                    <>
                                                                        <span>{"Connect Freighter"}</span>
                                                                        <span class="wallet-arrow">{"🔗"}</span>
                                                                    </>
                                                                }
                                                            }
                                                        }
                                                    </button>
                                                </div>
                                            }
                                        }
                                    }
                                </div>
                            },
                            LoginStep::Ready => html! {
                                <div class="step-panel fade-in">
                                    <div class="success-animation">
                                        <div class="success-circle">
                                            <div class="checkmark">{"✓"}</div>
                                        </div>
                                    </div>
                                    
                                    <h2 class="step-title">{"Ready for Launch!"}</h2>
                                    <p class="step-subtitle">{format!("Welcome aboard, {}!", (*username).clone())}</p>
                                    
                                    <div class="wallet-info">
                                        <div class="wallet-badge">
                                            <span class="wallet-label">{"Connected:"}</span>
                                            <span class="wallet-address">{
                                                if let Some(addr) = wallet_address.as_ref() {
                                                    format!("{}...{}", &addr[0..4], &addr[addr.len()-4..])
                                                } else {
                                                    "Unknown".to_string()
                                                }
                                            }</span>
                                        </div>
                                    </div>
                                    
                                    <form onsubmit={on_enter_game}>
                                        <button type="submit" class="launch-btn">
                                            <span class="btn-text">{"Enter Game"}</span>
                                            <span class="btn-rocket">{"🚀"}</span>
                                        </button>
                                    </form>
                                </div>
                            }
                        }
                    }
                </div>
            </div>
            
            // Modern CSS with animations
            <style>
                {r#"
                .stellar-login {
                    min-height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    padding: 2rem;
                    position: relative;
                    background: linear-gradient(135deg, #0d1117 0%, #1a1a2e 50%, #16213e 100%);
                    overflow: hidden;
                }

                .cosmic-bg {
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: 
                        radial-gradient(circle at 20% 80%, rgba(120, 119, 198, 0.3) 0%, transparent 50%),
                        radial-gradient(circle at 80% 20%, rgba(255, 119, 198, 0.3) 0%, transparent 50%),
                        radial-gradient(circle at 40% 40%, rgba(120, 219, 255, 0.2) 0%, transparent 50%);
                    animation: cosmic-drift 20s ease-in-out infinite;
                }

                @keyframes cosmic-drift {
                    0%, 100% { transform: translate(0, 0) rotate(0deg); }
                    50% { transform: translate(-20px, -10px) rotate(1deg); }
                }

                .login-card {
                    background: rgba(255, 255, 255, 0.05);
                    backdrop-filter: blur(20px);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 24px;
                    padding: 3rem;
                    width: 100%;
                    max-width: 480px;
                    box-shadow: 
                        0 20px 40px rgba(0, 0, 0, 0.4),
                        inset 0 1px 0 rgba(255, 255, 255, 0.1);
                    position: relative;
                    transition: all 0.6s cubic-bezier(0.4, 0, 0.2, 1);
                }

                .login-card.transitioning {
                    transform: scale(0.95);
                    opacity: 0.8;
                }

                .header {
                    text-align: center;
                    margin-bottom: 3rem;
                }

                .title {
                    font-size: 2.5rem;
                    font-weight: 700;
                    background: linear-gradient(135deg, #4f46e5, #06b6d4, #8b5cf6);
                    background-size: 200% 200%;
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    animation: gradient-shift 3s ease-in-out infinite;
                    margin-bottom: 2rem;
                }

                @keyframes gradient-shift {
                    0%, 100% { background-position: 0% 50%; }
                    50% { background-position: 100% 50%; }
                }

                .step-indicator {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 1rem;
                }

                .step-dot {
                    width: 40px;
                    height: 40px;
                    border-radius: 50%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-weight: 600;
                    font-size: 0.9rem;
                    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
                    background: rgba(255, 255, 255, 0.1);
                    color: rgba(255, 255, 255, 0.5);
                    border: 2px solid rgba(255, 255, 255, 0.2);
                }

                .step-dot.active {
                    background: linear-gradient(135deg, #4f46e5, #06b6d4);
                    color: white;
                    border-color: transparent;
                    box-shadow: 0 8px 20px rgba(79, 70, 229, 0.4);
                    transform: scale(1.1);
                }

                .step-dot.completed {
                    background: linear-gradient(135deg, #10b981, #059669);
                    color: white;
                    border-color: transparent;
                }

                .step-line {
                    width: 60px;
                    height: 2px;
                    background: rgba(255, 255, 255, 0.2);
                    transition: all 0.4s ease;
                }

                .step-content {
                    min-height: 300px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }

                .step-panel {
                    width: 100%;
                    text-align: center;
                }

                .fade-in {
                    animation: fade-in 0.6s cubic-bezier(0.4, 0, 0.2, 1);
                }

                @keyframes fade-in {
                    from { opacity: 0; transform: translateY(20px); }
                    to { opacity: 1; transform: translateY(0); }
                }

                .step-title {
                    font-size: 1.8rem;
                    font-weight: 600;
                    color: white;
                    margin-bottom: 0.5rem;
                }

                .step-subtitle {
                    color: rgba(255, 255, 255, 0.7);
                    margin-bottom: 2rem;
                    font-size: 1rem;
                }

                .input-group {
                    margin-bottom: 2rem;
                }

                .input-wrapper {
                    position: relative;
                }

                .stellar-input {
                    width: 100%;
                    padding: 1rem 1.5rem;
                    font-size: 1.1rem;
                    background: rgba(255, 255, 255, 0.05);
                    border: 2px solid rgba(255, 255, 255, 0.1);
                    border-radius: 16px;
                    color: white;
                    transition: all 0.3s ease;
                    outline: none;
                }

                .stellar-input::placeholder {
                    color: rgba(255, 255, 255, 0.4);
                }

                .stellar-input:focus {
                    border-color: #4f46e5;
                    box-shadow: 0 0 0 4px rgba(79, 70, 229, 0.2);
                }

                .primary-btn, .wallet-btn, .launch-btn {
                    width: 100%;
                    padding: 1rem 2rem;
                    font-size: 1.1rem;
                    font-weight: 600;
                    border: none;
                    border-radius: 16px;
                    cursor: pointer;
                    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 0.5rem;
                    position: relative;
                    overflow: hidden;
                }

                .primary-btn {
                    background: linear-gradient(135deg, #4f46e5, #06b6d4);
                    color: white;
                }

                .primary-btn:hover:not(:disabled) {
                    transform: translateY(-2px);
                    box-shadow: 0 10px 25px rgba(79, 70, 229, 0.4);
                }

                .primary-btn:disabled {
                    opacity: 0.5;
                    cursor: not-allowed;
                }

                .wallet-btn {
                    background: linear-gradient(135deg, #8b5cf6, #a855f7);
                    color: white;
                }

                .wallet-btn:hover:not(:disabled) {
                    transform: translateY(-2px);
                    box-shadow: 0 10px 25px rgba(139, 92, 246, 0.4);
                }

                .launch-btn {
                    background: linear-gradient(135deg, #f59e0b, #d97706);
                    color: white;
                    font-size: 1.2rem;
                    padding: 1.2rem 2rem;
                }

                .launch-btn:hover {
                    transform: translateY(-2px);
                    box-shadow: 0 15px 30px rgba(245, 158, 11, 0.4);
                }

                .wallet-connect {
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    gap: 1.5rem;
                }

                .wallet-icon {
                    font-size: 4rem;
                    animation: float 3s ease-in-out infinite;
                }

                @keyframes float {
                    0%, 100% { transform: translateY(0); }
                    50% { transform: translateY(-10px); }
                }

                .spinner {
                    width: 20px;
                    height: 20px;
                    border: 2px solid rgba(255, 255, 255, 0.3);
                    border-top: 2px solid white;
                    border-radius: 50%;
                    animation: spin 1s linear infinite;
                }

                @keyframes spin {
                    to { transform: rotate(360deg); }
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

                .wallet-info {
                    margin-bottom: 2rem;
                }

                .wallet-badge {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 12px;
                    padding: 1rem;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                }

                .wallet-label {
                    color: rgba(255, 255, 255, 0.7);
                    font-size: 0.9rem;
                }

                .wallet-address {
                    color: #06b6d4;
                    font-family: 'Monaco', 'Consolas', monospace;
                    font-weight: 600;
                }

                .error-slide {
                    margin-bottom: 1rem;
                    animation: slide-down 0.4s cubic-bezier(0.4, 0, 0.2, 1);
                }

                @keyframes slide-down {
                    from { transform: translateY(-20px); opacity: 0; }
                    to { transform: translateY(0); opacity: 1; }
                }

                .error-content {
                    background: rgba(239, 68, 68, 0.1);
                    border: 1px solid rgba(239, 68, 68, 0.3);
                    border-radius: 12px;
                    padding: 1rem;
                    display: flex;
                    align-items: center;
                    gap: 0.5rem;
                    color: #fca5a5;
                }

                .wallet-not-found {
                    text-align: center;
                    padding: 2rem;
                }

                .install-btn {
                    display: inline-block;
                    padding: 0.75rem 1.5rem;
                    background: linear-gradient(135deg, #4f46e5, #06b6d4);
                    color: white;
                    text-decoration: none;
                    border-radius: 12px;
                    font-weight: 600;
                    margin-top: 1rem;
                    transition: all 0.3s ease;
                }

                .install-btn:hover {
                    transform: translateY(-2px);
                    box-shadow: 0 10px 25px rgba(79, 70, 229, 0.4);
                }
                "#}
            </style>
        </div>
    }
}