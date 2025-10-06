use bevy::prelude::*;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::ecs::schedule::common_conditions::resource_exists;
use crate::shared::AppState;

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
#[allow(dead_code)] // Reserved for future track selection feature
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

pub fn start_game_music(
    mut commands: Commands,
    game_audio: Res<GameAudio>,
    mut music_state: ResMut<MusicState>,
    existing_music: Query<Entity, With<PlayingMusic>>,
) {
    // Don't start music if already playing
    if !existing_music.is_empty() {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"🎵 Music already playing, skipping".into());
        return;
    }

    // Start first game track
    let first_track = game_audio.get_track(0);

    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("🎵 Starting game music track 0: {:?}", first_track).into());

    let music_entity = commands.spawn((
        PlayingMusic,
        CurrentTrack(0),
        AudioPlayer(first_track.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.5)),  // Set volume to 50%
    )).id();

    music_state.current_track = 0;
    music_state.current_entity = Some(music_entity);

    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"🎵 Game music entity spawned successfully".into());

    #[cfg(not(target_arch = "wasm32"))]
    println!("🎵 Started game music track 0");
}

pub fn play_kick_sound(
    mut commands: Commands,
    mut kick_events: EventReader<PlayKickSoundEvent>,
    game_audio: Res<GameAudio>,
    _music_state: ResMut<MusicState>,
    _existing_music: Query<Entity, With<PlayingMusic>>,
) {
    for _event in kick_events.read() {
        // Play kick sound
        commands.spawn((
            AudioPlayer(game_audio.kick_sound.clone()),
            PlaybackSettings::ONCE,
        ));

        // Also start music if not playing (on first kick - ensures user interaction)
        #[cfg(target_arch = "wasm32")]
        if existing_music.is_empty() && music_state.current_entity.is_none() {
            web_sys::console::log_1(&"🎵 Starting music on first kick (user interaction)".into());

            let first_track = game_audio.get_track(0);
            let music_entity = commands.spawn((
                PlayingMusic,
                CurrentTrack(0),
                AudioPlayer(first_track),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(0.3)),  // Lower volume for background music
            )).id();

            music_state.current_track = 0;
            music_state.current_entity = Some(music_entity);
        }
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

pub fn ensure_music_on_input(
    mut commands: Commands,
    game_audio: Res<GameAudio>,
    mut music_state: ResMut<MusicState>,
    existing_music: Query<Entity, With<PlayingMusic>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
) {
    // Only check in game state
    if *app_state.get() != AppState::InGame {
        return;
    }

    // If any key is pressed and music isn't playing, start it
    if keyboard.get_just_pressed().next().is_some() && existing_music.is_empty() {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"🎵 Starting music on keyboard input".into());

        let first_track = game_audio.get_track(0);
        let music_entity = commands.spawn((
            PlayingMusic,
            CurrentTrack(0),
            AudioPlayer(first_track),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.3)),
        )).id();

        music_state.current_track = 0;
        music_state.current_entity = Some(music_entity);
    }
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PlayKickSoundEvent>()
            .add_event::<PlayStartGameSound>()
            .add_event::<PlayEndGameSound>()
            .add_systems(
                PreStartup,  // Changed from Startup to PreStartup to run earlier
                setup_audio_system,
            )
            .add_systems(
                OnEnter(AppState::InGame),
                start_game_music.run_if(resource_exists::<GameAudio>),  // Only run if resource exists
            )
            .add_systems(
                Update,
                (
                    handle_music_loop,
                    play_kick_sound,
                    play_start_game_sound,
                    play_end_game_sound,
                    ensure_menu_music_playing,
                    ensure_music_on_input,  // Start music on any keyboard input
                ),
            );
    }
}