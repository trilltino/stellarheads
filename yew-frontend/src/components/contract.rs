use yew::prelude::*;
use web_sys::HtmlSelectElement;
use wasm_bindgen::JsCast;
use crate::wallet::ConnectedWallet;
use shared::dto::contract::LeaderboardFunction;

#[derive(Properties, PartialEq)]
pub struct ContractProps {
    pub connected_wallet: Option<ConnectedWallet>,
    pub is_processing: bool,
    pub result_message: String,
    pub selected_function: Option<LeaderboardFunction>,
    pub on_sign_transaction: Callback<()>,
    pub on_select_function: Callback<LeaderboardFunction>,
}

#[function_component(ContractSection)]
pub fn contract_section(props: &ContractProps) -> Html {
    let execute_button_disabled = props.connected_wallet.is_none() || props.is_processing || props.selected_function.is_none();

    let on_execute = {
        let callback = props.on_sign_transaction.clone();
        Callback::from(move |_| callback.emit(()))
    };

    // Get all available functions
    let functions = get_all_leaderboard_functions();

    // State for selected dropdown value
    let selected_dropdown_value = use_state(|| String::new());

    let on_dropdown_change = {
        let selected_dropdown_value = selected_dropdown_value.clone();
        Callback::from(move |e: Event| {
            let target = e.target().unwrap();
            let select = target.dyn_into::<HtmlSelectElement>().unwrap();
            selected_dropdown_value.set(select.value());
        })
    };

    let on_select_function = {
        let callback = props.on_select_function.clone();
        let selected_dropdown_value = selected_dropdown_value.clone();
        let functions = functions.clone();
        Callback::from(move |_| {
            let value = (*selected_dropdown_value).clone();
            if !value.is_empty() {
                if let Some(function) = functions.iter().find(|f| f.name() == value) {
                    callback.emit(function.clone());
                }
            }
        })
    };

    html! {
        <>
            <div class="contract-section">
                <h2>{"Leaderboard Contract Functions"}</h2>

                {if let Some(ref wallet) = props.connected_wallet {
                    html! {
                        <p class="wallet-info">
                            {format!("Connected: {}...{}",
                                &wallet.address[..6],
                                &wallet.address[wallet.address.len()-6..]
                            )}
                        </p>
                    }
                } else {
                    html! {
                        <p class="wallet-info warning">
                            {"Please connect a wallet to interact with the contract"}
                        </p>
                    }
                }}

                <div class="function-selector">
                    <div class="dropdown-container">
                        <select
                            class="function-dropdown"
                            onchange={on_dropdown_change}
                            disabled={props.connected_wallet.is_none()}
                        >
                            <option value="">{"Choose a function..."}</option>
                            {functions.iter().map(|function| {
                                html! {
                                    <option value={function.name()}>
                                        {format!("{} {}", function.icon(), function.display_name())}
                                    </option>
                                }
                            }).collect::<Html>()}
                        </select>

                        <button
                            class="btn btn-select"
                            onclick={on_select_function}
                            disabled={props.connected_wallet.is_none() || (*selected_dropdown_value).is_empty()}
                        >
                            {"Select"}
                        </button>
                    </div>

                    {if let Some(ref selected_func) = props.selected_function {
                        html! {
                            <div class="selected-function-info">
                                <h3>{format!("{} {}", selected_func.icon(), selected_func.display_name())}</h3>
                                <p class="function-signature">{selected_func.signature()}</p>
                                <p class="function-description">{selected_func.description()}</p>

                                <button
                                    class="btn btn-execute"
                                    onclick={on_execute}
                                    disabled={execute_button_disabled}
                                >
                                    {if props.is_processing {
                                        "⏳ Processing...".to_string()
                                    } else {
                                        format!("🚀 Execute {}", selected_func.display_name())
                                    }}
                                </button>
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            </div>

            <div class="result-section">
                <h2>{"Result"}</h2>
                <div class="result-box">
                    {&props.result_message}
                </div>
            </div>
        </>
    }
}

/// Get all available leaderboard functions with default parameters
fn get_all_leaderboard_functions() -> Vec<LeaderboardFunction> {
    vec![
        LeaderboardFunction::Join {
            player: "GCRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ".to_string()
        },
        LeaderboardFunction::HasJoined {
            player: "GCRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ".to_string()
        },
        LeaderboardFunction::AddWin {
            player: "GCRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ".to_string()
        },
        LeaderboardFunction::GetWins {
            player: "GCRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ".to_string()
        },
        LeaderboardFunction::GetMyWins {
            player: "GCRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ".to_string()
        },
        LeaderboardFunction::GetAllPlayers,
        LeaderboardFunction::GetLeaderboard {
            limit: 10
        },
        LeaderboardFunction::GetPlayerCount,
        LeaderboardFunction::GetPlayer {
            player: "GCRBGOBUEZCZF5GIMWSE7MUTY22MGYUMFZY7NZNVPUEDB6MSJIPAJMBZ".to_string()
        },
    ]
}