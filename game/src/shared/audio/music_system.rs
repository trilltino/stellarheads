use bevy::prelude::*;
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
            _ => self.gamesong.clone(), // fallback to first track
        }
    }
}



#[derive(Component)]
pub struct BackgroundMusic;

#[derive(Component)]
pub struct MenuMusic;

#[derive(Component)]
pub struct SoundEffect;

#[derive(Component)]
pub struct CurrentTrack(pub usize);


#[derive(Event)]
pub struct PlayKickSound;

#[derive(Event)]
pub struct PlayStartGameSound;

#[derive(Event)]
pub struct PlayEndGameSound;


pub fn setup_audio_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    println!("🎵 Setting up audio system...");

    let game_audio = GameAudio {
        kick_sound: asset_server.load("sounds/effects/kick.ogg"),
        startgame_sound: asset_server.load("sounds/effects/startgame.ogg"),
        endgame_sound: asset_server.load("sounds/effects/endgame.ogg"),
        menu_music: asset_server.load("sounds/menu_music.ogg"),
        gamesong: asset_server.load("sounds/gamsong.ogg"),
        gamesong2: asset_server.load("sounds/gamesong2.ogg"),
        gamesong3: asset_server.load("sounds/gamesong3.mp3"),
        gamesong4: asset_server.load("sounds/gamesong4.mp3"),
    };

    commands.insert_resource(game_audio);
    commands.insert_resource(MusicState::default());

    println!("Audio system ready!");
}

pub fn start_background_music(
    mut commands: Commands,
    game_audio: Res<GameAudio>,
    mut music_state: ResMut<MusicState>,
    existing_music: Query<Entity, With<BackgroundMusic>>,
    mut start_game_events: EventWriter<PlayStartGameSound>,
) {
    println!("Starting background music for InGame state...");
    start_game_events.write(PlayStartGameSound);

    for entity in existing_music.iter() {
        commands.entity(entity).despawn();
    }

    music_state.current_track = 0;

    let track_handle = game_audio.get_track(music_state.current_track);

    let music_entity = commands.spawn((
        AudioPlayer(track_handle),
        PlaybackSettings::LOOP,
        BackgroundMusic,
        CurrentTrack(music_state.current_track),
    )).id();

    music_state.current_entity = Some(music_entity);
    println!("🎵 Started track {} with entity {:?}", music_state.current_track + 1, music_entity);
}



pub fn handle_music_loop(
    mut commands: Commands,
    game_audio: Res<GameAudio>,
    mut music_state: ResMut<MusicState>,
    time: Res<Time>,
    app_state: Res<State<AppState>>,
) {
    if !matches!(app_state.get(), AppState::InGame) {
        return;
    }

    static mut TIMER: f32 = 0.0;
    unsafe {
        TIMER += time.delta_secs();
        if TIMER > 60.0 {
            TIMER = 0.0;

            if let Some(entity) = music_state.current_entity {
                commands.entity(entity).despawn();
            }

            music_state.current_track = (music_state.current_track + 1) % 4;

            let track_handle = game_audio.get_track(music_state.current_track);

            let new_entity = commands.spawn((
                AudioPlayer(track_handle),
                PlaybackSettings::LOOP,
                BackgroundMusic,
                CurrentTrack(music_state.current_track),
            )).id();

            music_state.current_entity = Some(new_entity);
            println!("🎵 Switched to track {}", music_state.current_track + 1);
        }
    }
}



pub fn start_menu_music(
    mut commands: Commands,
    game_audio: Option<Res<GameAudio>>,
    existing_music: Query<Entity, Or<(With<BackgroundMusic>, With<MenuMusic>)>>,
) {
    println!("Starting menu music for LaunchMenu state...");

    let Some(game_audio) = game_audio else {
        println!("⚠️ GameAudio resource not yet loaded, skipping menu music");
        return;
    };

    for entity in existing_music.iter() {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        AudioPlayer(game_audio.menu_music.clone()),
        PlaybackSettings::LOOP,
        MenuMusic,
    ));

    println!("🎵 Started menu music");
}

pub fn ensure_menu_music_playing(
    mut commands: Commands,
    game_audio: Option<Res<GameAudio>>,
    existing_menu_music: Query<Entity, With<MenuMusic>>,
    app_state: Res<State<AppState>>,
) {
    // Only run this for LaunchMenu state
    if !matches!(app_state.get(), AppState::LaunchMenu) {
        return;
    }

    // Check if menu music is already playing
    if !existing_menu_music.is_empty() {
        return;
    }

    // Check if GameAudio is loaded
    let Some(game_audio) = game_audio else {
        return; // Audio not loaded yet
    };

    // Start menu music
    commands.spawn((
        AudioPlayer(game_audio.menu_music.clone()),
        PlaybackSettings::LOOP,
        MenuMusic,
    ));

    println!("🎵 Started menu music (delayed start)");
}

pub fn stop_menu_music(
    mut commands: Commands,
    existing_menu_music: Query<Entity, With<MenuMusic>>,
) {
    for entity in existing_menu_music.iter() {
        commands.entity(entity).despawn();
    }
    println!("🔇 Stopped menu music");
}

pub fn stop_audio_system(
    mut commands: Commands,
    existing_music: Query<Entity, With<BackgroundMusic>>,
    mut music_state: ResMut<MusicState>,
    mut end_game_events: EventWriter<PlayEndGameSound>,
) {

    end_game_events.write(PlayEndGameSound);

    for entity in existing_music.iter() {
        commands.entity(entity).despawn();
    }

    music_state.current_entity = None;
    println!("🔇 Stopped all audio");
}



pub fn play_kick_sound(
    mut commands: Commands,
    game_audio: Res<GameAudio>,
    mut kick_events: EventReader<PlayKickSound>,
) {
    for _ in kick_events.read() {
        commands.spawn((
            AudioPlayer(game_audio.kick_sound.clone()),
            PlaybackSettings::ONCE,
            SoundEffect,
        ));
        println!("🦵 Kick sound played!");
    }
}


pub fn play_start_game_sound(
    mut commands: Commands,
    game_audio: Res<GameAudio>,
    mut start_events: EventReader<PlayStartGameSound>,
) {
    for _ in start_events.read() {
        commands.spawn((
            AudioPlayer(game_audio.startgame_sound.clone()),
            PlaybackSettings::ONCE,
            SoundEffect,
        ));
        println!("🚀 Start game sound played!");
    }
}

pub fn play_end_game_sound(
    mut commands: Commands,
    game_audio: Res<GameAudio>,
    mut end_events: EventReader<PlayEndGameSound>,
) {
    for _ in end_events.read() {
        commands.spawn((
            AudioPlayer(game_audio.endgame_sound.clone()),
            PlaybackSettings::ONCE,
            SoundEffect,
        ));
        println!("🏁 End game sound played!");
    }
}


pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        println!("🎵 Registering audio plugin...");

        app
            .add_event::<PlayKickSound>()
            .add_event::<PlayStartGameSound>()
            .add_event::<PlayEndGameSound>()
            .add_systems(Startup, setup_audio_system)
            .add_systems(OnExit(AppState::LaunchMenu), stop_menu_music)
            .add_systems(OnEnter(AppState::InGame), start_background_music)
            .add_systems(OnExit(AppState::InGame), stop_audio_system)
            .add_systems(Update, (
                play_kick_sound,
                play_start_game_sound,
                play_end_game_sound,
                handle_music_loop,
                ensure_menu_music_playing,
            ));

        println!("✅ Audio plugin registered!");
    }
}