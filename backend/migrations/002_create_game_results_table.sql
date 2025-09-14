CREATE TABLE game_instances (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    game_session_id VARCHAR(255) NOT NULL, -- UUID as string

    -- Player information (for quick access)
    player_username VARCHAR(255) NOT NULL,
    player_wallet_address VARCHAR(255) NOT NULL,

    -- Game outcome for this player
    player_result VARCHAR(10) NOT NULL, -- 'Win', 'Loss', 'Draw'

    -- Final scores
    player_score INTEGER NOT NULL DEFAULT 0,  -- This player's score
    opponent_score INTEGER NOT NULL DEFAULT 0, -- Opponent's score (AI or other player)

    -- Game metadata
    duration_seconds REAL NOT NULL,
    game_mode VARCHAR(50) DEFAULT 'single_player_vs_ai',

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX idx_game_instances_user_id ON game_instances(user_id);
CREATE INDEX idx_game_instances_player_username ON game_instances(player_username);
CREATE INDEX idx_game_instances_player_wallet ON game_instances(player_wallet_address);
CREATE INDEX idx_game_instances_created_at ON game_instances(created_at);
CREATE INDEX idx_game_instances_player_result ON game_instances(player_result);
CREATE INDEX idx_game_instances_session_id ON game_instances(game_session_id);