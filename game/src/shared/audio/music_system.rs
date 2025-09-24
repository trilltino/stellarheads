#[cfg(not(target_arch = "wasm32"))]
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
#[cfg(not(target_arch = "wasm32"))]
use crate::shared::AppState;

#[cfg(not(target_arch = "wasm32"))]
pub mod audio_impl {
    use super::*;

    // ================= Audio Resources =================

    #[derive(Resource)]
    pub struct GameAudio {
        pub kick_sound: Handle<AudioSource>,
        pub startgame_sound: Handle<AudioSource>,
        pub endgame_sound: Handle<AudioSource>,
        pub menu_music: Handle<AudioSource>,
        pub gamesong: Handle<AudioSource>,
        pub gamesong2: Handle<AudioSource>,
        pub gamesong3: Handle<AudioSource>,
        pub gamesong4: Handle<AudioSource>,
    }

    #[derive(Resource, Default)]
    pub struct MusicState {
        pub current_track: usize, // 0-3 for gamesong1-4
        pub current_entity: Option<Entity>,
    }

    impl GameAudio {
        pub fn get_track(&self, track_index: usize) -> Handle<AudioSource> {
            match track_index {
                0 => self.gamesong.clone(),
                1 => self.gamesong2.clone(),
                2 => self.gamesong3.clone(),
                3 => self.gamesong4.clone(),
                _ => self.gamesong.clone(), // Default to first track
            }
        }
    }

    // ================= Audio Control Events =================


    #[derive(Component)]
    pub struct PlayingMusic;

    #[derive(Component)]
    pub struct CurrentTrack(usize);

    #[derive(Event)]
    pub struct PlayKickSoundEvent;

    // Alias for compatibility
    pub type PlayKickSound = PlayKickSoundEvent;

    #[derive(Event)]
    pub struct PlayStartGameSound;

    #[derive(Event)]
    pub struct PlayEndGameSound;

    pub fn setup_audio_system(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        let game_audio = GameAudio {
            kick_sound: asset_server.load("sounds/effects/kick.ogg"),
            startgame_sound: asset_server.load("sounds/effects/startgame.ogg"),
            endgame_sound: asset_server.load("sounds/effects/endgame.ogg"),
            menu_music: asset_server.load("sounds/menu_music.ogg"),
            gamesong: asset_server.load("sounds/gamsong.ogg"), // Note: typo in your filename
            gamesong2: asset_server.load("sounds/gamesong2.ogg"),
            gamesong3: asset_server.load("sounds/gamesong3.mp3"),
            gamesong4: asset_server.load("sounds/gamesong4.mp3"),
        };

        commands.insert_resource(game_audio);
        commands.insert_resource(MusicState {
            current_track: 0,
            current_entity: None,
        });
    }


    pub fn handle_music_loop(
        mut commands: Commands,
        game_audio: Res<GameAudio>,
        mut music_state: ResMut<MusicState>,
        finished_music: Query<Entity, (With<PlayingMusic>, Without<AudioPlayer>)>,
    ) {
        for entity in finished_music.iter() {
            commands.entity(entity).despawn();

            // Start next track
            music_state.current_track = (music_state.current_track + 1) % 4;
            let next_track = game_audio.get_track(music_state.current_track);

            let music_entity = commands.spawn((
                PlayingMusic,
                CurrentTrack(music_state.current_track),
                AudioPlayer(next_track),
                PlaybackSettings::LOOP,
            )).id();

            music_state.current_entity = Some(music_entity);
        }
    }


    pub fn ensure_menu_music_playing(
        mut commands: Commands,
        game_audio: Res<GameAudio>,
        existing_music: Query<Entity, With<PlayingMusic>>,
        app_state: Res<State<AppState>>,
    ) {
        if *app_state.get() == AppState::LaunchMenu && existing_music.is_empty() {
            commands.spawn((
                PlayingMusic,
                AudioPlayer(game_audio.menu_music.clone()),
                PlaybackSettings::LOOP,
            ));
        }
    }



    pub fn play_kick_sound(
        mut commands: Commands,
        mut kick_events: EventReader<PlayKickSoundEvent>,
        game_audio: Res<GameAudio>,
    ) {
        for _event in kick_events.read() {
            commands.spawn((
                AudioPlayer(game_audio.kick_sound.clone()),
                PlaybackSettings::ONCE,
            ));
        }
    }

    pub fn play_start_game_sound(
        mut commands: Commands,
        mut start_events: EventReader<PlayStartGameSound>,
        game_audio: Res<GameAudio>,
    ) {
        for _event in start_events.read() {
            commands.spawn((
                AudioPlayer(game_audio.startgame_sound.clone()),
                PlaybackSettings::ONCE,
            ));
        }
    }

    pub fn play_end_game_sound(
        mut commands: Commands,
        mut end_events: EventReader<PlayEndGameSound>,
        game_audio: Res<GameAudio>,
    ) {
        for _event in end_events.read() {
            commands.spawn((
                AudioPlayer(game_audio.endgame_sound.clone()),
                PlaybackSettings::ONCE,
            ));
        }
    }

    #[derive(Default)]
    pub struct GameAudioPlugin;

    impl Plugin for GameAudioPlugin {
        fn build(&self, app: &mut App) {
            app
                .add_event::<PlayKickSoundEvent>()
                .add_event::<PlayStartGameSound>()
                .add_event::<PlayEndGameSound>()
                .add_systems(
                    Startup,
                    setup_audio_system,
                )
                .add_systems(
                    Update,
                    (
                        handle_music_loop,
                        play_kick_sound,
                        play_start_game_sound,
                        play_end_game_sound,
                        ensure_menu_music_playing,
                    ),
                );
        }
    }
}

// Re-export the audio types for non-wasm builds
#[cfg(not(target_arch = "wasm32"))]
pub use audio_impl::*;

// For WASM builds, provide stub implementations
#[cfg(target_arch = "wasm32")]
pub mod audio_stubs {
    use bevy::prelude::*;

    #[derive(Default)]
    pub struct GameAudioPlugin;

    impl Plugin for GameAudioPlugin {
        fn build(&self, app: &mut App) {
            // Register events for WASM builds to prevent validation errors
            app
                .add_event::<PlayKickSoundEvent>()
                .add_event::<PlayStartGameSound>()
                .add_event::<PlayEndGameSound>();
        }
    }

    #[derive(Event)]
    pub struct PlayKickSoundEvent;

    // Alias for compatibility
    pub type PlayKickSound = PlayKickSoundEvent;

    #[derive(Event)]
    pub struct PlayStartGameSound;

    #[derive(Event)]
    pub struct PlayEndGameSound;
}

#[cfg(target_arch = "wasm32")]
pub use audio_stubs::*;