use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use web_sys::console;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use crate::soroban::sign_transaction_with_freighter;

#[derive(Debug, Serialize)]
struct TestContractRequest {
    player_address: String,
    username: String,
    action: String,
}

#[derive(Debug, Deserialize)]
struct TestContractResponse {
    transaction_xdr: String,
    network_passphrase: String,
    success: bool,
    message: String,
    estimated_fee: Option<u64>,
}

fn create_test_callback(
    testing: UseStateHandle<bool>,
    result_message: UseStateHandle<Option<String>>,
    last_transaction: UseStateHandle<Option<String>>,
    action: &'static str,
    loading_message: &'static str,
) -> Callback<yew::MouseEvent> {
    Callback::from(move |_| {
        let testing = testing.clone();
        let result_message = result_message.clone();
        let last_transaction = last_transaction.clone();

        let username = LocalStorage::get::<String>("username")
            .unwrap_or_else(|_| "TestPlayer".to_string());
        let wallet_address = LocalStorage::get::<String>("wallet_address")
            .unwrap_or_else(|_| "".to_string());

        if wallet_address.is_empty() {
            result_message.set(Some("❌ No wallet connected!".to_string()));
            return;
        }

        testing.set(true);
        result_message.set(Some(loading_message.to_string()));

        spawn_local(async move {
            let request = TestContractRequest {
                player_address: wallet_address.clone(),
                username: username.clone(),
                action: action.to_string(),
            };

            match Request::post("http://localhost:3000/api/soroban/test-signing")
                .header("Content-Type", "application/json")
                .json(&request)
                .unwrap()
                .send()
                .await
            {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<TestContractResponse>().await {
                            Ok(contract_response) => {
                                result_message.set(Some(format!("📋 {}", contract_response.message)));

                                match sign_transaction_with_freighter(
                                    &contract_response.transaction_xdr,
                                    &contract_response.network_passphrase,
                                ).await {
                                    Ok(signed) => {
                                        last_transaction.set(Some(signed.signed_xdr.clone()));
                                        result_message.set(Some(format!("🎉 {} signed successfully!", action.replace("_", " "))));
                                    },
                                    Err(e) => {
                                        result_message.set(Some(format!("❌ Signing failed: {}", e)));
                                    }
                                }
                            },
                            Err(e) => {
                                result_message.set(Some(format!("❌ Parse error: {}", e)));
                            }
                        }
                    } else {
                        result_message.set(Some(format!("❌ Backend error: {}", response.status())));
                    }
                },
                Err(e) => {
                    result_message.set(Some(format!("❌ Request failed: {}", e)));
                }
            }

            testing.set(false);
        });
    })
}

#[function_component(ContractTest)]
pub fn contract_test() -> Html {
    let testing = use_state(|| false);
    let result_message = use_state(|| None::<String>);
    let last_transaction = use_state(|| None::<String>);

    let test_join_leaderboard = create_test_callback(
        testing.clone(),
        result_message.clone(),
        last_transaction.clone(),
        "join_leaderboard", 
        "🚀 Creating leaderboard join transaction...",
    );

    let test_score_submission = create_test_callback(
        testing.clone(),
        result_message.clone(), 
        last_transaction.clone(),
        "test_score",
        "🎯 Creating test score transaction...",
    );

    let initialize_contract = create_test_callback(
        testing.clone(),
        result_message.clone(),
        last_transaction.clone(), 
        "initialize_contract",
        "🏗️ Creating contract initialization transaction...",
    );

    html! {
        <div class="contract-test-panel">
            <div class="test-header">
                <h3>{"🧪 Contract Signing Test"}</h3>
                <p>{"Test your real leaderboard contract with Freighter wallet"}</p>
            </div>

            <div class="test-buttons">
                <button 
                    class="test-btn join-btn"
                    onclick={test_join_leaderboard}
                    disabled={*testing}
                >
                    {
                        if *testing {
                            html! { <>{"⏳"}<span>{"Testing..."}</span></> }
                        } else {
                            html! { <>{"🌟"}<span>{"Join Leaderboard"}</span></> }
                        }
                    }
                </button>

                <button 
                    class="test-btn score-btn"
                    onclick={test_score_submission}
                    disabled={*testing}
                >
                    {
                        if *testing {
                            html! { <>{"⏳"}<span>{"Testing..."}</span></> }
                        } else {
                            html! { <>{"🎯"}<span>{"Test Score (1000)"}</span></> }
                        }
                    }
                </button>

                <button 
                    class="test-btn init-btn"
                    onclick={initialize_contract}
                    disabled={*testing}
                >
                    {
                        if *testing {
                            html! { <>{"⏳"}<span>{"Testing..."}</span></> }
                        } else {
                            html! { <>{"🏗️"}<span>{"Initialize Contract"}</span></> }
                        }
                    }
                </button>
            </div>

            {
                if let Some(message) = (*result_message).clone() {
                    html! {
                        <div class="test-result">
                            <pre>{message}</pre>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            {
                if let Some(xdr) = (*last_transaction).clone() {
                    html! {
                        <div class="transaction-details">
                            <h4>{"📋 Last Signed Transaction:"}</h4>
                            <code class="xdr-display">{format!("{}...{}", &xdr[0..40], &xdr[xdr.len()-40..])}</code>
                            <p class="help-text">{"This signed XDR can be submitted to the Stellar network"}</p>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            <style>
                {r#"
                .contract-test-panel {
                    background: rgba(255, 255, 255, 0.05);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 16px;
                    padding: 2rem;
                    margin: 1rem 0;
                }

                .test-header h3 {
                    margin: 0 0 0.5rem 0;
                    color: #4f46e5;
                    font-size: 1.4rem;
                }

                .test-header p {
                    margin: 0 0 2rem 0;
                    color: rgba(255, 255, 255, 0.7);
                    font-size: 0.9rem;
                }

                .test-buttons {
                    display: grid;
                    grid-template-columns: 1fr 1fr 1fr;
                    gap: 1rem;
                    margin-bottom: 2rem;
                }

                .test-btn {
                    flex: 1;
                    padding: 1rem;
                    border: none;
                    border-radius: 12px;
                    font-weight: 600;
                    cursor: pointer;
                    transition: all 0.3s ease;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 0.5rem;
                    font-size: 1rem;
                }

                .join-btn {
                    background: linear-gradient(135deg, #10b981, #059669);
                    color: white;
                }

                .join-btn:hover:not(:disabled) {
                    transform: translateY(-2px);
                    box-shadow: 0 8px 20px rgba(16, 185, 129, 0.4);
                }

                .score-btn {
                    background: linear-gradient(135deg, #f59e0b, #d97706);
                    color: white;
                }

                .score-btn:hover:not(:disabled) {
                    transform: translateY(-2px);
                    box-shadow: 0 8px 20px rgba(245, 158, 11, 0.4);
                }

                .init-btn {
                    background: linear-gradient(135deg, #8b5cf6, #a855f7);
                    color: white;
                }

                .init-btn:hover:not(:disabled) {
                    transform: translateY(-2px);
                    box-shadow: 0 8px 20px rgba(139, 92, 246, 0.4);
                }

                .test-btn:disabled {
                    opacity: 0.6;
                    cursor: not-allowed;
                }

                .test-result {
                    background: rgba(79, 70, 229, 0.1);
                    border: 1px solid rgba(79, 70, 229, 0.3);
                    border-radius: 12px;
                    padding: 1rem;
                    margin-bottom: 1rem;
                    font-family: 'Monaco', 'Consolas', monospace;
                    font-size: 0.9rem;
                    color: #06b6d4;
                    white-space: pre-wrap;
                }

                .transaction-details {
                    background: rgba(16, 185, 129, 0.1);
                    border: 1px solid rgba(16, 185, 129, 0.3);
                    border-radius: 12px;
                    padding: 1rem;
                }

                .transaction-details h4 {
                    margin: 0 0 1rem 0;
                    color: #10b981;
                    font-size: 1rem;
                }

                .xdr-display {
                    display: block;
                    background: rgba(255, 255, 255, 0.1);
                    padding: 0.75rem;
                    border-radius: 8px;
                    font-family: 'Monaco', 'Consolas', monospace;
                    font-size: 0.8rem;
                    color: #06b6d4;
                    word-break: break-all;
                    margin-bottom: 0.5rem;
                }

                .help-text {
                    margin: 0;
                    font-size: 0.8rem;
                    color: rgba(255, 255, 255, 0.6);
                }
                "#}
            </style>
        </div>
    }
}