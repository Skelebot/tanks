use amethyst::ecs::{Component, VecStorage};
use crate::weapons::Weapon;
use crate::input;

/// An Enum representing possible teams for tanks
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Team {
    P1,
    P2,
    // TODO_H: Add support for 4 tanks
}
impl Into<input::PlayerId> for Team {
    fn into(self) -> input::PlayerId {
        match self {
            Team::P1 => 0,
            Team::P2 => 1,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Class {
    Breach,
    Trophy,
    Snipe,
    Sombra,
    Raze,
    _Despacito,
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
    // The tank is stunned (by Sombra)
    Stunned,
}

/// A Component carrying information about a player's tank
pub struct Tank {
    pub team: Team,
    pub weapon: Weapon,
    pub class: Class,
    pub ability_refresh: Option<f32>,
    pub is_shooting: bool,
    pub is_using_ability: bool,
    pub state: TankState,
}

impl Tank {
    pub fn new(team: Team, weapon: Weapon, class: Class) -> Self {
        Tank {
            team,
            weapon,
            class,
            ability_refresh: None,
            is_shooting: false,
            is_using_ability: false,
            state: TankState::Alive,
        }
    }
}

impl Component for Tank {
    type Storage = VecStorage<Self>;
}