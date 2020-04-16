use nphysics2d as np;
use nalgebra as na;
use ncollide2d as nc;

use rand::{thread_rng, Rng};
use rand::distributions::{Distribution, Uniform};

use amethyst::{
    core::Transform,
    renderer::SpriteRender,
    window::ScreenDimensions,
    core::timing::Time,
};
use amethyst::ecs::prelude::*;
use crate::level::MazeLevel;
use crate::tank::{Tank, Team, TankState};
use crate::markers::TempMarker;
use crate::utils::SpawnsSpriteSheet;
use crate::physics;
use crate::weapons::Weapon;
use crate::config::MazeConfig;

const SPAWN_TIME: f32 = 3.0;
const MAX_SPAWNS: u16 = 5;

pub enum SpawnType {
    Weapon( Weapon ),
    _Unused,
}
pub struct Spawn {
    pub s_type: SpawnType,
}

impl Component for Spawn {
    type Storage = VecStorage<Self>;
}

pub struct SpawnSystem {
    spawn_timer: f32,
    spawns_alive: u16,
}

impl Default for SpawnSystem {
    fn default() -> Self {
        Self {
            spawn_timer: SPAWN_TIME,
            spawns_alive: 0,
        }
    }
}

impl<'s> System<'s> for SpawnSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'s, MazeLevel>,
        Entities<'s>,
        ReadExpect<'s, SpawnsSpriteSheet>,
        
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Transform>,
        WriteExpect<'s, physics::Physics>,
        WriteStorage<'s, physics::Body>,
        WriteStorage<'s, physics::Collider>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, Tank>,
        WriteStorage<'s, Spawn>,
        Read<'s, MazeConfig>,

        ReadExpect<'s, ScreenDimensions>,
        Read<'s, Time>,
    );

    fn run(
        &mut self,
        (
            level,
            entities,
            sprite_sheet,
            mut sprite_renders,
            mut transforms,
            mut physics,
            mut bodies,
            mut colliders,
            mut temp_markers,
            mut tanks,
            mut spawns,
            maze_config,
            screen_dimensions,
            time
        ): Self::SystemData,
    ) {
        // Count down to the next spawn only if there are less spawns than MAX_SPAWNS
        // This prevents the timer from still counting down even if the system can't spawn anymore,
        // also prevents instant spawns after MAX_SPAWNS has been reached, because the timer gets frozen
        // on SPAWN_TIME and starts counting down only when spawns_alive counts down.
        if self.spawns_alive <= MAX_SPAWNS {
            self.spawn_timer -= time.delta_seconds();
        }
        if self. spawn_timer <= 0.0 {
            // Spawn a spawn
            // Determine the location
            // We want spawns to appear in the middle of cells
            let x_shift = (screen_dimensions.width() / 2.0) - ((level.maze.width as f32 * maze_config.cell_width) / 2.0) + (maze_config.cell_width / 2.0);
            let y_shift = (screen_dimensions.height() / 2.0) - ((level.maze.height as f32 * maze_config.cell_height) / 2.0) + (maze_config.cell_height / 2.0);
            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet.handle.clone(),
                sprite_number: 0,
            };
            // Increase spawns_alive
        }
        // Check for collisions with spawns
        // Important info: Performance-wise it would be much better to just check if a tank's square intersects
        // the center of the spawn - then the spawns wouldn't have rigidbodies or colliders, resulting in less
        // stress on the physics engine. The only downside would be that the tank has to reach the center of the
        // spawn to actually trigger it, but again it could be solved by also checking against the corner points
        // of a spawn.
        // We choose to use sensor colliders with static rigidbodies for more uniform code.
        // TODO: Add a performance setting
        for (spawn, collider) in (&spawns, &colliders).join() {
            if let Some(interactions) =
                physics.geom_world.interactions_with(&physics.colliders, collider.handle, true)
            {
                for interaction in interactions {
                    // interaction is (collider_handle, collider, collider1_handle, collider1, Interaction)
                    let hit_handle = interaction.2;
                    // Match tank to collider handle and change it's weapon
                    for (tank, tank_collider) in (&mut tanks, &colliders).join() {
                        if hit_handle == tank_collider.handle {
                            // Change the tank's weapon or something else depending on the spawn's type
                            match &spawn.s_type {
                                SpawnType::Weapon(spawn_weapon) => tank.weapon = spawn_weapon.clone(),
                                _ => (),
                            }
                            // Remove the spawn
                        }
                    }
                }
            }
        }
    }
}

/// Randomize a spawn
/// # returns
/// * 0: A spawn
/// * 1: A coresponding sprite number to use with a SpawnsSpriteSheet
fn random_spawn() -> (Spawn, usize) {
    let mut thread_rng = thread_rng();
    let numbers = Uniform::new(0, 10);

    let spawn = Spawn {
        s_type: SpawnType::Weapon(Weapon::random())
    };

    // TODO_M: Use a Config
    let sprite_num = match &spawn.s_type {
        SpawnType::Weapon( weapon ) => {
            match &weapon {
                Weapon::Cannon { .. } => 0,
                Weapon::Beamer { .. } => 1,
                _ => 3,
            }
        }
        _ => 3
    };
    
    (spawn, sprite_num)
}