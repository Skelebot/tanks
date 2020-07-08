use amethyst::ecs::{Component, DenseVecStorage};
use crate::weapons::Weapon;

/// An Enum representing possible teams for tanks
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Team {
    P1,
    P2,
    // TODO_H: Add support for 4 tanks
}

#[derive(PartialEq, Eq, Debug)]
/// An enum describing the state of a tank
pub enum TankState {
    /// The tank is alive - can be controlled by a player and can shoot
    Alive,
    /// The tank has just been hit and is designated to be exploded in this frame
    Hit,
    /// The tank is not visible, cannot be moved and cannot shoot
    Destroyed,
}

/// A Component carrying information about a player's tank
pub struct Tank {
    pub team: Team,
    pub weapon: Weapon,
    pub is_shooting: bool,
    pub state: TankState,
}

impl Tank {
    pub fn new(team: Team, weapon: Weapon) -> Self {
        Tank {
            team,
            weapon,
            is_shooting: false,
            state: TankState::Alive,
        }
    }
}

impl Component for Tank {
    type Storage = DenseVecStorage<Self>;
}