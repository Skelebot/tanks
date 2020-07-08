use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    core::math,
    renderer::resources::Tint,
    ecs::{
        Join, System,
        Read, WriteStorage, WriteExpect, ReadExpect,
        Entities, Entity
    }
};
use crate::graphics::{CircleMesh, ShapeRender};
use crate::tank::{Tank, TankState};
use crate::physics;
use crate::weapons::Weapon;
use crate::config::TankConfig;
use crate::config::CannonConfig;
use crate::config::PerformanceConfig;
use crate::markers::*;

pub struct CannonSystem;

impl<'s> System<'s> for CannonSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'s, Tank>,
        WriteExpect<'s, physics::Physics>,
        WriteStorage<'s, physics::Body>,
        WriteStorage<'s, physics::Collider>,

        Read<'s, Time>,
        Entities<'s>,

        WriteStorage<'s, Transform>,
        WriteStorage<'s, Tint>,
        WriteStorage<'s, ShapeRender>,
        WriteStorage<'s, DynamicColorMarker>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, DeadlyMarker>,

        ReadExpect<'s,  TankConfig>,
        ReadExpect<'s,  CannonConfig>,
        ReadExpect<'s,  PerformanceConfig>,
        
        ReadExpect<'s, CircleMesh>,
    );

    fn run(
        &mut self,
        (
            mut tanks,
            mut physics,
            mut bodies,
            mut colliders,
            time,
            entities,
            mut transforms,
            mut tints,
            mut shape_renders,
            mut dyn_color_markers,
            mut temp_markers,
            mut deadly_markers,
            tank_config,
            cannon_config,
            performance_config,
            circle_mesh,
        ): Self::SystemData,
    ) {
        // Entities and Bodies to be added to them because we can't borrow bodies twice in the same scope
        // TODO_O: We can't add more than 4 bullets per frame, change this to an array
        let mut bodies_to_add: Vec<(Entity, physics::Body)> = Vec::new();
        for (tank, body) in (&mut tanks, &bodies).join() {
            if let Weapon::Cannon {
                    ref mut shooting_timer,
            } = tank.weapon {
                // The player is holding the shoot button and isn't destroyed 
                if tank.is_shooting && tank.state == TankState::Alive {
                    // If the cannon is ready to shoot
                    if shooting_timer.is_none() {
                        // Shoot

                        let body = physics.get_rigid_body(body.handle).unwrap();

                        // This is probably computionally expensive
                        if performance_config.test_wallscan {
                            // Trace an infinite ray from the tank's origin to in the direction it's facing
                            let ray = nc::query::Ray {
                                origin: body.position().translation.vector.into(),
                                dir: body.position().rotation * na::Vector2::new(0.0, 1.0)
                            };
                            let toi = (tank_config.size_y as f32/2.0) + cannon_config.self_safety_margin + performance_config.wallscan_toi_mod;
                            let interferences = physics.geom_world.interferences_with_ray(
                                &physics.colliders, 
                                &ray, 
                                toi,
                                &nc::pipeline::object::CollisionGroups::new()
                            ).count();  // We only care about the number of the interactions, so we count items in the iterator

                            // The ray always intersects with the tank. We could compute the origin to be at the end of the tank's barrel,
                            // but it's much less expensive to just take the tank's origin.
                            if interferences > 1 {
                                tank.state = TankState::Hit;
                                continue;
                            }
                        }

                        let pos = na::Isometry2::new(
                            body.position().translation.vector + body.position().rotation * na::Vector2::new(0.0, (tank_config.size_y as f32 / 2.0) + cannon_config.self_safety_margin),
                            body.position().rotation.angle(),
                        );
                        let vel_vec = body.position().rotation * na::Vector2::new(0.0, cannon_config.bullet_velocity);
                        let velocity = np::algebra::Velocity2::new(
                            na::Vector2::new(vel_vec.x, vel_vec.y),
                            5.0,   // Add a spin to the bullet - fixes some errors with zero-angle collisions 
                        );
                        let shape = nc::shape::ShapeHandle::new(nc::shape::Ball::new(cannon_config.bullet_radius));
                        let body = np::object::RigidBodyDesc::new()
                            .position(pos)
                            .velocity(velocity)
                            .build();
                        let body_handle = physics.add_rigid_body(body);
                        let collider = np::object::ColliderDesc::new(shape)
                            .material(np::material::MaterialHandle::new(
                                // We use a contact model that doesn't calculate friction either way
                                np::material::BasicMaterial::new(cannon_config.bullet_restitution, 0.0))
                            )
                            .ccd_enabled(true)
                            .margin(cannon_config.bullet_margin)
                            .density(cannon_config.bullet_density)
                            .build(np::object::BodyPartHandle(body_handle, 0));
                        let collider_handle = physics.add_collider(collider);

                        let shape_render = ShapeRender {
                            mesh: circle_mesh.handle.clone(),
                        };
                        let mut transform = Transform::default();
                        transform.set_scale(math::Vector3::new(
                            cannon_config.bullet_radius, cannon_config.bullet_radius, 1.0
                        ));
                        // TODO: Is this actually doing anything
                        transform.set_translation_x(-200.0);
                        let ent = entities
                            .build_entity()
                            .with(transform, &mut transforms)
                            .with(shape_render, &mut shape_renders)
                            .with(Tint(Default::default()), &mut tints)
                            // Bullets are neutral, so we can set them to be the same color as walls or text
                            .with(DynamicColorMarker(ColorKey::Text), &mut dyn_color_markers)
                            .with(physics::Collider::new(collider_handle), &mut colliders)
                            // We would do that but we already borrowed bodies, so we have to build the entity now and add the body later
                            //.with(physics::Body{handle: body_handle}, &mut bodies)
                            .with(TempMarker(Some(cannon_config.bullet_time)), &mut temp_markers)
                            .with(DeadlyMarker, &mut deadly_markers)
                            .build();
                        bodies_to_add.push((ent, physics::Body::new(body_handle)));
                        // Start the shooting timer
                        shooting_timer.replace(cannon_config.shoot_time);
                    }
                }
                // Update
                if let Some(timer) = shooting_timer {
                    *timer -= time.delta_seconds();
                    if *timer <= 0.0 {
                        *shooting_timer = None;
                    }
                }
            }
        }
        for (entity, body) in bodies_to_add.into_iter() {
            bodies.insert(entity, body).expect("Something went wrong when adding bodies to entities");
        }
    }
}