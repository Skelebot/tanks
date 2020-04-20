use rand::{thread_rng, Rng};

use amethyst::{
    ecs::{
        System, Join,
        WriteStorage, ReadStorage, WriteExpect, Read,
    },
    core::Transform,
    renderer::Camera,
    core::math as na,
    core::timing::Time,
};

/// A `Resource` describing camera shake
pub struct CameraShake {
    // A Vec of (duration, magnitude) pairs
    pub dms: Vec<(f32, f32)>,
    cam_origin: Option<na::Point2<f32>>,
}
impl Default for CameraShake {
    fn default() -> Self {
        Self {
            dms: vec![],
            cam_origin: None,
        }
    }
}

pub struct CameraShakeSystem;

impl<'s> System<'s> for CameraShakeSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        WriteExpect<'s, CameraShake>,
        Read<'s, Time>,
    );

    fn run(
        &mut self,
        (mut transforms, cameras, mut cam_shake, time): Self::SystemData,
    ) {
        let mut thread_rng = thread_rng();

        // Get a mutable reference to the transform of the first camera we find
        // There shouldn't be more than one camera anyway
        let mut transforms =  
            (&cameras, &mut transforms).join()
                .map(|x| x.1)
                .collect::<Vec<&mut Transform>>();
        let transform = &mut transforms[0];

        if !cam_shake.dms.is_empty() && cam_shake.cam_origin.is_none() {
            cam_shake.cam_origin.replace(transform.translation().remove_row(2).into());
        }

        let mut accumulated = na::Vector2::repeat(0.0);
        for (_, magnitude) in cam_shake.dms.iter() {
            accumulated += na::Vector2::new(
                thread_rng.gen_range(-1.0, 1.0) * magnitude,
                thread_rng.gen_range(-1.0, 1.0) * magnitude,
            );
        }

        let mut shakes_to_remove: Vec<usize> = Vec::new();
        for (index, (ref mut duration, _)) in cam_shake.dms.iter_mut().enumerate() {
            *duration -= time.delta_seconds();
            if *duration <= 0.0 {
                shakes_to_remove.push(index);
            }
        }

        for i in shakes_to_remove {
            cam_shake.dms.remove(i);
            // If this was the last shake
            if cam_shake.dms.is_empty() {
                // Put the camera back to it's original position
                transform.set_translation_xyz(
                    cam_shake.cam_origin.unwrap().x,
                    cam_shake.cam_origin.unwrap().y,
                    transform.translation().z,
                );
            }
        }

        if !cam_shake.dms.is_empty() {
            // We always move the camera around it's original point
            transform.set_translation_xyz(
                cam_shake.cam_origin.unwrap().x + accumulated.x,
                cam_shake.cam_origin.unwrap().y + accumulated.y,
                transform.translation().z,
            );             
        }
    }
}