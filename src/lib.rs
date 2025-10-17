use bincode::{Decode, Encode};
pub use rand::Rng;
pub use bevy::window::PrimaryWindow;
pub use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::{Bloom, BloomCompositeMode},
    prelude::*,
};

#[derive(Resource)]
pub struct CursorPos(pub Vec2);

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Alive(pub bool);

#[derive(Component)]
pub struct Enemy;

pub fn random_velocity() -> Vec2 {
    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let speed = rng.random_range(50.0..200.0);
    Vec2::from_angle(angle) * speed
}

pub fn random_position(range: f32) -> Vec2 {
    let mut rng = rand::rng();
    Vec2::new(
        rng.random_range(-range..range),
        rng.random_range(-range..range),
    )
}

#[derive(Encode, Decode, Debug)]
pub enum ServerMessage {
	
}

impl ServerMessage {
	pub fn encode(&self) -> [u8; 100] {
		let mut slice = [0u8; 100];
		bincode::encode_into_slice(self, &mut slice, bincode::config::standard()).unwrap();
		slice
	}
	pub fn decode(slice: &[u8]) -> Option<Self> {
		let o = bincode::decode_from_slice(slice, bincode::config::standard());
		match o {
		    Ok(r) => Some(r.0),
		    Err(_) => None,
		}
	}
}

#[derive(Encode, Decode, Debug)]
pub enum ClientMessage {
	Login,
}

impl ClientMessage {
	pub fn encode(&self) -> [u8; 100] {
		let mut slice = [0u8; 100];
		bincode::encode_into_slice(self, &mut slice, bincode::config::standard()).unwrap();
		slice
	}
	pub fn decode(slice: &[u8]) -> Option<Self> {
		let o = bincode::decode_from_slice(slice, bincode::config::standard());
		match o {
		    Ok(r) => Some(r.0),
		    Err(_) => None,
		}
	}
}
