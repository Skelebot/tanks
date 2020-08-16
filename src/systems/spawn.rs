use nphysics2d as np;
use nalgebra as na;
use ncollide2d as nc;

use rand::{thread_rng, Rng};
use rand::distributions::{Distribution, Uniform};

use amethyst::{
    core::Transform,
    renderer::SpriteRender,
    renderer::resources::Tint,
    window::ScreenDimensions,
    core::timing::Time,
};
use amethyst::ecs::prelude::*;
use crate::level::MazeLevel;
use crate::tank::Tank;
use crate::markers::*;
use crate::utils::SpawnsSpriteSheet;
use crate::physics;
use crate::weapons::Weapon;
use crate::config::{MazeConfig, SpawnConfig};

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
    spawn_distr: Option<Uniform<u32>>,
    taken_spawnpoints: Vec<(usize, usize)>,
}

impl Default for SpawnSystem {
    fn default() -> Self {
        Self {
            spawn_timer: 0.0,
            spawns_alive: 0,
            // We want to initialize it the first time someone calls run()
            // because we need the maze_config etc to actually initialize it
            spawn_distr: None,
            taken_spawnpoints: Vec::new(),
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
        WriteStorage<'s, Tint>,
        WriteStorage<'s, DynamicColorMarker>,
        WriteStorage<'s, Transform>,
        WriteExpect<'s, physics::Physics>,
        WriteStorage<'s, physics::Body>,
        WriteStorage<'s, physics::Collider>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, Tank>,
        WriteStorage<'s, Spawn>,

        ReadExpect<'s,  SpawnConfig>,
        ReadExpect<'s,  MazeConfig>,

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
            mut tints,
            mut dyn_color_markers,
            mut transforms,
            mut physics,
            mut bodies,
            mut colliders,
            mut temp_markers,
            mut tanks,
            mut spawns,
            spawn_config,
            maze_config,
            screen_dimensions,
            time
        ): Self::SystemData,
    ) {
        // If the level is about to be reset, zero the number of spawns
        // and keep the spawn timer frozen. This won't spawn any new spawns
        // and won't remove existing ones. When the LevelSystem resets the level,
        // all existing spawns will be automatically removed (TempMarkers)
        if level.reset_timer.is_some() {
            self.spawn_timer = spawn_config.spawn_time;
            self.spawns_alive = 0;
        }

        let mut rng = thread_rng();

        if self.spawn_distr.is_none() { self.spawn_distr.replace(Uniform::new(0, 10)); }
        if self.taken_spawnpoints.is_empty() {
            self.taken_spawnpoints.push((0, 0));
            self.taken_spawnpoints.push((level.maze.width-1, level.maze.height-1));
        }

        // Count down to the next spawn only if there are less spawns than MAX_SPAWNS
        // This prevents the timer from still counting down even if the system can't spawn anymore,
        // also prevents instant spawns after MAX_SPAWNS has been reached, because the timer gets frozen
        // on SPAWN_TIME and starts counting down only when spawns_alive counts down.
        if self.spawns_alive < spawn_config.max_spawns {
            self.spawn_timer -= time.delta_seconds();
        }
        if self. spawn_timer <= 0.0 {
            // Spawn a spawn
            // Determine the location
            // We want spawns to appear in the middle of cells
            let x_shift = (screen_dimensions.width() / 2.0) - ((level.maze.width as f32 * maze_config.cell_width) / 2.0) + (maze_config.cell_width / 2.0);
            let y_shift = (screen_dimensions.height() / 2.0) - ((level.maze.height as f32 * maze_config.cell_height) / 2.0) + (maze_config.cell_height / 2.0);

            let mut x_cell = rng.gen_range(0, level.maze.width);
            let mut y_cell = rng.gen_range(0, level.maze.height);

            // Generate new spawnpoints until we find one that isn't forbidden (taken)
            while self.taken_spawnpoints.contains(&(x_cell, y_cell)) {
                x_cell = rng.gen_range(0, level.maze.width);
                y_cell = rng.gen_range(0, level.maze.height);
            }

            let (spawn, num) = random_spawn(&mut rng, self.spawn_distr.unwrap());

            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet.handle.clone(),
                sprite_number: num,
            };

            // Transform
            let mut transform = Transform::default();
            transform.set_translation_xyz(
                x_shift + (x_cell as f32 * maze_config.cell_width as f32),
                y_shift + (y_cell as f32 * maze_config.cell_height as f32),
                -0.2
            );

            let mut spawn_rb_desc = np::object::RigidBodyDesc::new();
            spawn_rb_desc.set_status(np::object::BodyStatus::Static);
            let spawn_pos = na::Isometry2::new(
                na::Vector2::new(
                    x_shift + (x_cell as f32 * maze_config.cell_width as f32),
                    y_shift + (y_cell as f32 * maze_config.cell_height as f32)
                ),
                0.0
            );

            let rb = spawn_rb_desc.position(spawn_pos).build();
            let body = physics::Body::new(physics.add_rigid_body(rb));
            let collider_desc = 
                np::object::ColliderDesc::new(nc::shape::ShapeHandle::new(
                    nc::shape::Cuboid::new(na::Vector2::new(
                        spawn_config.spawn_size/2.0, spawn_config.spawn_size/2.0,
                    ))
                ))
                .sensor(true);
            let collider = physics::Collider::new(
                physics.add_collider(collider_desc.build(np::object::BodyPartHandle(body.handle, 0)))
            );


            // Create the entity
            entities
                .build_entity()
                .with(spawn, &mut spawns)
                .with(sprite_render, &mut sprite_renders)
                .with(Tint(Default::default()), &mut tints)
                .with(DynamicColorMarker(ColorKey::Text), &mut dyn_color_markers)
                .with(transform, &mut transforms)
                .with(body, &mut bodies)
                .with(collider, &mut colliders)
                .with(TempMarker(None), &mut temp_markers)
                .build();

            // Increase spawns_alive
            self.spawns_alive += 1;
            // Restart the timer
            self.spawn_timer = spawn_config.spawn_time;
        }

        // Spawns which were collected by tanks, so they have to be removed
        let mut spawns_to_remove: Vec<Entity> = Vec::new();
        // Check for collisions with spawns
        // Important info: Performance-wise it would be much better to just check if a tank's square intersects
        // the center of the spawn - then the spawns wouldn't have rigidbodies or colliders, resulting in less
        // stress on the physics engine. The only downside would be that the tank has to reach the center of the
        // spawn to actually trigger it, but again it could be solved by also checking against the corner points
        // of a spawn.
        // We choose to use sensor colliders with static rigidbodies for more uniform code.
        physics.maintain();
        // TODO_VL: Add a performance setting
        for (spawn, collider, entity) in (&spawns, &colliders, &entities).join() {
            if let Some(interactions) =
                physics.geom_world.interactions_with(&physics.colliders, collider.handle, false)
            {
                for interaction in interactions {
                    // interaction is (collider_handle, collider, collider1_handle, collider1, Interaction)
                    let handle0 = interaction.0;
                    let handle1 = interaction.2;
                    // Match tank to collider handle and change it's weapon
                    for (tank, tank_collider) in (&mut tanks, &colliders).join() {
                        if handle0 == tank_collider.handle || handle1 == tank_collider.handle {
                            // Change the tank's weapon or something else depending on the spawn's type
                            use std::mem::discriminant; // Returns a unique identifier for an enum variant
                                                        // which lets us check if two values are the same variant
                            #[allow(clippy::single_match)]
                            match &spawn.s_type {
                                SpawnType::Weapon(spawn_weapon) => {
                                    // Pick up only if the tank doesn't already have that weapon
                                    if !(discriminant(&tank.weapon) == discriminant(spawn_weapon)) {
                                        tank.weapon = spawn_weapon.clone();
                                        // Decrease the counter
                                        self.spawns_alive -= 1;
                                        spawns_to_remove.push(entity);
                                    }
                                },
                                _ => (),
                            }
                            // Remove the spawn
                        }
                    }
                }
            }
        }

        // Remove the spawns
        for entity in spawns_to_remove {
            // Remove bodies and colliders belonging to entities with a TempMarker Component
            if let Some(body) = bodies.get(entity) {
                physics.remove_rigid_body(body.handle);
            }
            if let Some(collider) = colliders.get(entity) {
                physics.remove_collider(collider.handle);
            }
            entities.delete(entity).expect("Couldn't remove the entity");
        }
    }
}

/// Randomize a spawn
/// Uses a given Rng to generate a random spawn
/// # returns
/// * 0: A spawn
/// * 1: A coresponding sprite number to use with a SpawnsSpriteSheet
fn random_spawn<R: Rng + ?Sized, D: Distribution<u32>>(rng: &mut R, dist: D) -> (Spawn, usize) {

    // We are sure that num is in range 0..10
    let num = dist.sample(rng);

    let s_type = match num {
        //a weapon
        _ => {
            let num = dist.sample(rng);
            match num {
                0..=7 => SpawnType::Weapon(Weapon::Cannon { shooting_timer: None }),
                // 3..=8 => SpawnType::Weapon(Weapon::Rocket { shooting_timer: None }),
                8..=10 => SpawnType::Weapon(Weapon::Beamer { shooting_timer: None, beam: None, heating_progress: 0.0, heating_square: None, overheat_timer: None }),
                _ => unreachable!(),
            }
        }
    };

    let spawn = Spawn {
        s_type
    };

    // TODO_M: Use a Config
    let sprite_num = match &spawn.s_type {
        SpawnType::Weapon( weapon ) => {
            match &weapon {
                Weapon::Cannon { .. } => 0,
                Weapon::Beamer { .. } => 1,
                Weapon::Rocket { .. } => 2,
                _ => 3,
            }
        }
        _ => 3
    };
    
    (spawn, sprite_num)
}

#[test]
/// A method to find out if tho enum values are the same variant
fn test_eq_enum_variant() {
    #[derive(Debug, PartialEq)]
    enum T {
        A{x: u32, y: u32},
    }

    let val1 = T::A{x: 0, y: 1};
    let val2 = T::A{x: 2, y: 3};

    // We need this to pass
    // assert_eq!(val1, val2);

    // Solution
    use std::mem::discriminant;
    assert_eq!(discriminant(&val1), discriminant(&val2));
}