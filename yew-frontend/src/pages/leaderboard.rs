use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use web_sys::MouseEvent;
use shared::dto::contract::{
    LeaderboardFunction, ContractXdrRequest, ContractXdrResponse,
    ContractSubmitRequest, ContractSubmitResponse, LeaderboardResponse,
};
use crate::wallet::{connect_wallet, sign_transaction, FreighterError, ConnectedWallet};
use crate::components::ContractSection;

#[function_component(LeaderboardPage)]
pub fn leaderboard_page() -> Html {
    let connected_wallet = use_state(|| None::<ConnectedWallet>);
    let leaderboard_data = use_state(|| None::<LeaderboardResponse>);
    let selected_function = use_state(|| None::<LeaderboardFunction>);
    let is_processing = use_state(|| false);
    let result_message = use_state(|| "Ready to interact with the leaderboard contract".to_string());
    let is_loading_leaderboard = use_state(|| false);

    // Connect wallet handler
    let on_connect_wallet = {
        let connected_wallet = connected_wallet.clone();
        let result_message = result_message.clone();

        Callback::from(move |_| {
            let connected_wallet = connected_wallet.clone();
            let result_message = result_message.clone();

            spawn_local(async move {
                result_message.set("🔗 Connecting to Freighter wallet...".to_string());

                match connect_wallet().await {
                    Ok(address) => {
                        let wallet = ConnectedWallet::freighter(address.clone());
                        connected_wallet.set(Some(wallet));
                        result_message.set(format!("✅ Connected to wallet: {}...{}",
                            &address[..6], &address[address.len()-6..]));
                    }
                    Err(FreighterError::UserRejected) => {
                        result_message.set("❌ Connection rejected by user".to_string());
                    }
                    Err(FreighterError::FreighterExtNotFound) => {
                        result_message.set("❌ Freighter wallet not found. Please install from https://freighter.app/".to_string());
                    }
                    Err(e) => {
                        result_message.set(format!("❌ Connection failed: {}", e));
                    }
                }
            });
        })
    };

    // Load leaderboard data
    let on_load_leaderboard = {
        let leaderboard_data = leaderboard_data.clone();
        let is_loading_leaderboard = is_loading_leaderboard.clone();
        let result_message = result_message.clone();

        Callback::from(move |_: MouseEvent| {
            let leaderboard_data = leaderboard_data.clone();
            let is_loading_leaderboard = is_loading_leaderboard.clone();
            let result_message = result_message.clone();

            spawn_local(async move {
                is_loading_leaderboard.set(true);
                result_message.set("📊 Loading leaderboard data...".to_string());

                // First get the leaderboard via contract call
                match load_leaderboard_from_contract().await {
                    Ok(data) => {
                        leaderboard_data.set(Some(data));
                        result_message.set("✅ Leaderboard loaded successfully".to_string());
                    }
                    Err(e) => {
                        result_message.set(format!("❌ Failed to load leaderboard: {}", e));
                    }
                }

                is_loading_leaderboard.set(false);
            });
        })
    };

    // Select function handler
    let on_select_function = {
        let selected_function = selected_function.clone();
        let connected_wallet = connected_wallet.clone();

        Callback::from(move |function: LeaderboardFunction| {
            // Update function with actual wallet address if available
            let function_with_wallet = if let Some(ref wallet) = *connected_wallet {
                update_function_with_wallet_address(function, &wallet.address)
            } else {
                function
            };

            selected_function.set(Some(function_with_wallet));
        })
    };

    // Sign transaction handler
    let on_sign_transaction = {
        let connected_wallet = connected_wallet.clone();
        let selected_function = selected_function.clone();
        let is_processing = is_processing.clone();
        let result_message = result_message.clone();
        let leaderboard_data = leaderboard_data.clone();

        Callback::from(move |_| {
            let connected_wallet = connected_wallet.clone();
            let selected_function = selected_function.clone();
            let is_processing = is_processing.clone();
            let result_message = result_message.clone();
            let leaderboard_data = leaderboard_data.clone();

            spawn_local(async move {
                if let (Some(wallet), Some(function)) = ((*connected_wallet).clone(), (*selected_function).clone()) {
                    is_processing.set(true);
                    result_message.set("🔄 Generating XDR...".to_string());

                    match execute_contract_function(wallet, function).await {
                        Ok(success_message) => {
                            result_message.set(success_message);

                            // If it was a leaderboard-affecting function, reload the data
                            if matches!((*selected_function).as_ref(),
                                Some(LeaderboardFunction::Join { .. }) |
                                Some(LeaderboardFunction::AddWin { .. })) {

                                // Reload leaderboard after a short delay
                                Timeout::new(2000, move || {
                                    spawn_local(async move {
                                        if let Ok(data) = load_leaderboard_from_contract().await {
                                            leaderboard_data.set(Some(data));
                                        }
                                    });
                                }).forget();
                            }
                        }
                        Err(e) => {
                            result_message.set(format!("❌ Transaction failed: {}", e));
                        }
                    }

                    is_processing.set(false);
                }
            });
        })
    };

    // Load leaderboard on component mount
    {
        let leaderboard_data = leaderboard_data.clone();
        let is_loading_leaderboard = is_loading_leaderboard.clone();
        let result_message = result_message.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                is_loading_leaderboard.set(true);
                result_message.set("📊 Loading leaderboard data...".to_string());

                match load_leaderboard_from_contract().await {
                    Ok(data) => {
                        leaderboard_data.set(Some(data));
                        result_message.set("✅ Leaderboard loaded successfully".to_string());
                    }
                    Err(e) => {
                        result_message.set(format!("❌ Failed to load leaderboard: {}", e));
                    }
                }

                is_loading_leaderboard.set(false);
            });
            || ()
        });
    }

    html! {
        <div class="leaderboard-page">
            <div class="page-header">
                <h1>{"🏆 Stellar Heads Leaderboard"}</h1>
                <p>{"Connect your wallet and interact with the leaderboard smart contract"}</p>
            </div>

            <div class="wallet-section">
                {if (*connected_wallet).is_none() {
                    html! {
                        <button class="btn btn-primary" onclick={on_connect_wallet}>
                            {"🔗 Connect Freighter Wallet"}
                        </button>
                    }
                } else {
                    html! {
                        <div class="wallet-connected">
                            <span class="status-indicator">{"✅ Wallet Connected"}</span>
                            <button class="btn btn-secondary" onclick={on_load_leaderboard}>
                                {"🔄 Refresh Leaderboard"}
                            </button>
                        </div>
                    }
                }}
            </div>

            <div class="leaderboard-content">
                <div class="leaderboard-display">
                    <h2>{"📊 Current Leaderboard"}</h2>

                    {if *is_loading_leaderboard {
                        html! {
                            <div class="loading">
                                <p>{"⏳ Loading leaderboard data..."}</p>
                            </div>
                        }
                    } else if let Some(ref data) = *leaderboard_data {
                        html! {
                            <div class="leaderboard-table">
                                <div class="leaderboard-stats">
                                    <p>{format!("Total Players: {}", data.total_players)}</p>
                                    {if let Some(ref updated) = data.last_updated {
                                        html! { <p>{format!("Last Updated: {}", updated)}</p> }
                                    } else {
                                        html! {}
                                    }}
                                </div>

                                {if data.entries.is_empty() {
                                    html! {
                                        <div class="empty-leaderboard">
                                            <p>{"No players have joined the leaderboard yet"}</p>
                                            <p>{"Be the first to join!"}</p>
                                        </div>
                                    }
                                } else {
                                    html! {
                                        <table class="leaderboard">
                                            <thead>
                                                <tr>
                                                    <th>{"Rank"}</th>
                                                    <th>{"Address"}</th>
                                                    <th>{"Username"}</th>
                                                    <th>{"Wins"}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {data.entries.iter().map(|entry| {
                                                    html! {
                                                        <tr key={entry.address.clone()}>
                                                            <td class="rank">{entry.rank}</td>
                                                            <td class="address">
                                                                {format!("{}...{}",
                                                                    &entry.address[..6],
                                                                    &entry.address[entry.address.len()-6..])}
                                                            </td>
                                                            <td class="username">
                                                                {entry.username.as_deref().unwrap_or("Anonymous")}
                                                            </td>
                                                            <td class="wins">{entry.wins}</td>
                                                        </tr>
                                                    }
                                                }).collect::<Html>()}
                                            </tbody>
                                        </table>
                                    }
                                }}
                            </div>
                        }
                    } else {
                        html! {
                            <div class="no-data">
                                <p>{"Click \"Refresh Leaderboard\" to load data"}</p>
                            </div>
                        }
                    }}
                </div>

                <div class="contract-interaction">
                    <ContractSection
                        connected_wallet={(*connected_wallet).clone()}
                        is_processing={*is_processing}
                        result_message={(*result_message).clone()}
                        selected_function={(*selected_function).clone()}
                        on_sign_transaction={on_sign_transaction}
                        on_select_function={on_select_function}
                    />
                </div>
            </div>
        </div>
    }
}

// Helper function to update function with actual wallet address
fn update_function_with_wallet_address(function: LeaderboardFunction, wallet_address: &str) -> LeaderboardFunction {
    match function {
        LeaderboardFunction::Join { .. } => LeaderboardFunction::Join { player: wallet_address.to_string() },
        LeaderboardFunction::HasJoined { .. } => LeaderboardFunction::HasJoined { player: wallet_address.to_string() },
        LeaderboardFunction::AddWin { .. } => LeaderboardFunction::AddWin { player: wallet_address.to_string() },
        LeaderboardFunction::GetWins { .. } => LeaderboardFunction::GetWins { player: wallet_address.to_string() },
        LeaderboardFunction::GetMyWins { .. } => LeaderboardFunction::GetMyWins { player: wallet_address.to_string() },
        LeaderboardFunction::GetPlayer { .. } => LeaderboardFunction::GetPlayer { player: wallet_address.to_string() },
        other => other,
    }
}

// Load leaderboard data from the contract
async fn load_leaderboard_from_contract() -> Result<LeaderboardResponse, String> {
    // Call backend API to get leaderboard data
    let response = Request::get("http://localhost:3000/api/leaderboard")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.ok() {
        response
            .json::<LeaderboardResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err(format!("Server error: {}", response.status()))
    }
}

// Execute a contract function
async fn execute_contract_function(wallet: ConnectedWallet, function: LeaderboardFunction) -> Result<String, String> {
    // Step 1: Generate XDR
    let xdr_request = ContractXdrRequest {
        source_account: wallet.address.clone(),
        function: function.clone(),
        wallet_type: Some(wallet.wallet_type.clone()),
    };

    let xdr_response = Request::post("http://localhost:3000/api/contract/generate-xdr")
        .json(&xdr_request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("XDR request failed: {}", e))?;

    if !xdr_response.ok() {
        return Err(format!("XDR generation failed: {}", xdr_response.status()));
    }

    let xdr_result: ContractXdrResponse = xdr_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse XDR response: {}", e))?;

    if !xdr_result.success {
        return Err(format!("XDR generation failed: {}", xdr_result.message));
    }

    let xdr = xdr_result.xdr.ok_or("No XDR in response")?;

    // Step 2: Sign with Freighter
    let signed_xdr = sign_transaction(&xdr, "Test SDF Network ; September 2015")
        .await
        .map_err(|e| format!("Signing failed: {}", e))?;

    // Step 3: Submit transaction
    let submit_request = ContractSubmitRequest {
        signed_xdr,
        function,
        wallet_type: Some(wallet.wallet_type),
    };

    let submit_response = Request::post("/api/contract/submit-transaction")
        .json(&submit_request)
        .map_err(|e| format!("Failed to serialize submit request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Submit request failed: {}", e))?;

    if !submit_response.ok() {
        return Err(format!("Transaction submission failed: {}", submit_response.status()));
    }

    let submit_result: ContractSubmitResponse = submit_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse submit response: {}", e))?;

    if !submit_result.success {
        return Err(format!("Transaction failed: {}", submit_result.message));
    }

    Ok(format!("✅ Transaction successful: {}", submit_result.message))
}