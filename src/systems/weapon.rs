use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    renderer::SpriteRender,
    window::ScreenDimensions,
    ui::UiText,
    ecs::{
        Join, System,
        Read, WriteStorage, ReadExpect, WriteExpect,
        Entities, Entity
    }
};
use crate::utils::SpriteSheetRes;
use crate::tank::{Tank, Team};
use crate::physics;
use crate::weapons::Weapon;
use crate::config::TankConfig;
use crate::markers::TempMarker;
use crate::scoreboard::Scoreboard;

const BEAMER_HEAT_TIME: f32 = 0.5;
const BEAMER_HEATING_MAX_SCALE: f32 = 23.0;
const BEAMER_BEAM_WIDTH: f32 = 20.0;
const BEAMER_SHOOT_TIME: f32 = 1.0;
const BEAMER_OVERHEAT_TIME: f32 = 2.0;

pub struct WeaponSystem;

impl<'s> System<'s> for WeaponSystem {
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

        Read<'s, TankConfig>,
        ReadExpect<'s, ScreenDimensions>,

        WriteStorage<'s, UiText>,
        WriteExpect<'s, Scoreboard>,
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
            tank_config,
            screen_dimensions,
            mut ui_text,
            mut scoreboard,
        ): Self::SystemData,
    ) {

        // Entities and Bodies to be added to them because we can't borrow bodies twice in the same scope
        // FIXME: Do something with this, it looks awful
        let mut bodies_to_add: Vec<(Entity, physics::Body)> = Vec::new();
        for (tank, body) in (&mut tanks, &bodies).join() {
            match tank.weapon {
                Weapon::Beamer {
                    ref mut heating_progress,
                    ref mut overheat_timer,
                    ref mut shooting_timer,
                    ref mut heating_square,
                    ref mut beam, 
                } => {
                    // The player is holding the shoot button
                    if tank.is_shooting {

                        // If the weapon can shoot and the weapon is not ready to shoot
                        if *heating_progress < 1.0 && overheat_timer.is_none() && shooting_timer.is_none() {

                            // Lock the tank in place
                            // TODO: Lock the velocity so the tank can slow down instead
                            // FIXME: Should the tank be able to rotate? Do not zero angular velocity
                            // Disabled for testing
                            // body.set_velocity(np::algebra::Velocity2::zero());

                            *heating_progress += time.delta_seconds() / BEAMER_HEAT_TIME;

                            if heating_square.is_none() {
                                // Initialize the heating square
                                let sprite_render = SpriteRender {
                                    sprite_sheet: sprite_sheet.handle.as_ref().expect("SpriteSheet is None").clone(),
                                    // TODO: Use a config
                                    sprite_number: match tank.team {
                                        Team::Red => 4,
                                        Team::Blue => 5,
                                    }
                                };
                                // The transform will be set and updated later so it moves with the player
                                let mut square_transform = Transform::default();
                                // Make the square appear over the tank sprite and over wall sprites
                                square_transform.set_translation_z(0.10);
                                // The heating square is a purely cosmetic entity
                                // TODO: Make the heating square also a sensor so the tanks can run into each other
                                //       while heating their beamers without actually shooting them to kill the other player
                                let square_entity = entities
                                    .build_entity()
                                    .with(square_transform, &mut transforms)
                                    .with(sprite_render, &mut sprite_renders)
                                    .with(TempMarker, &mut temp_markers)
                                    .build();
                                
                                heating_square.replace(square_entity);
                            }

                            // If the weapon is done heating up
                            if *heating_progress >= 1.0 {
                                // Shoot

                                // Create the beam entity

                                let sprite_render = SpriteRender {
                                    sprite_sheet: sprite_sheet.handle.as_ref().expect("SpriteSheet is None").clone(),
                                    // TODO: Use a config
                                    sprite_number: match tank.team {
                                        Team::Red => 4,
                                        Team::Blue => 5,
                                    }
                                };

                                // Calculate the beam length so that it's equal or more than the diagonal of our screen;
                                // we want the players to think the beam is infinite, so the beam's end can be just off-screen
                                let beam_length = screen_dimensions.diagonal().norm();
                                // Because the sprite is just one pixel, calculate the scale needed to make it the correct size
                                let scale = amethyst::core::math::Vector3::new(
                                    BEAMER_BEAM_WIDTH,
                                    beam_length,
                                    1.0
                                );

                                // The position will be set and updated later
                                let mut beam_transform = Transform::default();
                                beam_transform.set_scale(scale);
                                // Make the beam appear over the wall sprites
                                beam_transform.set_translation_z(0.2);

                                // Create a sensor for the beam for detecting physics bodies in the beam
                                let shape = nc::shape::ShapeHandle::new(nc::shape::Cuboid::new(na::Vector2::new(scale.x, scale.y)));
                                let body = np::object::RigidBodyDesc::new().status(np::object::BodyStatus::Kinematic).build();
                                let body_handle = physics.add_rigid_body(body);
                                let sensor = np::object::ColliderDesc::new(shape).sensor(true).build(np::object::BodyPartHandle(body_handle, 0));
                                let sensor_handle = physics.add_collider(sensor);
                                
                                let beam_entity = entities
                                    .build_entity()
                                    .with(beam_transform, &mut transforms)
                                    .with(sprite_render, &mut sprite_renders)
                                    .with(TempMarker, &mut temp_markers)
                                    .with(physics::Collider{handle: sensor_handle}, &mut colliders)
                                    // We would do that but we already borrowed bodies, so we have to build the entity and add the body later
                                    //.with(physics::Body{handle: body_handle}, &mut bodies)
                                    .build();

                                bodies_to_add.push((beam_entity, physics::Body{handle: body_handle}));

                                beam.replace(beam_entity);

                                // Recoil
                                // TODO: Steady force pushing the tank opposite to the shooting direction would be fun

                                // Start shooting timer
                                shooting_timer.replace(BEAMER_SHOOT_TIME);
                            }
                        } 
                        
                    }

                    // Update things related to the weapon

                    let body = physics.get_rigid_body_mut(body.handle).unwrap();
                    if let Some(square) = heating_square {
                        // Update the heating square's transform
                        let rotation = na::UnitQuaternion::from_axis_angle(&na::Vector::z_axis(), body.position().rotation.angle());
                        // TODO: Removing this is impossible until nalgebra versions from Amethyst and NPhysics match
                        let amethyst_rotation = amethyst::core::math::UnitQuaternion::from_axis_angle(&amethyst::core::math::Vector::z_axis(), body.position().rotation.angle());
                        let trans = body.position().translation.vector.push(0.1)
                            + rotation * na::Vector3::<f32>::new(0.0, tank_config.size_y as f32 / 2.0, 0.1);
                        let scale = *heating_progress * BEAMER_HEATING_MAX_SCALE;
                        transforms.get_mut(*square).unwrap()
                            .set_translation_xyz(trans.x, trans.y, trans.z)
                            .set_rotation(amethyst_rotation)
                            .prepend_rotation_z_axis(45.0_f32.to_radians())
                            .set_scale(amethyst::core::math::Vector3::new(scale, scale, 1.0));

                        // The beam only exists if the heating square exists
                        // and we can put the check here so we can use the rotations
                        // we calculated earlier
                        if let Some(beam) = beam {
                            // Update the beam's position
                            // The beam is bound to a physics body (beacause it has a sensor collider)
                            // so we have to update its position by the body
                            // There is a single frame where the player shot but we havent't added the
                            // body handle for the beam yet
                            // TODO: Because of that there is a single frame where the beam visibly appears at (0,0)
                            if let Some(beam_pbody) = bodies.get(*beam) {
                                let rotation = na::UnitComplex::from_angle(body.position().rotation.angle());
                                let trans = body.position().translation.vector
                                    + rotation * na::Vector2::new(0.0, (tank_config.size_y as f32 / 2.0) + (transforms.get(*beam).unwrap().scale().y / 2.0));
                                let angle = body.position().rotation.angle();
                                physics.get_rigid_body_mut(beam_pbody.handle).unwrap()
                                    .set_position(
                                        na::Isometry2::new(
                                            trans.xy(),
                                            angle
                                        )
                                    );
                            }
                        }
                    }
                    
                    if let Some(timer) = shooting_timer {
                        // Lock the tank in place
                        // TODO: Lock the velocity so the tank can slow down instead
                        // FIXME: Should the tank be able to rotate? Do not zero angular velocity
                        // Disabled for testing
                        // body.set_velocity(np::algebra::Velocity2::zero());

                        // Decrease the shooting timer
                        *timer -= time.delta_seconds();

                        // If the timer reached zero
                        if *timer <= 0.0 {
                            // Reset the heating progress
                            *heating_progress = 0.0;
                            // Remove the beam and the heating square
                            // TODO: Do a vanishing animation
                            physics.remove_collider(colliders.get(beam.unwrap()).unwrap().handle);
                            entities.delete(heating_square.unwrap()).expect("Couldn't remove heating square entity");
                            entities.delete(beam.unwrap()).expect("Couldn't remove beam entity");
                            *heating_square = None;
                            *beam = None;
                            *shooting_timer = None;
                            // Start overheat timer
                            overheat_timer.replace(BEAMER_OVERHEAT_TIME);
                        }
                    }
                    if let Some(timer) = overheat_timer {
                        // Decrease the overheat timer
                        *timer -= time.delta_seconds();
                        if *timer <= 0.0 { *overheat_timer = None; }
                    }                
                    if *heating_progress > 0.0 && !tank.is_shooting && shooting_timer.is_none() {
                        *heating_progress -= time.delta_seconds() / BEAMER_HEAT_TIME;
                    }
                },
                _ => (),
            }
        }
        for (entity, body) in bodies_to_add.drain(..) {
            bodies.insert(entity, body).expect("Something went wrong when adding bodies to entities");
        }
        // Check for tanks in the way of Weapon::Beamer's beams
        // If there are any, score for the tank's team
        // and reset the level
        // TODO: Level reset timer and delay between a tank getting destroyed
        //       and the other tank scoring
        physics.maintain();
        for (tank, _collider) in (&tanks, &colliders).join() {
            if let Weapon::Beamer { beam , .. } = tank.weapon {
                if let Some(beam) = beam {
                    if let Some(proximities) = physics.geom_world.proximities_with(&physics.colliders, colliders.get(beam).unwrap().handle, true) {

                        for proximity in proximities {
                            // Proximity is (collider_handle, collider, collider1_handle, collider1, ..)
                            // let coll1_handle = proximity.0;
                            let coll2_handle = proximity.2;
                            // If somehow the tank hit itself with its beam, continue
                            // this would be ok if a tank was hit with its own flying bullet,
                            // but when using a laser beam weapon that doesn't bounce we assume
                            // that it comes from an error while positioning the beam, so we
                            // skip the proximity event.

                            // Compare the victim handle with other tanks' handles to find the exact tank that got hit
                            for (other_tank, other_collider) in (&tanks, &colliders).join() {
                                if coll2_handle == other_collider.handle {
                                    // Score for the shooter tank's team
                                    println!("{:?} tank hit {:?} tank with a laser beam", tank.team, other_tank.team);
                                    scoreboard.score(tank.team, &mut ui_text);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
#[allow(dead_code)]
fn test_mut_enum() {
    enum A {
        Variant {
            sth: f32
        },
        Variant1,
    }

    let mut inst = A::Variant { sth: 0.2 };

    match inst {
        A::Variant { ref mut sth } => *sth += 1.0,
        _ => (),
    }

    if let A::Variant{sth} = inst {
        assert_eq!(sth, 1.2)
    }
}
