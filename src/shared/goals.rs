use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Goal;

#[derive(Bundle)]
pub struct GoalBundle {
    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
}

impl GoalBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        radius: f32,
        color: Color,
        position: Vec3,
    ) -> Self {
        Self {
            mesh: meshes.add(Circle::new(radius)).into(),
            material: materials.add(color).into(),
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Static,
            collider: Collider::circle(radius),
        }
    }
}

struct GroundInfo {
    y: f32,
    width: f32,
    height: f32,
}

impl GroundInfo {
    fn top(&self) -> f32 {
        self.y + self.height / 2.0
    }

    fn bottom(&self) -> f32 {
        self.y - self.height / 2.0
    }
}

fn setup_goal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let ground = GroundInfo {
        width: 1500.0,
        height: 50.0,
        y: -350.0,
    };

    // Spawn ground
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.3, 0.3, 0.3),
            Vec2::new(ground.width, ground.height),
        ),
        Transform::from_xyz(0.0, ground.y, 0.0),
        RigidBody::Static,
        Collider::rectangle(ground.width, ground.height),
    ));

    // Spawn multiple goals at different positions relative to ground
    let goal_radius = 20.0;
    let goal_positions = [
        Vec3::new(-525.0, ground.top() + goal_radius, 0.0),
        Vec3::new(525.0, ground.top() + goal_radius, 0.0),
    ];

    for position in goal_positions {
        commands.spawn((
            GoalBundle::new(
                &mut meshes,
                &mut materials,
                goal_radius,
                Color::srgb(1.0, 0.8, 0.0),
                position,
            ),
            Goal,
        ));
    }
}

pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_goal);
    }
}
