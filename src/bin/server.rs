use std::net::UdpSocket;

use dodgescrape2::*;

#[derive(Resource)]
pub struct ServerSocket {
    pub socket: UdpSocket,
    pub buf: [u8; 1500],
}

impl ServerSocket {
    pub fn new(
        socket: UdpSocket,
    ) -> Self {
        Self {
            socket,
            buf: [0; 1500],
        }
    }
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:7878").unwrap();
    socket.set_nonblocking(true).unwrap();
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ServerSocket::new(socket))
        .add_systems(Startup, (setup, spawn_enemies))
        .add_systems(Update, (receive_messages, apply_velocity_system, enemy_kill_system))
        .run();
}

fn receive_messages(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut server_socket: ResMut<ServerSocket>,
) {
    let ServerSocket { socket, buf } = &mut *server_socket;

    if let Ok((len, addr)) = socket.recv_from(buf) {
        let client_message_option = ClientMessage::decode(buf);
        match client_message_option {
            Some(client_message) => match client_message {
                ClientMessage::Login => {
                    commands.spawn((
                        Transform::from_xyz(200., 0., 1.),
                        Player,
                        Alive(true),
                        Radius(20.),
                        Velocity(Vec2::new(0., 0.)),
                        Mesh2d(meshes.add(Circle::new(20.))),
                        MeshMaterial2d(materials.add(Color::srgb(0., 1., 0.))),
                    ));
                },
            },
            None => todo!(),
        }
    }
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.),
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        DebandDither::Enabled,
    ));
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
    query: Query<(&mut Transform, &Velocity)>,
) {
    let d = time.delta_secs();
    for (mut transform, velocity) in query {
        transform.translation += velocity.0.extend(0.) * d;
    }
}

fn enemy_kill_system(
    players: Query<(&mut Alive, &Transform, &Radius), With<Player>>,
    enemies: Query<(&Transform, &Radius), With<Enemy>>,
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
