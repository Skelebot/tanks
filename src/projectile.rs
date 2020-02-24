use amethyst::{
    core::Transform,
    ecs::{Component, System, DenseVecStorage, Read, ReadStorage, WriteStorage, Entities},
    renderer::SpriteRender,
    input::{InputHandler, StringBindings},
    //core::timing::Time,
};

use crate::state::SpriteSheetRes;
use crate::tank::*;
use crate::config::BulletConfig;
use crate::physics::Physics;

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
        Read<'s, Option<PhysicsWorld<f32>>>,
        Entities<'s>,
        ReadStorage<'s, Tank>,
        WriteStorage<'s, Physics>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Projectile>,
        WriteStorage<'s, Transform>,
    );

    fn run(
        &mut self,
        _test: Self::SystemData
        //(input, /*time,*/ ss_handle, b_config, phys_world, entities, tanks, mut physics, mut sprite_renders, mut projectiles_data, mut transforms): Self::SystemData
    ) {
        ////Add new projectiles
        //let mut projectiles: Vec<(Projectile, Physics)> = Vec::new();
        //
        //for (tank, physics) in  (&tanks, &physics).join() {
        //    let fire = match tank.team {
        //        Team::Red => input.action_is_down("p1_fire"),
        //        Team::Blue => input.action_is_down("p2_fire"),
        //    };
        //    if let Some(shoot) = fire { if shoot {   //Because we can't use && here, it's experimental
        //        let velocity = physics.pos.rotation * na::Vector2::new(0.0, b_config.speed);
        //        let proj_rb_desc = 
        //            RigidBodyDesc{
        //                mode: BodyMode::Dynamic,
        //                mass: 0.5,
        //                friction: 0.1,
        //                bounciness: 1.0,
        //                ..Default::default()
        //            };

        //        projectiles.push(
        //            (Projectile {
        //                lifetime: 0,
        //                radius: b_config.radius,
        //            },
        //            Physics {
        //                pos: physics.pos,
        //                vel: velocity,
        //                rb_handle: Some(phys_world.as_ref().expect("PhysWorld is None").rigid_body_server()
        //                    .create(&proj_rb_desc)),
        //            }
        //            ));
        //    }}
        //}

        //for (projectile, p_physics) in projectiles {
        //    let sprite_render = SpriteRender {
        //        sprite_sheet: ss_handle.handle.as_ref().unwrap().clone(),
        //        sprite_number: b_config.sprite_num,
        //    };
        //    
        //    //Sprite's position
        //    let mut local_transform = Transform::default();
        //    local_transform.set_translation_xyz(
        //        p_physics.pos.translation.vector.x,
        //        p_physics.pos.translation.vector.y,
        //        0.8);

        //    entities
        //        .build_entity()
        //        .with(sprite_render, &mut sprite_renders)
        //        .with(projectile, &mut projectiles_data)
        //        .with(p_physics, &mut physics)
        //        .with(local_transform, &mut transforms)
        //        .build();
        //}
        //
        ////Do operations on all projectiles
        //for (projectile, p_physics, transform, entity) in (&mut projectiles_data, &mut physics, &mut transforms, &entities).join() {
        //    //Delete the projectile if the lifetime exceeds the max lifetime
        //    if projectile.lifetime > b_config.max_lifetime {
        //        entities.delete(entity).expect("Cannot delete non-existent particle");   //panic if the entity does not exist
        //    }

        //    //TODO: Do the physics
        //    p_physics.pos.translation.vector += p_physics.vel;

        //    //Translate the sprite
        //    transform.set_translation_xyz(
        //        p_physics.pos.translation.vector.x,
        //        p_physics.pos.translation.vector.y,
        //        0.8);

        //    //Increment the projectile's life time counter
        //    projectile.lifetime += 1;
        //}
    }
}
