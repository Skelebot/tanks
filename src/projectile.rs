use amethyst::{
    prelude::* ,
    core::math as na,
    core::Transform,
    core::timing::Time,
    ecs::{
        Component, DenseVecStorage, Entities, Join, Read, Write, ReadExpect, ReadStorage, System,
        WriteStorage,
    },
    input::{InputHandler, StringBindings},
    renderer::SpriteRender,
    renderer::debug_drawing::DebugLines,
    renderer::palette::Srgba,
};

use crate::config::BulletConfig;
use crate::physics::Physics;
use crate::state::SpriteSheetRes;
use crate::tank::*;

use amethyst_physics::prelude::*;

pub struct Projectile {
    pub lifetime: u32,
    pub radius: f32,
}

impl Component for Projectile {
    type Storage = DenseVecStorage<Self>;
}

pub struct ProjectileSystem;

impl<'s> System<'s> for ProjectileSystem {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        //Read<'s, Time>,
        Read<'s, SpriteSheetRes>,
        Read<'s, BulletConfig>,
        ReadExpect<'s, PhysicsWorld<f32>>,
        Entities<'s>,
        WriteStorage<'s, Tank>,
        WriteStorage<'s, Physics>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Projectile>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Write<'s, DebugLines>,
    );

    fn run(
        &mut self,
        (
            input,
            ss_handle,
            b_config,
            phys_world,
            entities,
            mut tanks,
            mut physics,
            mut sprite_renders,
            mut projectiles_data,
            mut transforms,
            time,
            mut debug_lines,
        ): Self::SystemData,
    ) {
        //Add new projectiles
        let mut projectiles: Vec<(Projectile, Physics)> = Vec::new();
        let rb_serv = phys_world.rigid_body_server();

        for (tank, physics) in (&mut tanks, &physics).join() {
            let fire = match tank.team {
                Team::Red => input.action_is_down("p1_fire"),
                Team::Blue => input.action_is_down("p2_fire"),
            };

            let rb_handle = physics.rb_handle.as_ref().unwrap().get();

            if let Some(shoot) = fire {
                if shoot && tank.weapon_timeout.is_none() {
                    //Set weapon timeout
                    tank.weapon_timeout.replace(0.8);

                    //Shooting recoil
                    let recoil_vec = rb_serv.transform(rb_handle).rotation * na::Vector3::new(0.0, -1.0 * 300_000.0, 0.0);
                    phys_world
                        .rigid_body_server()
                        .apply_impulse(rb_handle, &recoil_vec);

                    let velocity = phys_world.rigid_body_server().transform(rb_handle).rotation
                        * na::Vector3::new(0.0, b_config.speed, 0.0);

                    let proj_rb_desc = RigidBodyDesc {
                        mode: BodyMode::Dynamic,
                        mass: 1.5,
                        friction: 0.0,
                        bounciness: 1.0,
                        lock_rotation_x: true,
                        lock_rotation_y: true,
                        lock_translation_z: true,
                        ..Default::default()
                    };

                    //Set up the projectile's physical body
                    let proj_rb_handle = Some(rb_serv.create(&proj_rb_desc));

                    //FIXME: spawn the projectile outside of tank's collider
                    //Set the projectile's spawn position
                    rb_serv.set_transform(
                        proj_rb_handle.as_ref().unwrap().get(),
                        &rb_serv.transform(rb_handle),
                    );

                    //Set the projectile's initial velocity
                    rb_serv.set_linear_velocity(proj_rb_handle.as_ref().unwrap().get(), &velocity);

                    //Set the projectile's body shape
                    let shape_desc = ShapeDesc::Sphere { radius: b_config.radius };
                    let shape_tag = phys_world.shape_server().create(&shape_desc);

                    rb_serv.set_shape(
                        proj_rb_handle.as_ref().unwrap().get(),
                        Some(shape_tag.get()),
                    );

                    projectiles.push((
                        Projectile {
                            lifetime: 0,
                            radius: b_config.radius,
                        },
                        Physics {
                            rb_handle: proj_rb_handle,
                        },
                    ));
                }
            }

            //Decrease the weapon timeout
            if let Some(mut timeout) = tank.weapon_timeout.take() {
                timeout -= time.delta_seconds();
                if timeout <= 0.0 {
                    tank.weapon_timeout = None;
                } else {
                    tank.weapon_timeout.replace(timeout);
                }
            }
        }

        for (projectile, p_physics) in projectiles {
            let sprite_render = SpriteRender {
                sprite_sheet: ss_handle.handle.as_ref().unwrap().clone(),
                sprite_number: b_config.sprite_num,
            };

            let rb_handle = p_physics.rb_handle.as_ref().unwrap().get();

            rb_serv.set_contacts_to_report(rb_handle, 8);

            //Sprite's position
            let mut local_transform = Transform::default();
            local_transform.set_translation_xyz(
                rb_serv.transform(rb_handle).translation.vector.x,
                rb_serv.transform(rb_handle).translation.vector.y,
                0.8,
            );

            entities
                .build_entity()
                .with(sprite_render, &mut sprite_renders)
                .with(projectile, &mut projectiles_data)
                .with(p_physics, &mut physics)
                .with(local_transform, &mut transforms)
                .build();
        }

        //Do operations on all projectiles
        for (projectile, entity, p_physics) in (&mut projectiles_data, &entities, &physics).join() {

            let p_rb_tag = p_physics.rb_handle.as_ref().unwrap().get();
            debug_lines.draw_circle(
                na::Point3::from(rb_serv.transform(p_rb_tag).translation.vector),
                b_config.radius,
                8,
                Srgba::new(0.0, 1.0, 0.0, 1.0),
            );

            //Check for collisions
            let mut contacts: Vec<ContactEvent<f32>> = Vec::new();
            rb_serv.contact_events(p_rb_tag, &mut contacts);

            //for contact in contacts.iter() {
            //    for (tank, t_phys) in (&tanks, &physics).join() {
            //        let tank_rb_tag = t_phys.rb_handle.as_ref().unwrap().get();
            //        if contact.other_body == tank_rb_tag {
            //            match &tank.team {
            //                Team::Blue => println!("Blue tank hit"),
            //                Team::Red => println!("Red tank hit")
            //            }
            //        }
            //    }
            //}
        }
    }
}
