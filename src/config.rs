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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MazeConfig {
    pub cell_width: f32,
    pub cell_height: f32,
    pub w_thickness: f32,
    pub rb_margin: f32,
    pub w_density: f32,
    pub w_damping: f32,
    pub dynamic_walls: bool,
    pub maze_width: usize,
    pub maze_height: usize,
    pub sprite_num: usize,
    pub sprite_width: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpawnConfig {
    pub spawn_time: f32,
    pub max_spawns: u16,
    pub spawn_size: f32,
    // TODO_F: Spawn chances, spawn sprite numbers
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BeamerConfig {
    pub heat_time: f32,
    pub heating_max_scale: f32,
    pub beam_width: f32,
    pub shoot_time: f32,
    pub overheat_time: f32,
    pub self_safety_margin: f32,
    pub shake_magnitude: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CannonConfig {
    pub shoot_time: f32,
    pub bullet_time: f32,
    pub self_safety_margin: f32,
    pub bullet_density: f32,
    pub bullet_margin: f32,
    pub bullet_radius: f32,
    pub bullet_velocity: f32,
    pub bullet_restitution: f32,
    pub bullet_sprite_num: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceConfig {
    pub test_wallscan: bool,
    pub wallscan_toi_mod: f32,
    pub dynamic_particles: bool,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DestroyConfig {
    pub particles_enabled: bool,
    pub shake_enabled: bool,
    pub particle_sprite_nums: [usize; 3],
    pub red_particle_sprite_nums: [usize; 2],
    pub blue_particle_sprite_nums: [usize; 2],
    pub particle_damping: f32,
    pub tank_explosion_particle_num: usize,
    pub particle_vel_bounds: (f32, f32),
    pub particle_scale: f32,
    pub particle_density: f32,

    pub level_reset_delay: f32,

    pub tank_explosion_shake_duration: f32,
    pub tank_explosion_shake_magnitude: f32,
}