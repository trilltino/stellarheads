use bevy::prelude::*;

/// Create a minimal test app without window/rendering for unit tests
fn create_test_app() -> App {
    use avian2d::prelude::*;
    use bevy::state::app::StatesPlugin;
    use stellar_heads_game::shared::config::GameConfigPlugin;
    use stellar_heads_game::shared::scoring::ScoringPlugin;
    use stellar_heads_game::shared::AppState;

    let mut app = App::new();

    // Add only the minimal plugins needed for testing
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        PhysicsPlugins::default().with_length_unit(1.0),
        GameConfigPlugin,
        ScoringPlugin,
    ))
    .insert_state(AppState::InGame);

    app
}

mod scoring {
    use super::*;
    use bevy::ecs::event::Events;

    // Import game types
    use stellar_heads_game::shared::scoring::*;

    #[test]
    fn test_score_initialization() {
        let app = create_test_app();

        // Verify score resource is initialized
        assert!(app.world().contains_resource::<Score>());

        let score = app.world().resource::<Score>();
        assert_eq!(score.left_team, 0);
        assert_eq!(score.right_team, 0);
        assert_eq!(score.high_score, 0);
    }

    #[test]
    fn test_add_point_left_team() {
        let mut score = Score::default();

        score.add_point(GoalTeam::Left);

        assert_eq!(score.left_team, 1);
        assert_eq!(score.right_team, 0);
        assert_eq!(score.high_score, 1);
    }

    #[test]
    fn test_add_point_right_team() {
        let mut score = Score::default();

        score.add_point(GoalTeam::Right);

        assert_eq!(score.left_team, 0);
        assert_eq!(score.right_team, 1);
        assert_eq!(score.high_score, 1);
    }

    #[test]
    fn test_high_score_tracking() {
        let mut score = Score::default();

        score.add_point(GoalTeam::Left);
        score.add_point(GoalTeam::Right);
        score.add_point(GoalTeam::Left);

        assert_eq!(score.left_team, 2);
        assert_eq!(score.right_team, 1);
        assert_eq!(score.high_score, 3);
    }

    #[test]
    fn test_reset_score() {
        let mut score = Score::default();

        score.add_point(GoalTeam::Left);
        score.add_point(GoalTeam::Right);
        score.reset();

        assert_eq!(score.left_team, 0);
        assert_eq!(score.right_team, 0);
        // High score should persist after reset
        assert_eq!(score.high_score, 2);
    }

    #[test]
    fn test_get_winner_none() {
        let score = Score::default();
        assert_eq!(score.get_winner(5), None);
    }

    #[test]
    fn test_get_winner_left() {
        let mut score = Score::default();

        for _ in 0..5 {
            score.add_point(GoalTeam::Left);
        }

        assert_eq!(score.get_winner(5), Some(GoalTeam::Left));
    }

    #[test]
    fn test_get_winner_right() {
        let mut score = Score::default();

        for _ in 0..5 {
            score.add_point(GoalTeam::Right);
        }

        assert_eq!(score.get_winner(5), Some(GoalTeam::Right));
    }

    #[test]
    fn test_is_match_point() {
        let mut score = Score::default();

        assert!(!score.is_match_point(5));

        for _ in 0..4 {
            score.add_point(GoalTeam::Left);
        }

        assert!(score.is_match_point(5));
    }

    #[test]
    fn test_custom_winning_score() {
        let mut score = Score::default();

        // Test with winning score of 3
        for _ in 0..3 {
            score.add_point(GoalTeam::Left);
        }

        assert_eq!(score.get_winner(3), Some(GoalTeam::Left));
        assert_eq!(score.get_winner(5), None); // Not enough for 5
    }

    #[test]
    fn test_goal_scored_event_processing() {
        let mut app = create_test_app();

        // Send a goal scored event
        app.world_mut().resource_mut::<Events<GoalScored>>().send(GoalScored {
            goal_position: Vec3::new(400.0, 0.0, 0.0),
            scoring_team: GoalTeam::Left,
        });

        // Run systems
        app.update();

        // Check that score was updated
        let score = app.world().resource::<Score>();
        assert_eq!(score.left_team, 1);
        assert_eq!(score.right_team, 0);
    }

    #[test]
    fn test_match_finished_on_winning_score() {
        let mut app = create_test_app();

        // Send 5 goals for left team
        for _ in 0..5 {
            app.world_mut().resource_mut::<Events<GoalScored>>().send(GoalScored {
                goal_position: Vec3::ZERO,
                scoring_team: GoalTeam::Left,
            });
            app.update();
        }

        // Check that match finished event was sent
        let match_events = app.world().resource::<Events<MatchFinished>>();
        let events: Vec<_> = match_events.iter_current_update_events().collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].winner, Some(GoalTeam::Left));
    }

    #[test]
    fn test_game_timer_countdown() {
        let mut timer = GameTimer::default();
        let initial_time = timer.remaining_time;

        // Simulate 1 second passing
        timer.remaining_time -= 1.0;

        assert_eq!(timer.remaining_time, initial_time - 1.0);
        assert!(!timer.is_finished);
    }

    #[test]
    fn test_game_timer_finish() {
        let mut timer = GameTimer::default();

        // Simulate time running out
        timer.remaining_time = -1.0;

        // Manually trigger the finish logic
        if timer.remaining_time <= 0.0 {
            timer.remaining_time = 0.0;
            timer.is_finished = true;
        }

        assert_eq!(timer.remaining_time, 0.0);
        assert!(timer.is_finished);
    }

    #[test]
    fn test_score_notification_creation() {
        let notification = ScoreNotification::new("Test".to_string(), 2.0);

        assert_eq!(notification.text, "Test");
        assert_eq!(notification.timer, 2.0);
        assert_eq!(notification.max_time, 2.0);
    }

    #[test]
    fn test_game_session_generation() {
        let session1 = GameSession::new();
        let session2 = GameSession::new();

        // Each session should have a unique ID
        assert_ne!(session1.session_id, session2.session_id);

        // UUIDs should be valid format (just check they're not empty)
        assert!(!session1.session_id.is_empty());
        assert!(!session2.session_id.is_empty());
    }

    #[test]
    fn test_player_info_default() {
        let player_info = PlayerInfo::default();

        assert_eq!(player_info.username, "");
        assert_eq!(player_info.wallet_address, "");
    }

    #[test]
    fn test_score_notifications_resource() {
        let mut notifications = ScoreNotifications::default();

        assert_eq!(notifications.notifications.len(), 0);

        notifications.notifications.push(ScoreNotification::new("Goal!".to_string(), 2.0));

        assert_eq!(notifications.notifications.len(), 1);
        assert_eq!(notifications.notifications[0].text, "Goal!");
    }
}
