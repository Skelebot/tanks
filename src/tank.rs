use amethyst::ecs::{Component, DenseVecStorage};

#[derive(PartialEq, Eq, Debug)]
pub enum Team {
    Red,
    Blue
}

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