use std::net::UdpSocket;
use std::collections::HashMap;

use dodgescrape2::*;

fn main() {
    App::new()
        .insert_resource(ClientSocket::new())
        .insert_resource(CursorPos(Vec2::ZERO))
        .insert_resource(EntityMap::default())
        .insert_resource(NetIDMap::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (receive_messages, cursor_position_system, player_movement_system))
        .run();
}

#[derive(Resource)]
pub struct ClientSocket {
    pub socket: UdpSocket,
    pub buf: [u8; 1000],
}

#[derive(Resource, Default)]
struct NetIDMap(HashMap<Entity, NetIDType>);
#[derive(Resource, Default)]
struct EntityMap(HashMap<NetIDType, Entity>);

#[derive(Component)]
struct Controlled;

impl ClientSocket {
	pub fn new() -> Self {
	    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
	    socket.set_nonblocking(true).unwrap();
		Self {
			socket,
		    buf: [0; 1000],
		}
	}
	pub fn send(&self, bytes: &[u8]) {
		self.socket.send_to(bytes, "127.0.0.1:7878").unwrap();
	}
}

fn setup(
	socket: Res<ClientSocket>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
	let login_message = ClientMessage::Login;
	socket.send(&login_message.encode());

    // commands.spawn((
    //     Camera2d,
    //     Camera {
    //         clear_color: ClearColorConfig::Custom(Color::BLACK),
    //         ..default()
    //     },
    //     Tonemapping::TonyMcMapface,
    //     Bloom::default(),
    //     DebandDither::Enabled,
    //     Transform::from_xyz(200., 0., 1.),
    //     Player,
    //     Alive(true),
    //     Radius(20.),
    //     Velocity(Vec2::new(0., 0.)),
    //     Mesh2d(meshes.add(Circle::new(20.))),
    //     // 3. Put something bright in a dark environment to see the effect
    //     MeshMaterial2d(materials.add(Color::srgb(0., 1., 0.))),
    // ));
}

fn cursor_position_system(
    window: Single<&Window, With<PrimaryWindow>>,
    mut cursor: ResMut<CursorPos>,
) {
    let window_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);

    if let Some(cursor_position) = window.cursor_position() {
        cursor.0 = (cursor_position - window_center) * Vec2::new(1., -1.); // relative to center
    }
}

fn player_movement_system(
    cursor: Res<CursorPos>,
    player_query: Query<(Entity, &mut Velocity, &Alive), (With<Player>, With<Controlled>)>,
    mut client_socket: ResMut<ClientSocket>,
    mut net_id_map: Res<NetIDMap>,
) {
    for (player_entity, mut velocity, alive) in player_query {
        if alive.0 || true {
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

        let net_id = net_id_map.0.get(&player_entity).unwrap();
        client_socket.send(&ClientMessage::SetVelocity(*net_id, velocity.0.into()).encode());
    }
}

fn receive_messages(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut client_socket: ResMut<ClientSocket>,
    mut entity_map: ResMut<EntityMap>,
    mut net_id_map: ResMut<NetIDMap>,
    mut enemy_query: Query<&mut Transform, (With<Enemy>, Without<Player>)>, // without are required to exclude the queries
    mut player_query: Query<&mut Transform, (With<Player>, Without<Enemy>)>, // without are required to exclude the queries
) {
    let ClientSocket { socket, buf } = &mut *client_socket;

    while let Ok((len, addr)) = socket.recv_from(buf) {
    	let server_message_option = ServerMessage::decode(&buf[..len]);
    	match server_message_option {
	        Some(server_message) => match server_message {
	            ServerMessage::Ok(id) => {
	            	println!("player was created successfully with id {:?}", id);
	            },
	            ServerMessage::UpdateEnemies(enemies) => {
				    let mut rng = rand::rng();
	            	for enemy in enemies {
	            		// check if enemy exists on local data
	            		if let Some(enemy_entity) = entity_map.0.get(&enemy.net_id) {
	            			let enemy_transform_result = enemy_query.get_mut(*enemy_entity);
	            			match enemy_transform_result {
			                    Ok(mut enemy_transform) => {
			                    	enemy_transform.translation = enemy.position.clone().into();
			                    },
			                    Err(_) => { },
			                }
	            		}

	            		// create enemy if doesn't exist on local data
	            		if !entity_map.0.contains_key(&enemy.net_id) {
					        let material = MeshMaterial2d(materials.add(Color::srgb(
					            rng.random_range(0.0..4.0),
					            rng.random_range(0.0..4.0),
					            rng.random_range(0.0..4.0),
					        )));

					        let id = commands.spawn((
					            Mesh2d(meshes.add(Circle::new(40.))),
					            material,
					            Transform::from_translation(enemy.position.into()),
					            Velocity(Vec2::new(0., 0.)),
					            Enemy,
					            Radius(40.),
					        )).id();

					        entity_map.0.insert(enemy.net_id, id);
					        net_id_map.0.insert(id, enemy.net_id);
	            		}
	            	}
	            },
	            ServerMessage::UpdatePlayers(players) => {
	            	for player in players {
	            		// check if player exists on local data
	            		if let Some(player_entity) = entity_map.0.get(&player.net_id) {
	            			let player_transform_result = player_query.get_mut(*player_entity);
	            			match player_transform_result {
			                    Ok(mut player_transform) => {
			                    	player_transform.translation = player.position.clone().into();
			                    },
			                    Err(_) => { },
			                }
	            		}

	            		// create player if doesn't exist on local data
	            		if !entity_map.0.contains_key(&player.net_id) {
					        let id = commands.spawn((
						        Camera2d,
						        Camera {
						            clear_color: ClearColorConfig::Custom(Color::BLACK),
						            ..default()
						        },
						        Tonemapping::TonyMcMapface,
						        Bloom::default(),
						        DebandDither::Enabled,

					            Mesh2d(meshes.add(Circle::new(20.))),
					            Transform::from_translation(player.position.into()),
					            Velocity(Vec2::new(0., 0.)),
		                        MeshMaterial2d(materials.add(Color::srgb(0., 1., 0.))),
					            Player,
					            Controlled,
					            Alive(true),
					            Radius(20.),
					        )).id();

					        entity_map.0.insert(player.net_id, id);
					        net_id_map.0.insert(id, player.net_id);
	            		}
	            	}
	            },
	        },
	        None => todo!(),
	    }
    }
}

