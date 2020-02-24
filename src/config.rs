use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BulletConfig {
    pub sprite_num: usize,
    pub radius: f32,
    pub speed: f32,
    pub max_lifetime: u32,
}

impl Default for BulletConfig {
    fn default() -> Self {
        BulletConfig {
            sprite_num: 2,
            radius: 10.0,
            speed: 5.0,
            max_lifetime: 100,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputConfig {
    pub rotation_sensitivity: f32,
    pub movement_speed: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        InputConfig {
            rotation_sensitivity: 5.0,
            movement_speed: 0.2,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TankConfig {
    pub size_x: u32,
    pub size_y: u32,
    pub mass: f32,
    pub friction: f32,
    pub bounciness: f32,
}

impl Default for TankConfig {
    fn default() -> Self {
        TankConfig {
            size_x: 24,
            size_y: 24,
            mass: 0.1,
            friction: 0.1,
            bounciness: 0.5,
        }
    }
}
