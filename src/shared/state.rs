use bevy::{dev_tools::states::*, prelude::*};

#[derive(Clone, Copy, Resource, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    LaunchMenu,
    InGame,
    Paused,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum GameUI {
    MainMenuUI,
    GameHUD,
    PausedMenuUI,
    ResultUI,
}

impl ComputedStates for GameUI {
    type SourceStates = AppState;

    fn compute(game_state: AppState) -> Option<Self> {
        match game_state {
            AppState::LaunchMenu => Some(GameUI::MainMenuUI),
            AppState::InGame => Some(GameUI::GameHUD),
            AppState::Paused => Some(GameUI::PauseMenuUI),
            AppState::GameOver => Some(GameUI::ResultUI),
        }
    }
}
