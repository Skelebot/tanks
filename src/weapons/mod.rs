use amethyst::ecs::Entity;

#[derive(Clone)]
pub enum Weapon {
    Beamer {
        heating_progress: f32,
        shooting_timer: Option<f32>,
        overheat_timer: Option<f32>,
        heating_square: Option<Entity>,
        beam: Option<Entity>,
    },
    _Popper,
    _Railgun,
    _Shotgun,
}

impl Default for Weapon {
    fn default() -> Self {
        Self::Beamer {
            heating_progress: 0.0,
            shooting_timer: None,
            overheat_timer: None,
            heating_square: None,
            beam: None
        }
    }
}