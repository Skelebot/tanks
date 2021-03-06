use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    renderer::SpriteRender,
    ecs::{
        Join, System,
        Read, WriteStorage, WriteExpect, ReadExpect,
        Entities, Entity
    }
};
use crate::utils::TanksSpriteSheet;
use crate::tank::{Tank, TankState};
use crate::physics;
use crate::weapons::Weapon;
use crate::config::TankConfig;
use crate::config::PerformanceConfig;
use crate::markers::*;

const ROCKET_SPRITE_NUM: usize = 13;
const ROCKET_WIDTH: f32 = 6.0;
const ROCKET_HEIGHT: f32 = 9.0;
const ROCKET_SELF_SAFETY_MARGIN: f32 = 7.5;
const ROCKET_VELOCITY: f32 = 50.0;
const ROCKET_MARGIN: f32 = 0.5;
const ROCKET_SHOOT_TIME: f32 = 1.0;
const ROCKET_ACCELERATION: f32 = 3.0;

pub struct RocketSystem;

impl<'s> System<'s> for RocketSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'s, Tank>,
        WriteExpect<'s, physics::Physics>,
        WriteStorage<'s, physics::Body>,
        WriteStorage<'s, physics::Collider>,

        Read<'s, Time>,
        Entities<'s>,

        WriteStorage<'s, Transform>,
        ReadExpect<'s, TanksSpriteSheet>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, DeadlyMarker>,
        WriteStorage<'s, AcceleratingMarker>,

        ReadExpect<'s,  TankConfig>,
        ReadExpect<'s,  PerformanceConfig>,
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
            sprite_sheet,
            mut sprite_renders,
            mut temp_markers,
            mut deadly_markers,
            mut accelerating_markers,
            tank_config,
            performance_config,
        ): Self::SystemData,
    ) {
        // Entities and Bodies to be added to them because we can't borrow bodies twice in the same scope
        let mut bodies_to_add: Vec<(Entity, physics::Body)> = Vec::new();
        for (tank, body) in (&mut tanks, &bodies).join() {
            if let Weapon::Rocket {
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
                            let toi = (tank_config.size_y as f32/2.0) + ROCKET_SELF_SAFETY_MARGIN + performance_config.wallscan_toi_mod;
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
                            body.position().translation.vector + body.position().rotation * na::Vector2::new(0.0, (tank_config.size_y as f32 / 2.0) + ROCKET_SELF_SAFETY_MARGIN),
                            body.position().rotation.angle(),
                        );

                        let vel_vec = body.position().rotation * na::Vector2::new(0.0, ROCKET_VELOCITY);
                        let velocity = np::algebra::Velocity2::new(
                            na::Vector2::new(vel_vec.x, vel_vec.y), 0.0
                        );

                        let shape = nc::shape::ShapeHandle::new(
                            nc::shape::Cuboid::new(
                                na::Vector2::new(ROCKET_WIDTH, ROCKET_HEIGHT)
                            ));

                        let body = np::object::RigidBodyDesc::new()
                            .position(pos)
                            .velocity(velocity)
                            .build();

                        let body_handle = physics.add_rigid_body(body);
                        let collider = np::object::ColliderDesc::new(shape)
                            .ccd_enabled(true)
                            .margin(ROCKET_MARGIN)
                            .density(10.0)
                            .build(np::object::BodyPartHandle(body_handle, 0));
                        let collider_handle = physics.add_collider(collider);

                        let sprite_render = SpriteRender {
                            sprite_number: ROCKET_SPRITE_NUM,
                            sprite_sheet: sprite_sheet.handle.clone(),
                        };
                        let mut transform = Transform::default();
                        transform.set_translation_x(-200.0);
                        let ent = entities
                            .build_entity()
                            .with(transform, &mut transforms)
                            .with(sprite_render, &mut sprite_renders)
                            .with(physics::Collider::new(collider_handle), &mut colliders)
                            // We would do that but we already borrowed bodies, so we have to build the entity now and add the body later
                            //.with(physics::Body{handle: body_handle}, &mut bodies)
                            .with(TempMarker(None), &mut temp_markers)
                            .with(AcceleratingMarker(ROCKET_ACCELERATION), &mut accelerating_markers)
                            .with(DeadlyMarker, &mut deadly_markers)
                            .build();
                        bodies_to_add.push((ent, physics::Body::new(body_handle)));
                        // Start the shooting timer
                        shooting_timer.replace(ROCKET_SHOOT_TIME);
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