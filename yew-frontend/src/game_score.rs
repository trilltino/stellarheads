use yew::prelude::*;
use gloo::storage::{LocalStorage, Storage};
use web_sys::console;
use crate::soroban::{submit_score_to_contract, get_contract_info};
use wasm_bindgen_futures::spawn_local;

#[derive(Properties, PartialEq)]
pub struct GameScoreProps {
    pub score: u64,
    pub game_mode: String,
    pub duration: u64,
    pub achievements: Vec<String>,
}

#[function_component(GameScore)]
pub fn game_score(props: &GameScoreProps) -> Html {
    let submitting = use_state(|| false);
    let result_message = use_state(|| None::<String>);
    let contract_info = use_state(|| None::<crate::soroban::ContractInfo>);

    // Load contract info on component mount
    {
        let contract_info = contract_info.clone();
        use_effect_with_deps(
            move |_| {
                let contract_info = contract_info.clone();
                spawn_local(async move {
                    match get_contract_info().await {
                        Ok(info) => {
                            console::log_1(&format!("📋 Loaded contract: {}", info.contract_address).into());
                            contract_info.set(Some(info));
                        },
                        Err(e) => {
                            console::log_1(&format!("❌ Failed to load contract info: {}", e).into());
                        }
                    }
                });
                || ()
            },
            (),
        );
    }

    let submit_to_blockchain = {
        let submitting = submitting.clone();
        let result_message = result_message.clone();
        let score = props.score;
        let game_mode = props.game_mode.clone();
        let duration = props.duration;
        let achievements = props.achievements.clone();

        Callback::from(move |_| {
            let submitting = submitting.clone();
            let result_message = result_message.clone();
            let game_mode = game_mode.clone();
            let achievements = achievements.clone();

            // Get user data from localStorage
            let username = LocalStorage::get::<String>("username")
                .unwrap_or_else(|_| "Player".to_string());
            let wallet_address = LocalStorage::get::<String>("wallet_address")
                .unwrap_or_else(|_| "".to_string());

            if wallet_address.is_empty() {
                result_message.set(Some("❌ No wallet connected!".to_string()));
                return;
            }

            submitting.set(true);
            result_message.set(Some("🚀 Submitting score to blockchain...".to_string()));

            spawn_local(async move {
                match submit_score_to_contract(
                    &wallet_address,
                    &username,
                    score,
                    &game_mode,
                    duration,
                    achievements,
                ).await {
                    Ok(result) => {
                        console::log_1(&format!("✅ Score submitted! Hash: {}", result.hash).into());
                        result_message.set(Some(format!("🎉 Score submitted to blockchain! Transaction: {}", &result.hash[0..8])));
                    },
                    Err(e) => {
                        console::log_1(&format!("❌ Failed to submit score: {}", e).into());
                        result_message.set(Some(format!("❌ Failed: {}", e)));
                    }
                }
                submitting.set(false);
            });
        })
    };

    html! {
        <div class="game-score-card">
            <div class="score-header">
                <h2>{"🏆 Game Complete!"}</h2>
                <div class="final-score">
                    <span class="score-value">{props.score}</span>
                    <span class="score-label">{"points"}</span>
                </div>
            </div>

            <div class="game-stats">
                <div class="stat-item">
                    <span class="stat-label">{"Mode:"}</span>
                    <span class="stat-value">{&props.game_mode}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">{"Time:"}</span>
                    <span class="stat-value">{format!("{}s", props.duration)}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">{"Achievements:"}</span>
                    <span class="stat-value">{props.achievements.len()}</span>
                </div>
            </div>

            {
                if let Some(info) = (*contract_info).clone() {
                    html! {
                        <div class="contract-info">
                            <p class="contract-address">
                                {"📋 Contract: "}<code>{format!("{}...{}", &info.contract_address[0..8], &info.contract_address[info.contract_address.len()-8..])}</code>
                            </p>
                            <p class="network-info">
                                {"🌐 Network: "}<span class="network-badge">{&info.network_name}</span>
                            </p>
                        </div>
                    }
                } else {
                    html! {
                        <div class="loading-contract">
                            <span class="spinner"></span>
                            <span>{"Loading contract info..."}</span>
                        </div>
                    }
                }
            }

            {
                if let Some(message) = (*result_message).clone() {
                    html! {
                        <div class="result-message">
                            <p>{message}</p>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            <div class="action-buttons">
                <button 
                    class="submit-btn"
                    onclick={submit_to_blockchain}
                    disabled={*submitting || contract_info.is_none()}
                >
                    {
                        if *submitting {
                            html! {
                                <>
                                    <span class="spinner"></span>
                                    <span>{"Submitting..."}</span>
                                </>
                            }
                        } else {
                            html! {
                                <>
                                    <span>{"🌟 Submit to Blockchain"}</span>
                                    <span class="freighter-icon">{"🛸"}</span>
                                </>
                            }
                        }
                    }
                </button>
                
                <button class="play-again-btn">
                    {"🔄 Play Again"}
                </button>
            </div>

            <style>
                {r#"
                .game-score-card {
                    background: rgba(255, 255, 255, 0.05);
                    backdrop-filter: blur(20px);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 24px;
                    padding: 2rem;
                    max-width: 500px;
                    margin: 0 auto;
                    text-align: center;
                    color: white;
                }

                .score-header h2 {
                    font-size: 2rem;
                    margin-bottom: 1rem;
                    background: linear-gradient(135deg, #4f46e5, #06b6d4);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                }

                .final-score {
                    display: flex;
                    align-items: baseline;
                    justify-content: center;
                    gap: 0.5rem;
                    margin-bottom: 2rem;
                }

                .score-value {
                    font-size: 3rem;
                    font-weight: bold;
                    color: #10b981;
                }

                .score-label {
                    font-size: 1rem;
                    color: rgba(255, 255, 255, 0.7);
                }

                .game-stats {
                    display: grid;
                    grid-template-columns: 1fr 1fr 1fr;
                    gap: 1rem;
                    margin-bottom: 2rem;
                    padding: 1rem;
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 12px;
                }

                .stat-item {
                    display: flex;
                    flex-direction: column;
                    gap: 0.25rem;
                }

                .stat-label {
                    font-size: 0.8rem;
                    color: rgba(255, 255, 255, 0.6);
                }

                .stat-value {
                    font-weight: 600;
                    color: #06b6d4;
                }

                .contract-info {
                    background: rgba(79, 70, 229, 0.1);
                    border: 1px solid rgba(79, 70, 229, 0.3);
                    border-radius: 12px;
                    padding: 1rem;
                    margin-bottom: 2rem;
                    font-size: 0.9rem;
                }

                .contract-address code {
                    background: rgba(255, 255, 255, 0.1);
                    padding: 0.25rem 0.5rem;
                    border-radius: 6px;
                    font-family: 'Monaco', 'Consolas', monospace;
                    color: #06b6d4;
                }

                .network-badge {
                    background: linear-gradient(135deg, #10b981, #059669);
                    color: white;
                    padding: 0.25rem 0.5rem;
                    border-radius: 6px;
                    font-size: 0.8rem;
                    font-weight: 600;
                }

                .loading-contract {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 0.5rem;
                    padding: 1rem;
                    color: rgba(255, 255, 255, 0.7);
                }

                .result-message {
                    background: rgba(16, 185, 129, 0.1);
                    border: 1px solid rgba(16, 185, 129, 0.3);
                    border-radius: 12px;
                    padding: 1rem;
                    margin-bottom: 2rem;
                    color: #10b981;
                }

                .action-buttons {
                    display: flex;
                    gap: 1rem;
                    flex-direction: column;
                }

                .submit-btn, .play-again-btn {
                    padding: 1rem 2rem;
                    border: none;
                    border-radius: 16px;
                    font-weight: 600;
                    font-size: 1.1rem;
                    cursor: pointer;
                    transition: all 0.3s ease;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 0.5rem;
                }

                .submit-btn {
                    background: linear-gradient(135deg, #4f46e5, #06b6d4);
                    color: white;
                }

                .submit-btn:hover:not(:disabled) {
                    transform: translateY(-2px);
                    box-shadow: 0 10px 25px rgba(79, 70, 229, 0.4);
                }

                .submit-btn:disabled {
                    opacity: 0.5;
                    cursor: not-allowed;
                }

                .play-again-btn {
                    background: rgba(255, 255, 255, 0.1);
                    color: white;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                }

                .play-again-btn:hover {
                    background: rgba(255, 255, 255, 0.2);
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
                "#}
            </style>
        </div>
    }
}