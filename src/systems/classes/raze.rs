use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    renderer::resources::Tint,
    renderer::SpriteRender,
    input::InputHandler,
    ecs::{
        Join, System,
        Read, WriteStorage, WriteExpect, ReadExpect,
        Entities, Entity,
        Component, DenseVecStorage,
    }
};
use crate::utils::TanksSpriteSheet;
use crate::graphics::{CircleMesh, ShapeRender, TintBox, SecondaryColor, map_range};
use crate::tank::{Tank, TankState, Class, Team};
use crate::physics;
use crate::input::TankBindingTypes;
use crate::weapons::Weapon;
use crate::config::TankConfig;
use crate::config::RazeConfig;
use crate::config::PerformanceConfig;
use crate::markers::*;

pub struct RazeSystem;
impl<'s> System<'s> for RazeSystem {
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
        WriteStorage<'s, TintBox>,
        WriteStorage<'s, SecondaryColor>,
        WriteStorage<'s, DynamicColorMarker>,
        WriteStorage<'s, DynamicSecondaryColorMarker>,
        WriteStorage<'s, SpriteRender>,
        ReadExpect<'s, TanksSpriteSheet>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, DeadlyMarker>,
        WriteStorage<'s, RocketMarker>,

        ReadExpect<'s,  TankConfig>,
        ReadExpect<'s,  RazeConfig>,
        ReadExpect<'s,  PerformanceConfig>,
    );
    fn run(&mut self, (
        mut tanks,
        mut physics,
        mut bodies,
        mut colliders,

        time,
        entities,

        mut transforms,
        mut tints,
        mut tint_boxes,
        mut secondary_colors,
        mut dyn_color_markers,
        mut dyn_sec_color_markers,
        mut sprite_renders,
        sprite_sheet,
        mut temp_markers,
        mut deadly_markers,
        mut rocket_markers,

        tank_config,
        raze_config,
        performance_config,
    ): Self::SystemData) {
        let mut bodies_to_add: Vec<(Entity, physics::Body)> = Vec::new();
        for (tank, body) in (&mut tanks, &bodies).join() {
            if tank.class != Class::Raze { return; }
            if tank.is_using_ability && tank.state == TankState::Alive {
                // If the ability is ready to use
                if tank.ability_refresh.is_none() {
                    let body = physics.get_rigid_body(body.handle).unwrap();
                    if performance_config.test_wallscan {
                        // Trace an infinite ray from the tank's origin to in the direction it's facing
                        let ray = nc::query::Ray {
                            origin: body.position().translation.vector.into(),
                            dir: body.position().rotation * na::Vector2::new(0.0, 1.0)
                        };
                        let toi = (tank_config.size_y as f32/2.0) + raze_config.rocket_self_safety_margin + performance_config.wallscan_toi_mod;
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
                        body.position().translation.vector + body.position().rotation * na::Vector2::new(0.0, (tank_config.size_y as f32 / 2.0) + raze_config.rocket_self_safety_margin),
                        body.position().rotation.angle(),
                    );

                    let vel_vec = body.position().rotation * na::Vector2::new(0.0, raze_config.rocket_velocity);
                    let velocity = np::algebra::Velocity2::new(
                        na::Vector2::new(vel_vec.x, vel_vec.y), 0.0
                    );

                    let shape = nc::shape::ShapeHandle::new(
                        nc::shape::Cuboid::new(
                            na::Vector2::new(raze_config.rocket_width, raze_config.rocket_height)
                        ));

                    let body = np::object::RigidBodyDesc::new()
                        .position(pos)
                        .velocity(velocity)
                        .build();

                    let body_handle = physics.add_rigid_body(body);
                    let collider = np::object::ColliderDesc::new(shape)
                        .ccd_enabled(true)
                        .margin(raze_config.rocket_margin)
                        .density(10.0)
                        .build(np::object::BodyPartHandle(body_handle, 0));
                    let collider_handle = physics.add_collider(collider);

                    let sprite_render = SpriteRender {
                        sprite_number: raze_config.rocket_sprite_num,
                        sprite_sheet: sprite_sheet.handle.clone(),
                    };
                    let mut transform = Transform::default();
                    transform.set_translation_x(-200.0);
                    let x = -0.5;
                    let y = map_range(6., 0., raze_config.rocket_height as f32, -0.5, 0.5);
                    let width = 1.0;
                    //let height = map_range(3., 0., raze_config.rocket_height as f32, 0.0, 1.0);
                    let height = 1.0;
                    let ent = entities
                        .build_entity()
                        .with(transform, &mut transforms)
                        .with(sprite_render, &mut sprite_renders)
                        .with(physics::Collider::new(collider_handle), &mut colliders)
                        // We would do that but we already borrowed bodies, so we have to build the entity now and add the body later
                        //.with(physics::Body{handle: body_handle}, &mut bodies)
                        .with(TempMarker(Some(raze_config.rocket_lifetime)), &mut temp_markers)
                        .with(Tint::default(), &mut tints)
                        .with(DynamicColorMarker(tank.team.into()), &mut dyn_color_markers)
                        .with(SecondaryColor(Tint::default()), &mut secondary_colors)
                        .with(DynamicSecondaryColorMarker(ColorKey::Text), &mut dyn_sec_color_markers)
                        .with(TintBox([x, y, width, height]), &mut tint_boxes)
                        .with(DeadlyMarker, &mut deadly_markers)
                        .with(RocketMarker(tank.team), &mut rocket_markers)
                        .build();
                    bodies_to_add.push((ent, physics::Body::new(body_handle)));
                    // Start the shooting timer
                    tank.ability_refresh.replace(raze_config.rocket_shoot_time);

                }
            }
            if let Some(ref mut refresh) = tank.ability_refresh {
                *refresh -= time.delta_seconds();
                if *refresh <= 0.0 {
                    tank.ability_refresh = None;
                }
            }
        }
        for (entity, body) in bodies_to_add.into_iter() {
            bodies.insert(entity, body).expect("Something went wrong when adding bodies to entities");
        }

        // Update rockets
        for (rocket_marker, rocket_body) in (&rocket_markers, &bodies).join() {
            // Unwrap() here usually panics in the exact frame the rocket gets removed
            let mut dir: Option<na::Vector2::<f32>> = None;
            if let Some(rocket_rb) = physics.get_rigid_body(rocket_body.handle) {
                for (tank, tank_body) in (&tanks, &bodies).join() {
                    // Do not attack the tank that shot the rocket
                    if rocket_marker.0 == tank.team { continue; }
                    // Only target alive and stunned tanks
                    if !(tank.state == TankState::Alive || tank.state == TankState::Stunned) { continue; }
                    let tank_rb = physics.get_rigid_body(tank_body.handle).unwrap();
                    // Calculate the distance between the rocket and the tank
                    let rocket_pos = rocket_rb.position().translation.vector;
                    let tank_pos = tank_rb.position().translation.vector;
                    let distance = na::distance(&rocket_pos.into(), &tank_pos.into());

                    if distance < raze_config.rocket_radius {
                        // Calculate the direction vector from the rocket towards the tank
                        dir = Some((tank_pos - rocket_pos).normalize());
                    }
                }
            }
            if let Some(dir) = dir {
                // If dir is Some then we are sure that the rocket is valid
                let rb = physics.get_rigid_body(rocket_body.handle).unwrap();
                // Check if there is something in the way of the rocket
                let ray = nc::query::Ray {
                    origin: rb.position().translation.vector.into(),
                    dir: dir,
                };
                let toi = raze_config.rocket_radius;
                let cg = nc::pipeline::object::CollisionGroups::new();
                let interferences = physics.geom_world.interferences_with_ray(
                    &physics.colliders, 
                    &ray, 
                    toi,
                    &cg,
                ).count();  // We only care about the number of the interactions, so we count items in the iterator

                // If dir is Some then we are sure that the rocket is valid
                let mut rb = physics.get_rigid_body_mut(rocket_body.handle).unwrap();
                if interferences < 3 {
                    rb.set_velocity(
                        *rb.velocity() +
                        np::math::Velocity::new(
                            dir * time.delta_seconds() * 100.0 * raze_config.rocket_accel,
                            0.0
                        )
                    );
                }
            }
        }
    }
}

pub struct RocketMarker(pub Team);
impl Component for RocketMarker {
    type Storage = DenseVecStorage<Self>;
}