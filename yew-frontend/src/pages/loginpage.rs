use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::{InputEvent, SubmitEvent};
use gloo::storage::{LocalStorage, Storage};
use crate::routing::Route;
use crate::wallet::connect_wallet;
// Removed unused import: use crate::soroban::complete_join_flow;
use crate::services::ApiClient;
use shared::dto::auth::Guest;

#[derive(Debug, Clone, PartialEq)]
enum LoginStep {
    Username,
    WalletConnection,
    JoiningContract,
    Complete,
}


#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let navigator = match use_navigator() {
        Some(nav) => nav,
        None => {
            web_sys::console::log_1(&"Failed to get navigator".into());
            return html! { <div>{"Navigation error"}</div> };
        }
    };
    let username = use_state(String::new);
    let wallet_address = use_state(|| None::<String>);
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let freighter_available = use_state(|| false);
 
    let current_step = use_state(|| LoginStep::Username);
    let show_step_animation = use_state(|| false);

    // Wallet availability check on mount
    {
        let freighter_available = freighter_available.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local({
                let freighter_available = freighter_available.clone();
                async move {
                    // Check Freighter immediately
                    if crate::wallet::is_freighter_available().await {
                        freighter_available.set(true);
                    } else {
                        // Wait a moment for extension to load, then check once more
                        gloo::timers::future::sleep(std::time::Duration::from_millis(1000)).await;
                        freighter_available.set(crate::wallet::is_freighter_available().await);
                    }
                }
            });
            || ()
        });
    }

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

    // Freighter wallet connection handler
    let connect_freighter_wallet = {
        let loading = loading.clone();
        let error_message = error_message.clone();
        let wallet_address = wallet_address.clone();
        let current_step = current_step.clone();
        let show_step_animation = show_step_animation.clone();
        let username = username.clone();

        move || {
            let loading = loading.clone();
            let error_message = error_message.clone();
            let wallet_address = wallet_address.clone();
            let current_step = current_step.clone();
            let show_step_animation = show_step_animation.clone();
            let username = username.clone();

            loading.set(true);
            error_message.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                web_sys::console::log_1(&"🔗 Starting wallet connection process...".into());
                let connect_result = connect_wallet().await.map_err(|e| e.to_string());

                match connect_result {
                    Ok(address) => {
                        wallet_address.set(Some(address.clone()));
                        web_sys::console::log_1(&format!("Freighter wallet connected: {address}").into());

                        let username_val = username.as_str().to_string();

                        // Save user to backend database
                        let api_client = ApiClient::new();
                        let guest = Guest {
                            username: username_val.clone(),
                            wallet_address: address.clone(),
                        };

                        web_sys::console::log_1(&"💾 Attempting to save user to backend...".into());
                        match api_client.register_guest(guest).await {
                            Ok(response) => {
                                web_sys::console::log_1(&format!("✅ User saved to backend: {}", response.message).into());

                                // Store wallet address and user data locally
                                let _ = LocalStorage::set("wallet_address", &address);
                                let _ = LocalStorage::set("username", &username_val);
                                let _ = LocalStorage::set("user_id", &response.user.id);

                                // Complete the login flow
                                show_step_animation.set(true);
                                gloo::timers::callback::Timeout::new(300, {
                                    let current_step = current_step.clone();
                                    let show_step_animation = show_step_animation.clone();
                                    let loading = loading.clone();
                                    move || {
                                        current_step.set(LoginStep::Complete);
                                        show_step_animation.set(false);
                                        loading.set(false);
                                    }
                                }).forget();
                            }
                            Err(backend_err) => {
                                web_sys::console::log_1(&format!("❌ Backend save failed: {backend_err}").into());

                                // Check if it's a network connection issue to backend
                                if backend_err.contains("Network error") || backend_err.contains("Failed to fetch") {
                                    error_message.set(Some("Cannot connect to backend server. Please ensure the server is running on localhost:3000".to_string()));
                                } else {
                                    error_message.set(Some(format!("Failed to save user: {backend_err}")));
                                }
                                loading.set(false);
                            }
                        }
                    }
                    Err(err) => {
                        web_sys::console::log_1(&format!("❌ Freighter connection failed: {err}").into());

                        // Provide more helpful error messages to user
                        let user_message = match err.as_str() {
                            msg if msg.contains("User rejected") => {
                                "Connection cancelled. Please click 'Connect' and approve the request in Freighter.".to_string()
                            }
                            msg if msg.contains("Freighter") && msg.contains("not found") => {
                                "Freighter wallet not found. Please install Freighter extension and refresh the page.".to_string()
                            }
                            _ => {
                                format!("Freighter connection failed: {err}")
                            }
                        };

                        error_message.set(Some(user_message));
                        loading.set(false);
                    }
                }
            });
        }
    };

    let on_connect_wallet = {
        let connect_fn = connect_freighter_wallet.clone();
        Callback::from(move |_| {
            connect_fn();
        })
    };


    let _on_enter_game = {
        let username = username.clone();
        let wallet_address = wallet_address.clone();
        let navigator = navigator.clone();
        let loading = loading.clone();
        let current_step = current_step.clone();
        let error_message = error_message.clone();
        
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            if !username.is_empty() && wallet_address.is_some() {
                let navigator = navigator.clone();
                let username_val = (*username).clone();
                let wallet_addr = match wallet_address.as_ref() {
                    Some(addr) => addr.clone(),
                    None => {
                        error_message.set(Some("No wallet address available".to_string()));
                        return;
                    }
                };
                let loading = loading.clone();
                let current_step = current_step.clone();
                let _error_message = error_message.clone();
                
                loading.set(true);
                
                // Update step to show joining contract
                current_step.set(LoginStep::JoiningContract);
                
                // Skip contract joining - just complete the login
                LocalStorage::set("username", &username_val).ok();
                LocalStorage::set("wallet_address", &wallet_addr).ok();

                current_step.set(LoginStep::Complete);
                loading.set(false);

                // Navigate to game
                navigator.push(&Route::Game);
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
                        <div class={format!("step-dot {}", if *current_step == LoginStep::Username { "active" } else if matches!(*current_step, LoginStep::WalletConnection | LoginStep::Complete) { "completed" } else { "" })}>
                            <span>{"1"}</span>
                        </div>
                        <div class="step-line"></div>
                        <div class={format!("step-dot {}", if *current_step == LoginStep::WalletConnection { "active" } else if *current_step == LoginStep::Complete { "completed" } else { "" })}>
                            <span>{"2"}</span>
                        </div>
                        <div class="step-line"></div>
                        <div class={format!("step-dot {}", if *current_step == LoginStep::Complete { "active" } else { "" })}>
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
                                    <p class="step-subtitle">{format!("Hello {}, connect your Freighter wallet", (*username).clone())}</p>

                                    <div class="wallet-options">
                                        // Freighter wallet option
                                        <div class="wallet-connect">
                                            <div class="wallet-icon">{"🚀"}</div>
                                            <div class="wallet-info">
                                                <h3>{"Freighter"}</h3>
                                                <p>{"Official Stellar Development Foundation wallet"}</p>
                                            </div>
                                            {
                                                if !*freighter_available {
                                                    html! {
                                                        <button class="wallet-btn disabled" disabled=true>
                                                            <span>{"Not Installed"}</span>
                                                        </button>
                                                    }
                                                } else {
                                                    html! {
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
                                                                            <span>{"Connect"}</span>
                                                                            <span class="wallet-arrow">{"🔗"}</span>
                                                                        </>
                                                                    }
                                                                }
                                                            }
                                                        </button>
                                                    }
                                                }
                                            }
                                        </div>

                                    </div>

                                    {
                                        if !*freighter_available {
                                            html! {
                                                <div class="wallet-status">
                                                    <p>{"Freighter wallet not detected. Please install Freighter and refresh."}</p>
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>
                            },
                            LoginStep::JoiningContract => html! {
                                <div class="step-panel fade-in">
                                    <div class="loading-animation">
                                        <div class="spinner"></div>
                                    </div>
                                    
                                    <h2 class="step-title">{"Joining Contract"}</h2>
                                    <p class="step-subtitle">{"Creating transaction and signing with Freighter..."}</p>
                                    
                                    <div class="progress-steps">
                                        <div class="progress-step active">{"1. Creating transaction"}</div>
                                        <div class="progress-step active">{"2. Signing with Freighter"}</div>
                                        <div class="progress-step">{"3. Submitting to blockchain"}</div>
                                    </div>
                                </div>
                            },
                            LoginStep::Complete => html! {
                                <div class="step-panel fade-in">
                                    <div class="success-animation">
                                        <div class="success-circle">
                                            <div class="checkmark">{"✓"}</div>
                                        </div>
                                    </div>
                                    
                                    <h2 class="step-title">{"Welcome to Stellar Heads!"}</h2>
                                    <p class="step-subtitle">{format!("Successfully set up, {}!", (*username).clone())}</p>
                                    
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
                                    
                                    {
                                        if let Some(error) = (*error_message).clone() {
                                            html! {
                                                <div class="error-message">
                                                    <div class="error-icon">{"⚠️"}</div>
                                                    <p>{error}</p>
                                                    <small>{"You're still logged in locally"}</small>
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <div class="success-message">
                                                    <div class="success-icon">{"🎉"}</div>
                                                    <p>{"Successfully connected wallet!"}</p>
                                                    <small>{"Ready to play Stellar Heads"}</small>
                                                </div>
                                            }
                                        }
                                    }
                                    
                                    <button 
                                        class="launch-btn"
                                        onclick={{
                                            let navigator = navigator.clone();
                                            Callback::from(move |_| {
                                                navigator.push(&Route::Game);
                                            })
                                        }}
                                    >
                                        <span>{"🚀 Enter Game"}</span>
                                    </button>
                                </div>
                            }
                        }
                    }
                </div>
            </div>
        </div>
    }
}