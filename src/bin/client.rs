use rand::Rng;
use bevy::window::PrimaryWindow;
use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::{Bloom, BloomCompositeMode},
    prelude::*,
};

fn main() {
    App::new()
        .insert_resource(CursorPos(Vec2::ZERO))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_enemies)
        .add_systems(Update, (cursor_position_system, player_movement_system, apply_velocity_system, enemy_kill_system))
        .run();
}

#[derive(Resource)]
pub struct CursorPos(Vec2);

#[derive(Component)]
pub struct Velocity(Vec2);

#[derive(Component)]
pub struct Radius(f32);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Alive(bool);

#[derive(Component)]
pub struct Enemy;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface, // 1. Using a tonemapper that desaturates to white is recommended
        Bloom::default(),           // 2. Enable bloom for the camera
        DebandDither::Enabled,      // Optional: bloom causes gradients which cause banding
        Transform::from_xyz(200., 0., 1.),
        Player,
        Alive(true),
        Radius(20.),
        Velocity(Vec2::new(0., 0.)),
        Mesh2d(meshes.add(Circle::new(20.))),
        // 3. Put something bright in a dark environment to see the effect
        MeshMaterial2d(materials.add(Color::srgb(0., 1., 0.))),
    ));
}

fn random_velocity() -> Vec2 {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let speed = rng.random_range(50.0..200.0);
    Vec2::from_angle(angle) * speed
}

fn random_position(range: f32) -> Vec2 {
    let mut rng = rand::rng();
    Vec2::new(
        rng.random_range(-range..range),
        rng.random_range(-range..range),
    )
}


fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();
    for _ in 0..100 {
        let velocity = Velocity(random_velocity());
        let position = random_position(1000.);
        let material = MeshMaterial2d(materials.add(Color::srgb(
            rng.random_range(0.0..4.0),
            rng.random_range(0.0..4.0),
            rng.random_range(0.0..4.0),
        )));

        // Circle mesh
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(40.))),
            // 3. Put something bright in a dark environment to see the effect
            material,
            Transform::from_translation(position.extend(0.)),
            velocity,
            Enemy,
            Radius(40.),
        ));
    }
}

fn cursor_position_system(
    window: Single<&Window, With<PrimaryWindow>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor: ResMut<CursorPos>,
) {
    let window_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);

    if let Some(cursor_position) = window.cursor_position() {
        cursor.0 = (cursor_position - window_center) * Vec2::new(1., -1.); // relative to center
    }
}

fn player_movement_system(
    cursor: Res<CursorPos>,
    mut query: Query<(&mut Velocity, &Alive), With<Player>>,
) {
    for (mut velocity, alive) in query {
        if alive.0 {
            let speed = 300.0; // units per second
            let length = cursor.0.length();
            let threshold = 200.;
            if length == 0. {
                continue;
            }
            let percentage = length / threshold;

            velocity.0 = cursor.0.normalize() * percentage * speed;
        }
        else {
            velocity.0 = Vec2::ZERO;
        }
    }
}

fn apply_velocity_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    let d = time.delta_secs();
    for (mut transform, velocity) in query {
        transform.translation += velocity.0.extend(0.) * d;
    }
}

fn enemy_kill_system(
    mut players: Query<(&mut Alive, &Transform, &Radius), With<Player>>,
    mut enemies: Query<(&Transform, &Radius), With<Enemy>>,
) {
    for (mut player_alive, player_pos, player_radius) in players {
        for (enemy_pos, enemy_radius) in enemies {
            let distance = player_pos.translation.distance(enemy_pos.translation);
            if distance - player_radius.0 - enemy_radius.0 <= 0. {
                player_alive.0 = false;
            }
        }
    }
}

