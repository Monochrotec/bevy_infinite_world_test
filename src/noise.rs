use bevy::prelude::*;
use noise::{Fbm, Perlin};

#[derive(Resource)]
pub struct Noises {
    pub terrain_grass: Fbm<Perlin>,
    pub terrain_stone: Fbm<Perlin>,
}
