use bevy::prelude::*;
use rand::{RngExt, SeedableRng, rngs::Xoshiro256PlusPlus};

#[derive(Resource)]
pub struct Noises {
    pub terrain: Noise1d,
}

pub struct Noise1d {
    rng: Xoshiro256PlusPlus,
}

impl Noise1d {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Xoshiro256PlusPlus::seed_from_u64(seed),
        }
    }

    pub fn get(&mut self, pos: f64) -> f64 {
        let floor_point_pos = pos.floor();

        self.get(pos)

        let prev_point_num = self.rng.random_range(-1.0..=1.);
        let next_point_num = self.rng.random_range(-1.0..=1.);

        let prev_point_distance = pos - floor_point_pos;

        lerp(prev_point_distance, prev_point_num, next_point_num)
    }
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}
