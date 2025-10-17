use dodgescrape2::*;

fn main() {
    App::new()
        .insert_resource(CursorPos(Vec2::ZERO))
        .add_systems(Startup, spawn_enemies)
        .add_systems(Update, (apply_velocity_system, enemy_kill_system))
        .run();
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
