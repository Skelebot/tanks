use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    renderer::SpriteRender,
    ecs::{
        Join, System,
        Read, WriteStorage, WriteExpect,
        Entities, Entity
    }
};
use crate::utils::SpriteSheetRes;
use crate::tank::{Tank, TankState};
use crate::physics;
use crate::weapons::Weapon;
use crate::config::TankConfig;
use crate::markers::*;

const CANNON_SHOOT_TIME: f32 = 0.2;
const CANNON_SELF_SAFETY_MARGIN: f32 = 3.0;
const CANNON_BULLET_SPRITE_NUM: usize = 2;
const CANNON_BULLET_DENSITY: f32 = 25.0;
const CANNON_BULLET_MARGIN: f32 = 1.0;
const CANNON_BULLET_RADIUS: f32 = 3.0;
const CANNON_BULLET_VELOCITY: f32 = 100.0;
const CANNON_BULLET_RESTITUTION: f32 = 2.0;
const CANNON_BULLET_FRICTION: f32 = 0.0;

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
        Read<'s, SpriteSheetRes>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, DeadlyMarker>,

        Read<'s, TankConfig>,
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
            tank_config,
        ): Self::SystemData,
    ) {

        // Entities and Bodies to be added to them because we can't borrow bodies twice in the same scope
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
                        let pos = na::Isometry2::new(
                            body.position().translation.vector + body.position().rotation * na::Vector2::new(0.0, (tank_config.size_y as f32 / 2.0) + CANNON_SELF_SAFETY_MARGIN),
                            body.position().rotation.angle(),
                        );
                        let vel_vec = body.position().rotation * na::Vector2::new(0.0, CANNON_BULLET_VELOCITY);
                        let velocity = np::algebra::Velocity2::linear(vel_vec.x, vel_vec.y);
                        let shape = nc::shape::ShapeHandle::new(nc::shape::Ball::new(CANNON_BULLET_RADIUS));
                        let body = np::object::RigidBodyDesc::new()
                            .position(pos)
                            .velocity(velocity)
                            .build();
                        let body_handle = physics.add_rigid_body(body);
                        let collider = np::object::ColliderDesc::new(shape)
                            .material(np::material::MaterialHandle::new(
                                np::material::BasicMaterial::new(CANNON_BULLET_RESTITUTION, CANNON_BULLET_FRICTION))
                            )
                            .ccd_enabled(true)
                            .margin(CANNON_BULLET_MARGIN)
                            .set_density(CANNON_BULLET_DENSITY)
                            .build(np::object::BodyPartHandle(body_handle, 0));
                        let collider_handle = physics.add_collider(collider);

                        let sprite_render = SpriteRender {
                            sprite_number: CANNON_BULLET_SPRITE_NUM,
                            sprite_sheet: sprite_sheet.handle.as_ref().unwrap().clone()
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
                            .with(TempMarker, &mut temp_markers)
                            .with(DeadlyMarker, &mut deadly_markers)
                            .build();
                        bodies_to_add.push((ent, physics::Body::new(body_handle)));
                        // Start the shooting timer
                        shooting_timer.replace(CANNON_SHOOT_TIME);
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
        for (entity, body) in bodies_to_add.drain(..) {
            bodies.insert(entity, body).expect("Something went wrong when adding bodies to entities");
        }
    }
}