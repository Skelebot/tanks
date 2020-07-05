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
    renderer::{
        resources::Tint,
        SpriteRender,
    },
    core::transform::Transform,
};

use crate::utils::TanksSpriteSheet;
use crate::tank::{Tank, Team, TankState};
use crate::physics;
use crate::markers::*;
use crate::level::MazeLevel;
use crate::scoreboard::Scoreboard;
use crate::systems::camshake::CameraShake;
use crate::config::DestroyConfig;
use crate::config::PerformanceConfig;

// TODO_VL: Make it possible to explode things like walls

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
        WriteStorage<'s, Tint>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, DeadlyMarker>,

        // TODO_L: Make a level reset timer Resource so that we don't have to fetch the whole level
        WriteExpect<'s, MazeLevel>,

        WriteExpect<'s, Scoreboard>,

        WriteExpect<'s, CameraShake>,
        ReadExpect<'s, DestroyConfig>,
        ReadExpect<'s, PerformanceConfig>,
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
            mut tints,
            mut transforms,
            mut temp_markers,
            deadly_markers,
            mut level,
            mut scoreboard,
            mut cam_shake,
            destroy_config,
            performance_config,
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
                        // We can't destroy a tank that's already destroyed
                        // This shouldn't even happen because we deactivate the rigidbody of dead tanks,
                        // but somehow still does. This didn't happen back when we were moving dead tanks
                        // out of frame, but now we only deactivate and hide them, so this is (somehow) necessary.
                        if tank.state != TankState::Alive { continue; }
                        if hit_handle == tank_collider.handle {
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

        for (tank, body, tint) in (&mut tanks, &bodies, &mut tints).join() {
            if tank.state != TankState::Hit { continue; }
            // Tell the scoreboard the tank lost the round
            scoreboard.report_destroyed(tank.team);

            if destroy_config.particles_enabled {
                let mut thread_rng = thread_rng();

                // Create debris particles with random SpriteRenders and velocity vectors
                let sprite_numbers = match tank.team {
                    Team::P1 => [&destroy_config.particle_sprite_nums[..], &destroy_config.red_particle_sprite_nums[..]].concat(),
                    Team::P2 => [&destroy_config.particle_sprite_nums[..], &destroy_config.blue_particle_sprite_nums[..]].concat(),
                };
                // Use uniform distribution
                let numbers = Uniform::new(0, sprite_numbers.len());
                let angles = Uniform::new(0.0_f32, 360.0_f32);
                for _ in 0..destroy_config.tank_explosion_particle_num {
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

                    let velocity = thread_rng.gen_range(
                        destroy_config.particle_vel_bounds.0, 
                        destroy_config.particle_vel_bounds.1
                    );

                    particles.push((position, angle, velocity, sprite_render));
                }
            }

            let rb = physics.get_rigid_body_mut(body.handle).unwrap();

            // "Hide" the tank
            // Deactivate the tank's RigidBody
            use nphysics2d::object::Body;
            rb.set_status(np::object::BodyStatus::Disabled);
            // Hide the tank's sprite
            // This actually sets the sprite's transparency to 100%, making it invisible,
            // though the sprite and rigidbody are still there.
            tint.0.alpha = 0.0;

            // Set the tank's state to Destroyed
            tank.state = TankState::Destroyed;

            if destroy_config.shake_enabled {
                // Start shaking the camera
                cam_shake.dms.push((destroy_config.tank_explosion_shake_duration, destroy_config.tank_explosion_shake_magnitude));
            }

            // Start the level reset countdown
            level.reset_timer.replace(destroy_config.level_reset_delay);
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
                .set_linear_damping(destroy_config.particle_damping)
                .set_angular_damping(destroy_config.particle_damping)
                .set_velocity(np::algebra::Velocity2::linear(vel.x, vel.y));
            // TODO_M: Dynamic/kinematic particles choice for performance
            let body = physics::Body { handle: physics.add_rigid_body(particle_rb_desc.position(position).build()) };


            // Create the transform
            let mut transform= Transform::default();
            transform.set_translation_xyz(start.x, start.y, 0.0);
            transform.set_scale(amethyst::core::math::Vector3::new(destroy_config.particle_scale, destroy_config.particle_scale, 1.0));

            // Create the entity
            let mut builder = entities.build_entity()
                .with(sprite_render, &mut sprite_renders)
                .with(transform, &mut transforms)
                .with(TempMarker(None), &mut temp_markers);

                if performance_config.dynamic_particles {
                    // Create the collider
                    let particle_collider_desc =
                        np::object::ColliderDesc::new(nc::shape::ShapeHandle::new(
                            nc::shape::Cuboid::new(na::Vector2::new(
                                destroy_config.particle_scale/2.0,
                                destroy_config.particle_scale/2.0,
                            ))
                        ))
                        .density(destroy_config.particle_density);

                    let collider = physics::Collider {
                        handle: physics.add_collider(particle_collider_desc.build(
                            np::object::BodyPartHandle(body.handle, 0)
                        ))
                    };
                    builder = builder.with(collider, &mut colliders);
                }

            builder.with(body, &mut bodies).build();
        }
    }
}
