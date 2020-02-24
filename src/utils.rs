use amethyst::{
    core::Transform,
    ecs::{System, ReadStorage, WriteStorage, ReadExpect,
         Join},
};

use crate::physics::{Physics};

use amethyst_physics::prelude::*;

pub struct SpriteTransformSystem;

///Moves the sprite to an entity's position
impl<'s> System<'s> for SpriteTransformSystem {
    type SystemData = (
        ReadExpect<'s, PhysicsWorld<f32>>,
        ReadStorage<'s, Physics>,
        WriteStorage<'s, Transform>,
    );

    fn run(
        &mut self,
        (phys_world, physics, mut transforms): Self::SystemData
    ){
        for (phys, trans) in (&physics, &mut transforms).join() {
            let rb_transform = phys_world
                .rigid_body_server()
                .transform(phys.rb_handle.as_ref().unwrap().get());

            trans.set_rotation(rb_transform.rotation);
            trans.set_translation_xyz(
                rb_transform.translation.vector.x,
                rb_transform.translation.vector.y,
                0.0);
        }
    }
}

