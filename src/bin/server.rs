use std::net::{SocketAddr, UdpSocket};
use std::collections::HashMap;
use dodgescrape2::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:7878").unwrap();
    socket.set_nonblocking(true).unwrap();
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ServerSocket::new(socket))
        .insert_resource(IDCounter(0))
        .insert_resource(EntityMap::default())
        .insert_resource(NetIDMap::default())
        .add_systems(Startup, (setup, spawn_enemies))
        .add_systems(Update, (receive_messages, apply_velocity_system, enemy_kill_system, broadcast_enemies, broadcast_players))
        .run();
}

#[derive(Component)]
pub struct UpdateAddress {
    addr: SocketAddr,
}

#[derive(Resource, Default)]
struct NetIDMap(HashMap<Entity, NetIDType>);
#[derive(Resource, Default)]
struct EntityMap(HashMap<NetIDType, Entity>);

#[derive(Resource)]
struct IDCounter(pub NetIDType);

#[derive(Resource)]
pub struct ServerSocket {
    pub socket: UdpSocket,
    pub buf: [u8; 1000],
}

impl ServerSocket {
    pub fn new(
        socket: UdpSocket,
    ) -> Self {
        Self {
            socket,
            buf: [0; 1000],
        }
    }
    pub fn send_to(&self, bytes: &[u8], addr: SocketAddr) -> bool {
        match self.socket.send_to(bytes, addr) {
            Ok(l) => l == bytes.len(),
            Err(_) => false,
        }
    }
}

fn receive_messages(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut server_socket: ResMut<ServerSocket>,
    mut id_counter: ResMut<IDCounter>,
    mut net_id_map: ResMut<NetIDMap>,
    mut entity_map: ResMut<EntityMap>,
    mut player_query: Query<&mut Velocity, With<Player>>,
) {
    let ServerSocket { socket, buf } = &mut *server_socket;

    while let Ok((len, addr)) = socket.recv_from(buf) {
        let client_message_option = ClientMessage::decode(buf);
        match client_message_option {
            Some(client_message) => match client_message {
                ClientMessage::Login => {
                    let id = commands.spawn((
                        Transform::from_xyz(200., 0., 1.),
                        Player,
                        Alive(true),
                        Radius(20.),
                        Velocity(Vec2::new(-200., 0.)),
                        Mesh2d(meshes.add(Circle::new(20.))),
                        MeshMaterial2d(materials.add(Color::srgb(0., 1., 0.))),
                        UpdateAddress {addr},
                    )).id();

                    net_id_map.0.insert(id, id_counter.0);
                    entity_map.0.insert(id_counter.0, id);
                    socket.send_to(&ServerMessage::Ok(id_counter.0).encode(), addr);

                    id_counter.0 += 1;
                },
                ClientMessage::SetVelocity(player_net_id, velocity) => {
                    let player_entity_option = entity_map.0.get(&player_net_id);
                    let mut player_exists = false;
                    match player_entity_option {
                        Some(player_entity) => {
                            let mut player_velocity_result = player_query.get_mut(*player_entity);
                            match player_velocity_result {
                                Ok(mut player_velocity) => {
                                    player_exists = true;
                                    player_velocity.0 = velocity.into();
                                },
                                Err(_) => {},
                            }
                        },
                        None => {},
                    }
                    if !player_exists {
                        entity_map.0.remove(&player_net_id);
                    }
                },
            },
            None => todo!(),
        }
    }
}

const ENEMIES_PER_PACKAGE: usize = (1000. / std::mem::size_of::<EnemyPackage>() as f32).floor() as usize;
const PLAYERS_PER_PACKAGE: usize = (1000. / std::mem::size_of::<PlayerPackage>() as f32).floor() as usize;

fn broadcast_enemies(
    server_socket: Res<ServerSocket>,
    client_addresses: Query<(Entity, &UpdateAddress)>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut net_id_map: ResMut<NetIDMap>,
) {
    let enemy_package_vec_count = (enemy_query.iter().len() as f32 / ENEMIES_PER_PACKAGE as f32).ceil() as usize;
    let mut enemy_package_vec = Vec::<Vec<EnemyPackage>>::new();
    let mut enemy_packages: Vec<EnemyPackage> = Vec::with_capacity(ENEMIES_PER_PACKAGE);
    let mut counter = 0;
    for (enemy_entity, enemy_transform) in enemy_query {
        let net_id = net_id_map.0.get(&enemy_entity).unwrap();
        enemy_packages.push(EnemyPackage {
            net_id: *net_id,
            position: enemy_transform.translation.into(),
        });
        counter += 1;
        if counter >= ENEMIES_PER_PACKAGE {
            counter = 0;
            enemy_package_vec.push(enemy_packages);
            enemy_packages = Vec::with_capacity(ENEMIES_PER_PACKAGE);
        }
    }
    if enemy_packages.len() > 0 {
        enemy_package_vec.push(enemy_packages);
    }

    for enemy_packages in enemy_package_vec {
        let message = ServerMessage::UpdateEnemies(enemy_packages);
        let bytes = message.encode();

        for (id, addr) in client_addresses {
            server_socket.send_to(&bytes, addr.addr);
        }
    }
}

fn broadcast_players(
    server_socket: Res<ServerSocket>,
    client_addresses: Query<(Entity, &UpdateAddress)>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut net_id_map: ResMut<NetIDMap>,
) {
    let player_package_vec_count = (player_query.iter().len() as f32 / PLAYERS_PER_PACKAGE as f32).ceil() as usize;
    let mut player_package_vec = Vec::<Vec<PlayerPackage>>::new();
    let mut player_packages: Vec<PlayerPackage> = Vec::with_capacity(PLAYERS_PER_PACKAGE);
    let mut counter = 0;
    for (player_entity, player_transform) in player_query {
        let net_id = net_id_map.0.get(&player_entity).unwrap();
        player_packages.push(PlayerPackage {
            net_id: *net_id,
            position: player_transform.translation.into(),
        });
        counter += 1;
        if counter >= PLAYERS_PER_PACKAGE {
            counter = 0;
            player_package_vec.push(player_packages);
            player_packages = Vec::with_capacity(PLAYERS_PER_PACKAGE);
        }
    }
    if player_packages.len() > 0 {
        player_package_vec.push(player_packages);
    }

    for player_packages in player_package_vec {
        let message = ServerMessage::UpdatePlayers(player_packages);
        let bytes = message.encode();

        for (id, addr) in client_addresses {
            server_socket.send_to(&bytes, addr.addr);
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
    mut id_counter: ResMut<IDCounter>,
    mut net_id_map: ResMut<NetIDMap>,
    mut entity_map: ResMut<EntityMap>,
) {
    let mut rng = rand::rng();
    for _ in 0..10 {
        let velocity = Velocity(random_velocity());
        let position = random_position(100.);
        let material = MeshMaterial2d(materials.add(Color::srgb(
            rng.random_range(0.0..4.0),
            rng.random_range(0.0..4.0),
            rng.random_range(0.0..4.0),
        )));

        // Circle mesh
        let id = commands.spawn((
            Mesh2d(meshes.add(Circle::new(40.))),
            // 3. Put something bright in a dark environment to see the effect
            material,
            Transform::from_translation(position.extend(0.)),
            velocity,
            Enemy,
            Radius(40.),
        )).id();

        net_id_map.0.insert(id, id_counter.0);
        entity_map.0.insert(id_counter.0, id);
        id_counter.0 += 1;
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
