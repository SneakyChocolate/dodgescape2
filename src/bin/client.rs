use std::net::UdpSocket;
use std::collections::HashMap;

use dodgescrape2::*;

fn main() {
    App::new()
        .insert_resource(ClientSocket::new())
        .insert_resource(CursorPos(Vec2::ZERO))
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

#[derive(Resource)]
struct NetIDMap(HashMap<NetIDType, Entity>);

impl ClientSocket {
	pub fn new() -> Self {
		Self {
			socket: UdpSocket::bind("0.0.0.0:0").unwrap(),
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
    query: Query<(&mut Velocity, &Alive), With<Player>>,
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

fn receive_messages(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut client_socket: ResMut<ClientSocket>,
) {
    let ClientSocket { socket, buf } = &mut *client_socket;

    if let Ok((len, addr)) = socket.recv_from(buf) {
    	let server_message_option = ServerMessage::decode(buf);
    	match server_message_option {
	        Some(server_message) => match server_message {
	            ServerMessage::Ok => {
	            	println!("player was created successfully");
	            },
	            ServerMessage::UpdateEnemies(enemies) => todo!(),
	        },
	        None => todo!(),
	    }
    }
}
