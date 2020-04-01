use amethyst::{
    core::Transform,
    renderer::{SpriteRender},
    ecs::{
        System, Entities, 
        WriteStorage, Read, ReadExpect, WriteExpect,
    },
    window::ScreenDimensions,
};
use crate::level::MazeLevel;
use crate::tank::Tank;
use crate::markers::TempMarker;
use crate::utils::SpriteSheetRes;
use crate::physics;
use crate::config::MazeConfig;

pub struct LevelSystem;

impl<'s> System<'s> for LevelSystem {
    type SystemData = (
        Read<'s, MazeConfig>,
        WriteExpect<'s, MazeLevel>,
        Entities<'s>,
        Read<'s, SpriteSheetRes>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Transform>,
        WriteExpect<'s, physics::Physics>,
        WriteStorage<'s, physics::Body>,
        WriteStorage<'s, physics::Collider>,
        WriteStorage<'s, TempMarker>,
        WriteStorage<'s, Tank>,
        ReadExpect<'s, ScreenDimensions>,
    );

    fn run(
        &mut self,
        (
            maze_config,
            mut level,
            entities,
            sprite_sheet,
            mut sprite_renders,
            mut transforms,
            mut physics,
            bodies,
            colliders,
            temp_markers,
            mut tanks,
            screen_dimensions,
        ): Self::SystemData,
    ) {
        if level.should_be_reset {
            level.reset_level(
                &maze_config,
                &entities, 
                &sprite_sheet,
                &mut sprite_renders, 
                &mut transforms,
                &mut physics,
                bodies,
                colliders,
                &screen_dimensions,
                temp_markers,
                &mut tanks
            );
            physics.maintain();
            level.should_be_reset = false;
        }
    }
}