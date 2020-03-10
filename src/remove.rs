use amethyst::{
    ecs::{System, Read, Write, Join, WriteStorage, ReadStorage, ReadExpect, World, WorldExt, Entities}
};

use amethyst_physics::prelude::*;

use crate::config::BulletConfig;
use crate::projectile::Projectile;
use crate::physics::Physics;

pub struct RemoveSystem;

impl <'s> System<'s> for RemoveSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Projectile>,
        ReadExpect<'s, PhysicsWorld<f32>>,
        ReadStorage<'s, Physics>,
        Read<'s, BulletConfig>,
    );

    fn run(&mut self, (entities, mut projectiles, phys_world, physics, b_config): Self::SystemData) {
        let rb_serv = phys_world.rigid_body_server();
        //Delete the projectile if the lifetime exceeds the max lifetime
        for (projectile, p_phys, entity) in (&mut projectiles, &physics, &entities).join() {
            let p_tag = p_phys.rb_handle.as_ref().unwrap().get();
            if projectile.lifetime > b_config.max_lifetime {
                rb_serv.set_shape(p_tag, None);
                rb_serv.set_contacts_to_report(p_tag, 0);
                entities
                    .delete(entity)
                    .expect("Cannot remove projectile");
            } else {
                //TODO: Use Time instead
                projectile.lifetime += 1;
            }
        }
    }
}