use amethyst::ecs::{Component, DenseVecStorage};
use crate::weapons::Weapon;

/// An Enum representing possible teams for tanks
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Team {
    Red,
    Blue
}

/// A Component carrying information about a player's tank
pub struct Tank {
    pub team: Team,
    pub weapon: Weapon,
    pub is_shooting: bool,
    //TODO: ammo, different weapons
}

impl Tank {
    pub fn new(team: Team, weapon: Weapon) -> Self {
        Tank {
            team,
            weapon,
            is_shooting: false
        }
    }
}

impl Component for Tank {
    type Storage = DenseVecStorage<Self>;
}