use amethyst::ecs::Entity;

#[derive(Clone)]
#[allow(dead_code)]     // When we use the random() method everywhere, the compiler tells us the variants are never constructed
pub enum Weapon {
    Beamer {
        heating_progress: f32,
        shooting_timer: Option<f32>,
        overheat_timer: Option<f32>,
        heating_square: Option<Entity>,
        beam: Option<Entity>,
    },
    Cannon {
        shooting_timer: Option<f32>,
    },
    Rocket {
        shooting_timer: Option<f32>,
    },
    _Railgun,
    _Shotgun,
}

/*impl Weapon {
    pub fn random() -> Self {
        let mut rand = thread_rng();
        let num = rand.gen_range(0, 5);
        match num {
            0 => Self::Beamer { heating_progress: 0.0, shooting_timer: None, overheat_timer: None, heating_square: None, beam: None },
            1..=4 => Self::Cannon { shooting_timer: None },
            _ => Self::default(),
        }
    }
}*/

impl Default for Weapon {
    fn default() -> Self {
        Self::Cannon {
            shooting_timer: None,
        }
    }
}