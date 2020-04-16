use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use rand::{thread_rng, Rng};
use rand::distributions::{Distribution, Uniform};

use amethyst::{
    ecs::{
        Entities, Join, System,
        WriteStorage, WriteExpect, ReadExpect,
    },
    renderer::SpriteRender,
    core::transform::Transform,
};

use crate::utils::TanksSpriteSheet;
use crate::tank::{Tank, Team, TankState};
use crate::physics;
use crate::markers::*;
use crate::level::MazeLevel;
use crate::scoreboard::Scoreboard;

// TODO_H: Use a config
const PARTICLE_SPRITE_NUMS: [usize; 3] = [6, 7, 8];
const RED_PARTICLE_SPRITE_NUMS: [usize; 2] = [9, 10];
const BLUE_PARTICLE_SPRITE_NUMS: [usize; 2] = [11, 12];
const PARTICLE_DAMPING: f32 = 0.3;
const PARTICLE_TANK_NUM: u32 = 12;
// TODO_VL: Make it possible to explode things like walls
// const PARTICLE_OTHER_NUM: u32 = 5;
const PARTICLE_MIN_VEL: f32 = 400.0;
const PARTICLE_MAX_VEL: f32 = 450.0;
const PARTICLE_SCALE: f32 = 4.0;
const PARTICLE_DENSITY: f32 = 10.0;

const LEVEL_RESET_DELAY: f32 = 3.0;

pub struct DestroySystem;

impl<'s> System<'s> for DestroySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'s>,
        WriteExpect<'s, physics::Physics>,
        WriteStorage<'s, physics::Body>,
        WriteStorage<'s, physics::Collider>,

        WriteStorage<'s, Tank>,

        ReadExpect<'s, TanksSpriteSheet>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, DeadlyMarker>,

        // TODO: Make a level reset timer Resource so that we don't have to fetch the whole level
        WriteExpect<'s, MazeLevel>,

        WriteExpect<'s, Scoreboard>,
    );

    fn run (
        &mut self,
        (
            entities,
            mut physics,
            mut bodies,
            mut colliders,
            mut tanks,
            sprite_sheet,
            mut sprite_renders,
            mut transforms,
            mut temp_markers,
            deadly_markers,
            mut level,
            mut scoreboard,
        ): Self::SystemData
    ) {
        // Check for tanks colliding with entities marked with DeadlyMarker
        // Score for the tank's team
        // and reset the level
        physics.maintain();
        for (collider, _) in (&colliders, &deadly_markers).join() {
            if let Some(interactions) = 
                physics.geom_world.interactions_with(&physics.colliders, collider.handle, true)
            {
                for interaction in interactions {
                    // interaction is (collider_handle, collider, collider1_handle, collider1, Interaction)
                    // The first collider is the one that hit the second, so the first will be our
                    // deadly entity (for example a bullet or a laser beam)
                    // and the second will be a wall, a tank or something else
                    // We don't need the deadly collider handle, we only care what it hit
                    // let deadly_handle = interaction.0;
                    let hit_handle = interaction.2;
                    // Match tank to collider handle and determine it's team
                    for (tank, tank_collider) in (&mut tanks, &colliders).join() {
                        if hit_handle == tank_collider.handle {
                            // Tell the scoreboard the tank lost the round
                            scoreboard.report_destroyed(tank.team);
                            // We change the tank's state to 'Hit' so that the following code 
                            // will do the explosion and stuff
                            tank.state = TankState::Hit;
                        }
                    }
                }
            }
        }
        // Explode tanks that were hit, then change their state to destroyed
        // Position, angle, velocity, spriterender
        let mut particles = Vec::<(na::Vector2::<f32>, f32, f32, SpriteRender)>::new();

        for (tank, body) in (&mut tanks, &bodies).join() {
            // TODO: Use a config for all of this
            if tank.state != TankState::Hit { continue; }

            let mut thread_rng = thread_rng();

            // Create debris particles with random SpriteRenders and velocity vectors
            let sprite_numbers = match tank.team {
                Team::Red => [&PARTICLE_SPRITE_NUMS[..], &RED_PARTICLE_SPRITE_NUMS[..]].concat(),
                Team::Blue => [&PARTICLE_SPRITE_NUMS[..], &BLUE_PARTICLE_SPRITE_NUMS[..]].concat(),
            };
            // Use uniform distribution
            let numbers = Uniform::new(0, sprite_numbers.len());
            let angles = Uniform::new(0.0_f32, 360.0_f32);
            for _ in 0..PARTICLE_TANK_NUM {
                let sprite_render = SpriteRender {
                    sprite_sheet: sprite_sheet.handle.clone(),
                    sprite_number: sprite_numbers[numbers.sample(&mut thread_rng)]
                };
                // TODO_L: Weight the angle using the direction from which the tank was hit
                //       so that the particles fly in the opposite direction
                // Angle at which the projectile will be thrown
                let angle = angles.sample(&mut thread_rng);
                let position = 
                    physics.get_rigid_body(body.handle).unwrap().position().translation.vector
                    + na::Vector2::new(thread_rng.gen_range(-3.0, 3.0), thread_rng.gen_range(-3.0, 3.0));
                let velocity = thread_rng.gen_range(PARTICLE_MIN_VEL, PARTICLE_MAX_VEL);
                particles.push((position, angle, velocity, sprite_render));
            }

            let rb = physics.get_rigid_body_mut(body.handle).unwrap();
            // Hide the tank
            // In order to hide the tank the easiest method is to move it off-screen just before disabling it
            // We assume the tank isn't bigger than 100 pixels
            // Also this shouldn't be in absolute pixels because we may want to move the camera
            // TODO_H: Find a better way to hide sprites
            let new_pos = na::Isometry2::new(
                na::Vector2::new(-100.0, -100.0),
                rb.position().rotation.angle()
            );
            rb.set_position(new_pos);
            // Deactivate the tank's RigidBody
            use nphysics2d::object::Body;
            physics.get_rigid_body_mut(body.handle).unwrap().set_status(np::object::BodyStatus::Disabled);
            // Start the level reset countdown
            level.reset_timer.replace(LEVEL_RESET_DELAY);
            tank.state = TankState::Destroyed;
        }

        // Create the particles
        for (start, angle, velocity, sprite_render) in particles {
            // Create the body's position
            let position = na::Isometry2::new(
                start,
                angle
            );

            // Determine the velocity vector
            let vel = na::UnitComplex::new(angle) * na::Vector2::new(0.0, velocity);
            // Create the RigidBody
            let mut particle_rb_desc = np::object::RigidBodyDesc::new();
            particle_rb_desc
                .set_linear_damping(PARTICLE_DAMPING)
                .set_angular_damping(PARTICLE_DAMPING)
                .set_velocity(np::algebra::Velocity2::linear(vel.x, vel.y));
            // TODO_M: Dynamic/kinematic particles choice for performance
            let body = physics::Body { handle: physics.add_rigid_body(particle_rb_desc.position(position).build()) };

            // Create the collider
            let particle_collider_desc =
                np::object::ColliderDesc::new(nc::shape::ShapeHandle::new(
                    nc::shape::Cuboid::new(na::Vector2::new(
                        PARTICLE_SCALE/2.0,
                        PARTICLE_SCALE/2.0,
                    ))
                ))
                .density(PARTICLE_DENSITY);

            let collider = physics::Collider {
                handle: physics.add_collider(particle_collider_desc.build(
                    np::object::BodyPartHandle(body.handle, 0)
                ))
            };

            // Create the transform
            let mut transform= Transform::default();
            transform.set_translation_xyz(start.x, start.y, 0.0);
            transform.set_scale(amethyst::core::math::Vector3::new(PARTICLE_SCALE, PARTICLE_SCALE, 1.0));

            // Create the entity
            entities.build_entity()
                .with(sprite_render, &mut sprite_renders)
                .with(transform, &mut transforms)
                .with(body, &mut bodies)
                .with(collider, &mut colliders)
                .with(TempMarker(None), &mut temp_markers)
                .build();
        }
    }
}
