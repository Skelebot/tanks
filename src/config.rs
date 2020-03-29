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
    pub sprite_nums: Vec<usize>,
}

impl Default for TankConfig {
    fn default() -> Self {
        panic!("Couldn't load TankConfig");
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MazeConfig {
    pub cell_width: f32,
    pub cell_height: f32,
    pub w_thickness: f32,
    pub rb_margin: f32,
    pub w_density: f32,
    pub w_damping: f32,
    pub maze_width: usize,
    pub maze_height: usize,
    pub sprite_num: usize,
    pub sprite_width: f32,
}

impl Default for MazeConfig {
    fn default() -> Self {
        panic!("Couldn't load MazeConfig");
    }
}