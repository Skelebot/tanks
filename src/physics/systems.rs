use amethyst::{
    core::Transform,
    ecs::{
        System, Join,
        ReadStorage, WriteStorage, WriteExpect, ReadExpect
    }
};

use crate::physics::{Physics, Body};

pub struct StepperSystem;

impl<'s> System<'s> for StepperSystem {
    type SystemData = (
        WriteExpect<'s, Physics>,
    );
    
    fn run (&mut self, mut physics: Self::SystemData) {
        physics.0.step();
    }
}

///Physics To Transform System
pub struct PTTSystem;

impl<'s> System<'s> for PTTSystem {
    type SystemData = (
        ReadExpect<'s, Physics>,
        ReadStorage<'s, Body>,
        WriteStorage<'s, Transform>,
    );
    
    fn run (&mut self, (physics, bodies, mut transforms): Self::SystemData) {
        for (body, transform) in (&bodies, &mut transforms).join() {
            if let Some(rb) = physics.get_rigid_body(body.handle) {
                let pos = rb.position();
                transform.set_translation(
                    amethyst::core::math::Vector3::<f32>::new(
                        pos.translation.vector.x, pos.translation.vector.y, 1.0
                    )
                );
                transform.set_rotation(
                    amethyst::core::math::UnitQuaternion::from_axis_angle(
                        &amethyst::core::math::Vector::z_axis(),
                        pos.rotation.angle(),
                    )
                );
            }
        }
    }
}