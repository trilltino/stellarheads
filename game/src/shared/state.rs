use bevy::prelude::*;

#[derive(Clone, Copy, Resource, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    LaunchMenu,
    InGame,
    Paused,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameUI {
    MainMenuUI,
    GameHUD,
    PausedMenuUI,
    ResultUI,
}

impl ComputedStates for GameUI {
    type SourceStates = AppState;

    fn compute(source_states: Self::SourceStates) -> Option<Self> {
        match source_states {
            AppState::LaunchMenu => Some(GameUI::MainMenuUI),
            AppState::InGame => Some(GameUI::GameHUD),
            AppState::Paused => Some(GameUI::PausedMenuUI),
            AppState::GameOver => Some(GameUI::ResultUI),
        }
    }
}

