use amethyst::{
    prelude::*,
    ecs::{Entity, Entities, Read, WriteStorage},
    core::math as na,
    core::Transform,
    renderer::SpriteRender,
    window::ScreenDimensions,
};
use specs_physics::{
    ncollide::shape::{Cuboid, ShapeHandle},
    nphysics::object::{ColliderDesc, RigidBody, RigidBodyDesc, BodyPartHandle},
    BodyComponent, ColliderComponent
};
use crate::utils::mazegen::Maze;
use crate::utils::SpriteSheetRes;

const CELL_WIDTH: f32 = 64.0;
const CELL_HEIGHT: f32 = 64.0;
const W_THICKNESS: f32 = 2.0;

pub struct MazeLevel {
    pub maze: Maze,
    wall_entities: Vec<Entity>,
    pub starting_positions: [na::Point2<f32>; 2],
}

impl MazeLevel {

    pub fn new(world: &mut World, dimensions: &ScreenDimensions, width: usize, height: usize) -> Self {
        let mut maze = Maze::new(width, height);
        maze.build();
        
        let mut level = MazeLevel {
            maze: maze,
            wall_entities: Vec::new(),
            starting_positions: [na::Point::origin(); 2]
        };

        //Actually create wall entities
        level.rebuild(
            world.entities(),
            world.system_data(),
            world.system_data(),
            world.system_data(),
            world.system_data(),
            world.system_data(),
            dimensions
        );

        return level;
    }

    pub fn rebuild(
        &mut self, 
        entities: Entities, 
        ss_handle: Read<SpriteSheetRes>,
        mut sprite_renders: WriteStorage<SpriteRender>,
        mut transforms: WriteStorage<Transform>,
        mut bodies: WriteStorage<BodyComponent<f32>>,
        mut colliders: WriteStorage<ColliderComponent<f32>>,
        screen_dimensions: &ScreenDimensions,
     ) {

        //Determine the shift of everything so that the maze sits in the middle of the screen
        //TODO: Scaling, if the maze cannot fit on the screen
        let x_shift = (screen_dimensions.width() / 2.0) - ((self.maze.width as f32 * CELL_WIDTH) / 2.0);
        let y_shift = (screen_dimensions.height() / 2.0) - ((self.maze.height as f32 * CELL_HEIGHT) / 2.0);

        //Remove all existing wall entities (if any)
        for entity in self.wall_entities.iter() {
            entities.delete(*entity).expect("Cannot remove a nonexistent wall");
        }
        self.wall_entities.clear();

        //Reset and regenerate the maze
        self.maze.reset();
        self.maze.build();

        // Determine the starting positions for players, which are the first cell
        // where the maze generation started, and the last cell it reached, resulting
        // in pretty balanced starting positions
        self.starting_positions = [
            na::Point2::<f32>::new(
                self.maze.start_cell.col as f32 * CELL_WIDTH + x_shift, 
                self.maze.start_cell.row as f32 * CELL_HEIGHT + y_shift
            ),
            na::Point2::<f32>::new(
                self.maze.end_cell.col as f32 * CELL_WIDTH + x_shift, 
                self.maze.end_cell.row as f32 * CELL_HEIGHT + y_shift
            ),
        ];

        // position, rigid body, whether the wall is horizontal
        let mut w_pos_rb_h: Vec<(na::Isometry2<f32>, RigidBody<f32>, bool)> = Vec::new();

        // Determine the position and create a rigidbody for every horizontal wall
        for (y_index, h_row) in self.maze.walls_h.iter().enumerate() {
            for (x_index, h_wall) in h_row.iter().enumerate() {
                if *h_wall {
                    //Determine the position
                    let pos = na::Isometry2::from_parts(
                        na::Translation::from(na::Vector2::new(
                            (CELL_WIDTH * 0.5) + (x_index as f32 * CELL_WIDTH) + x_shift,
                            (y_index as f32 * CELL_HEIGHT) + y_shift
                        )),
                        na::UnitComplex::new(0.0)
                    );
                    //Create the rigid body
                    let rb = RigidBodyDesc::new()
                        .position(pos)
                        .build();

                    w_pos_rb_h.push((pos, rb, true));
                }
            }
        }

        // Determine the position and create a rigidbody for every vertical wall
        for (y_index, v_row) in self.maze.walls_v.iter().enumerate() {
            for (x_index, v_wall) in v_row.iter().enumerate() {
                if *v_wall {
                    //Determine the position
                    let pos = na::Isometry2::from_parts(
                        na::Translation::from(na::Vector2::new(
                            (x_index as f32 * CELL_WIDTH) + x_shift,
                            (CELL_HEIGHT * 0.5) + (y_index as f32 * CELL_HEIGHT) + y_shift
                        )),
                        na::UnitComplex::new(0.0)
                    );
                    //Create the rigid body
                    let rb = RigidBodyDesc::new()
                        .position(pos)
                        .build();

                    w_pos_rb_h.push((pos, rb, false));
                }
            }
        }

        for (pos, rb, horizontal) in w_pos_rb_h.drain(..) {
            // Create Physics for the entity
            // Create a renderable sprite
            let sprite_render = SpriteRender {
                sprite_sheet: ss_handle.handle.as_ref().expect("SpriteSheet is None").clone(),
                sprite_number: if horizontal { 4 } else { 3 },   //TODO: Change to use a config
            };

            // Sprite's position
            let mut wall_transform = Transform::default();
            wall_transform.set_translation_xyz(
                pos.translation.vector.x,
                pos.translation.vector.y,
                0.5
            );

            let wall_collider = if horizontal {
                ColliderDesc::new(ShapeHandle::new(
                    Cuboid::new(na::Vector2::new(
                        CELL_WIDTH * 0.5,
                        W_THICKNESS * 0.5,
                    ))
                )).density(2000.0)
            } else {
                ColliderDesc::new(ShapeHandle::new(
                    Cuboid::new(na::Vector2::new(
                        W_THICKNESS * 0.5,
                        CELL_HEIGHT * 0.5
                    ))
                )).density(2000.0)
            };

            // Create the entity
            let entity_builder = entities
                .build_entity()
                .with(sprite_render, &mut sprite_renders)
                .with(wall_transform, &mut transforms)
                .with(BodyComponent::new(rb), &mut bodies);

            let collider_component = ColliderComponent(
                wall_collider.build(BodyPartHandle(entity_builder.entity, 0))
            );

            let entity = entity_builder
                .with(collider_component, &mut colliders)
                .build();

            self.wall_entities.push(entity);
        }
    }
}