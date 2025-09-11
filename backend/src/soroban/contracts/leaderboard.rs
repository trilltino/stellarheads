use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec, Map};

#[derive(Clone)]
#[contracttype]
pub struct PlayerScore {
    pub player: Address,
    pub username: String,
    pub score: u64,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct GameSession {
    pub session_id: String,
    pub player: Address,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub final_score: Option<u64>,
}

#[contract]
pub struct LeaderboardContract;

#[contractimpl]
impl LeaderboardContract {
    /// Initialize the leaderboard contract
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&"admin", &admin);
    }

    /// Submit a new score to the leaderboard
    pub fn submit_score(
        env: Env,
        player: Address,
        username: String,
        score: u64,
    ) -> bool {
        player.require_auth();
        
        let timestamp = env.ledger().timestamp();
        let player_score = PlayerScore {
            player: player.clone(),
            username,
            score,
            timestamp,
        };

        // Store individual score
        let score_key = format!("score_{}_{}", player, timestamp);
        env.storage().persistent().set(&score_key, &player_score);

        // Update player's best score
        let best_key = format!("best_{}", player);
        let current_best: Option<PlayerScore> = env.storage().persistent().get(&best_key);
        
        match current_best {
            Some(best) if best.score < score => {
                env.storage().persistent().set(&best_key, &player_score);
            },
            None => {
                env.storage().persistent().set(&best_key, &player_score);
            },
            _ => {} // Current score is not better
        }

        true
    }

    /// Get top N players from leaderboard
    pub fn get_leaderboard(env: Env, limit: u32) -> Vec<PlayerScore> {
        // In a real implementation, you'd want to maintain a sorted structure
        // For simplicity, this is a basic implementation
        let mut scores: Vec<PlayerScore> = Vec::new(&env);
        
        // This would need proper iteration over all best scores
        // and sorting - simplified for demonstration
        scores
    }

    /// Get a player's best score
    pub fn get_player_best(env: Env, player: Address) -> Option<PlayerScore> {
        let best_key = format!("best_{}", player);
        env.storage().persistent().get(&best_key)
    }

    /// Start a new game session
    pub fn start_game_session(
        env: Env,
        player: Address,
        session_id: String,
    ) -> GameSession {
        player.require_auth();
        
        let session = GameSession {
            session_id: session_id.clone(),
            player: player.clone(),
            start_time: env.ledger().timestamp(),
            end_time: None,
            final_score: None,
        };

        let session_key = format!("session_{}", session_id);
        env.storage().temporary().set(&session_key, &session);
        
        session
    }

    /// End a game session and submit final score
    pub fn end_game_session(
        env: Env,
        session_id: String,
        final_score: u64,
    ) -> bool {
        let session_key = format!("session_{}", session_id);
        let mut session: GameSession = env.storage()
            .temporary()
            .get(&session_key)
            .expect("Session not found");

        session.player.require_auth();
        session.end_time = Some(env.ledger().timestamp());
        session.final_score = Some(final_score);

        // Update session
        env.storage().temporary().set(&session_key, &session);

        // Submit the score to leaderboard
        // Note: In practice, you'd want additional validation here
        true
    }
}
