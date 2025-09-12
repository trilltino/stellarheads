use bevy::prelude::*;
use avian2d::prelude::*;
use bevy_egui::EguiPlugin;
use shared::protocol::*;
use crate::AppState;

// ================= Renderer Plugin =================

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app
            // Rendering plugins
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "🌟 Stellar Heads - Multiplayer".into(),
                    resolution: (1366.0, 768.0).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }))
            .add_plugins(PhysicsPlugins::default())
            .add_plugins(EguiPlugin)
            // Visual systems
            .add_systems(Startup, (
                setup_camera,
                setup_ground,
            ))
            .add_systems(Update, (
                render_networked_entities,
                update_ui,
            ))
            .add_systems(OnEnter(AppState::InGame), setup_game_visuals);
    }
}

// ================= Setup Systems =================

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,
            ..default()
        },
        Name::new("Main Camera"),
    ));
    
    println!("📷 Camera setup complete");
}

fn setup_ground(mut commands: Commands) {
    // Ground
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.8, 0.2), Vec2::new(5000.0, 50.0)),
        Transform::from_xyz(0.0, -350.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(2500.0, 25.0),
        Name::new("Ground"),
    ));
    
    // Walls  
    for (x, name) in [(-683.0, "Left Wall"), (683.0, "Right Wall")] {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.4, 0.4, 0.4), Vec2::new(60.0, 1000.0)),
            Transform::from_xyz(x, 0.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(30.0, 500.0),
            Name::new(name),
        ));
    }
    
    // Ceiling
    commands.spawn((
        Transform::from_xyz(0.0, 384.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(683.0, 10.0),
        Name::new("Ceiling"),
    ));
    
    println!("🏟️ Ground and boundaries setup complete");
}

fn setup_game_visuals() {
    println!("🎮 Game visuals initialized");
}

// ================= Rendering Systems =================

fn render_networked_entities(
    mut commands: Commands,
    balls: Query<(Entity, &NetworkedBall, &Transform), (Added<NetworkedBall>, Without<Sprite>)>,
    players: Query<(Entity, &NetworkedPlayer, &Transform), (Added<NetworkedPlayer>, Without<Sprite>)>,
    goals: Query<(Entity, &NetworkedGoal, &Transform), (Added<NetworkedGoal>, Without<Sprite>)>,
) {
    // Render balls
    for (entity, ball, transform) in balls.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::splat(ball.radius * 2.0)),
            *transform,
        ));
        println!("⚽ Rendered networked ball");
    }
    
    // Render players  
    for (entity, player, transform) in players.iter() {
        let color = match player.team {
            shared::dto::user::Team::Left => Color::srgb(0.2, 0.7, 0.9), // Blue
            shared::dto::user::Team::Right => Color::srgb(1.0, 0.4, 0.2), // Orange
        };
        
        commands.entity(entity).insert((
            Sprite::from_color(color, Vec2::splat(40.0)),
            *transform,
        ));
        println!("👤 Rendered networked player");
    }
    
    // Render goals
    for (entity, goal, transform) in goals.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(Color::srgba(1.0, 1.0, 0.0, 0.8), Vec2::new(65.0, 160.0)),
            *transform,
        ));
        println!("🥅 Rendered networked goal");
    }
}

fn update_ui(
    mut contexts: bevy_egui::EguiContexts,
    app_state: Res<State<AppState>>,
) {
    bevy_egui::egui::Window::new("Game Info")
        .default_pos([10.0, 10.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("State: {:?}", app_state.get()));
            ui.label("🌟 Stellar Heads - Multiplayer");
            
            match app_state.get() {
                AppState::MainMenu => {
                    ui.label("Main Menu - Choose your game mode");
                }
                AppState::Lobby => {
                    ui.label("In Lobby - Waiting for players...");
                }
                AppState::InGame => {
                    ui.label("In Game - Playing!");
                    ui.label("Controls: WASD/Arrows to move, Space to jump, X to kick");
                }
                AppState::GameOver => {
                    ui.label("Game Over");
                }
            }
        });
}
