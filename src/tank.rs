use amethyst::ecs::{Component, DenseVecStorage};

/// An Enum representing possible teams for tanks
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Team {
    Red,
    Blue
}

/// A Component carrying information about a player's tank
pub struct Tank {
    pub team: Team,
    pub weapon_timer: Option<f32>,
    //TODO: ammo, different weapons
}

impl Tank {
    pub fn new(team: Team) -> Self {
        Tank {
            team,
            weapon_timer: None
        }
    }
}

impl Component for Tank {
    type Storage = DenseVecStorage<Self>;
}