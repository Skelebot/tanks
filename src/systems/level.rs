use amethyst::{
    core::Transform,
    renderer::{SpriteRender},
    ecs::{
        System, Entities, Join,
        ReadStorage, WriteStorage, Read, Write, ReadExpect, WriteExpect,
    },
    window::ScreenDimensions,
};
use specs_physics::{
    BodyComponent, ColliderComponent,
    GeometricalWorldRes, MechanicalWorldRes,
    bodies::BodySet,
    colliders::ColliderSet,
    joints::JointConstraintSet,
};
use crate::level::MazeLevel;
use crate::tank::Tank;
use crate::markers::TempMarker;
use crate::utils::SpriteSheetRes;
pub struct LevelSystem;

impl<'s> System<'s> for LevelSystem {
    type SystemData = (
        Write<'s, MazeLevel>,
        Entities<'s>,
        Read<'s, SpriteSheetRes>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Transform>,
        BodySet<'s, f32>,
        ColliderSet<'s, f32>,
        WriteStorage<'s, TempMarker>,
        ReadStorage<'s, Tank>,
        ReadExpect<'s, ScreenDimensions>,
        WriteExpect<'s, MechanicalWorldRes<f32>>,
        WriteExpect<'s, GeometricalWorldRes<f32>>,
        JointConstraintSet<'s, f32>,
    );

    fn run(
        &mut self,
        (
            mut level,
            entities,
            sprite_sheet,
            mut sprite_renders,
            mut transforms,
            mut bodies,
            mut colliders,
            temp_markers,
            tanks,
            screen_dimensions,
            mut mech_world,
            mut geom_world,
            mut constraints,
        ): Self::SystemData,
    ) {
        if level.should_be_reset {
            // for collider in (&mut colliders.storage).join() {
            //     match collider.user_data().unwrap().downcast_ref::<String>() {
            //         Some(value) => print!("data: {};    ", value),
            //         None => println!("None"),
            //     }
            //     println!("position: {:?}", collider.position().translation.vector);
            // }
            println!("---------");
            level.reset_level(
                &entities, 
                &sprite_sheet,
                &mut sprite_renders, 
                &mut transforms,
                &mut bodies,
                &mut colliders,
                &screen_dimensions,
                temp_markers,
                &tanks
            );
            geom_world.maintain(&mut bodies, &mut colliders);
            mech_world.maintain(&mut geom_world, &mut bodies, &mut colliders, &mut constraints);
            level.should_be_reset = false;

            // for collider in (&mut colliders.storage).join() {
            //     match collider.user_data().unwrap().downcast_ref::<String>() {
            //         Some(value) => print!("data: {};    ", value),
            //         None => println!("None"),
            //     }
            //     println!("position: {:?}", collider.position().translation.vector);
            // }
            // println!("////////////////");
        }
    }
}