use dodgescrape2::*;

fn main() {
    App::new()
        .insert_resource(CursorPos(Vec2::ZERO))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_enemies)
        .add_systems(Update, (cursor_position_system, player_movement_system, apply_velocity_system, enemy_kill_system))
        .run();
}

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
