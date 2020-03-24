use serde::{Serialize, Deserialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TankConfig {
    pub size_x: u32,
    pub size_y: u32,
    pub density: f32,
    pub linear_accel: f32,
    pub angular_accel: f32,
    pub max_linear_vel: f32,
    pub max_angular_vel: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

impl Default for TankConfig {
    fn default() -> Self {
        panic!("Couldn't load TankConfig");
    }
}