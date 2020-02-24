use amethyst::{
    core::Transform,
    ecs::{System, ReadStorage, WriteStorage, Read, Write, Join},
    core::math as na,
};

use crate::physics::{Physics};
use crate::tank::Tank;
use crate::projectile::Projectile;

pub struct PhysicsSystem;

//TODO: delta?
impl<'s> System<'s> for PhysicsSystem {
    type SystemData = (
        WriteStorage<'s, Physics>,
        WriteStorage<'s, Transform>,
    );
    
    fn run(
        &mut self,
        (mut physics, mut transforms): Self::SystemData
    ){
        for (phys, trans) in (&mut physics, &mut transforms).join() {
            //Update sprite transform
            trans.set_rotation(
                na::UnitQuaternion::from_axis_angle(
                    &na::Vector3::z_axis(),
                    phys.pos.rotation.angle()));
            trans.set_translation_xyz(
                phys.pos.translation.vector.x,
                phys.pos.translation.vector.y,
                0.0);
        }
    }
}
