use bevy::prelude::*;
use crate::shared::AppState;

// ================= Audio Resources =================

#[derive(Resource)]
pub struct GameAudio {
    pub kick_sound: Handle<AudioSource>,
    pub startgame_sound: Handle<AudioSource>,
    pub endgame_sound: Handle<AudioSource>,
    pub gamesong: Handle<AudioSource>,
    pub gamesong2: Handle<AudioSource>,
}

#[derive(Resource, Default)]
pub struct MusicState {
    pub current_track: usize, // 0 for gamesong, 1 for gamesong2
    pub current_entity: Option<Entity>,
}



#[derive(Component)]
pub struct BackgroundMusic;

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
        gamesong: asset_server.load("sounds/gamsong.ogg"),
        gamesong2: asset_server.load("sounds/gamesong2.ogg"),
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

    let track_handle = if music_state.current_track == 0 {
        game_audio.gamesong.clone()
    } else {
        game_audio.gamesong2.clone()
    };

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

            music_state.current_track = 1 - music_state.current_track;

            let track_handle = if music_state.current_track == 0 {
                game_audio.gamesong.clone()
            } else {
                game_audio.gamesong2.clone()
            };

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
            .add_systems(OnEnter(AppState::InGame), start_background_music)
            .add_systems(OnExit(AppState::InGame), stop_audio_system)
            .add_systems(Update, (
                play_kick_sound,
                play_start_game_sound,
                play_end_game_sound,
                handle_music_loop,
            ));

        println!("✅ Audio plugin registered!");
    }
}