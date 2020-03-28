use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use amethyst::{
    prelude::*,
    ecs::{Entities, Read, WriteStorage, ReadStorage, Join, WriteExpect},
    core::Transform,
    renderer::SpriteRender,
    window::ScreenDimensions,
};
use crate::utils::mazegen::Maze;
use crate::utils::SpriteSheetRes;
use crate::markers::TempMarker;
use crate::tank::Tank;
use crate::physics;

// TODO: Use a config
const CELL_WIDTH: f32 = 64.0;
const CELL_HEIGHT: f32 = 64.0;
const W_THICKNESS: f32 = 2.0;
const RB_MARGIN: f32 = 0.8;
const W_DENSITY: f32 = 8.0;
const W_DAMPING: f32 = 3.5;

pub struct MazeLevel {
    pub maze: Maze,
    pub starting_positions: [na::Point2<f32>; 2],
    pub should_be_reset: bool,
}
impl Default for MazeLevel {
    //TODO: Fix this
    fn default() -> Self {
        MazeLevel {
            maze: Maze::new(4, 4),
            starting_positions: [na::Point::origin(); 2],
            should_be_reset: false
        }
    }
}
impl MazeLevel {

    pub fn new(world: &mut World, dimensions: &ScreenDimensions, width: usize, height: usize) -> Self {
        let mut maze = Maze::new(width, height);
        maze.build();
        
        let mut level = MazeLevel {
            maze: maze,
            starting_positions: [na::Point::origin(); 2],
            should_be_reset: false,
        };

        //Actually create wall entities
        level.rebuild(
            &world.entities(),
            &world.system_data(),
            &mut world.system_data(),
            &mut world.system_data(),
            &mut world.system_data(),
            &mut world.system_data(),
            &mut world.system_data(),
            &mut world.system_data(),
            dimensions
        );

        return level;
    }

    pub fn rebuild(
        &mut self, 
        entities: &Entities, 
        ss_handle: &Read<SpriteSheetRes>,
        mut sprite_renders: &mut WriteStorage<SpriteRender>,
        mut transforms: &mut WriteStorage<Transform>,
        physics: &mut WriteExpect<physics::Physics>,
        mut bodies: &mut WriteStorage<physics::Body>,
        mut colliders: &mut WriteStorage<physics::Collider>,
        mut temp_markers: &mut WriteStorage<TempMarker>,
        screen_dimensions: &ScreenDimensions,
     ) {

        //Determine the shift of everything so that the maze sits in the middle of the screen
        //TODO: Scaling, if the maze cannot fit on the screen
        let x_shift = (screen_dimensions.width() / 2.0) - ((self.maze.width as f32 * CELL_WIDTH) / 2.0);
        let y_shift = (screen_dimensions.height() / 2.0) - ((self.maze.height as f32 * CELL_HEIGHT) / 2.0);

        // Every wall entity has a TempMarker Component, so it will be removed every level change
        // Reset and regenerate the maze
        self.maze.reset();
        self.maze.build();

        // Determine the starting positions for players, which are the first cell
        // where the maze generation started, and the last cell it reached, resulting
        // in pretty balanced starting positions
        self.starting_positions = [
            na::Point2::<f32>::new(
                self.maze.start_cell.col as f32 * CELL_WIDTH + (CELL_WIDTH * 0.5) + x_shift, 
                self.maze.start_cell.row as f32 * CELL_HEIGHT + (CELL_HEIGHT) * 0.5 + y_shift
            ),
            na::Point2::<f32>::new(
                self.maze.end_cell.col as f32 * CELL_WIDTH + (CELL_WIDTH * 0.5) + x_shift, 
                self.maze.end_cell.row as f32 * CELL_HEIGHT + (CELL_HEIGHT) * 0.5 + y_shift
            ),
        ];

        // position, rigid body, whether the wall is horizontal, whether the wall is an outer wall
        let mut w_pos_rb_h: Vec<(na::Isometry2<f32>, np::object::RigidBody<f32>, bool)> = Vec::new();

        // The RigidBody description to be cloned for every wall
        let mut wall_rb_desc = np::object::RigidBodyDesc::new();
        wall_rb_desc
            .set_linear_damping(W_DAMPING)
            .set_angular_damping(W_DAMPING);

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

                    let outer = if y_index == 0 ||
                        y_index == self.maze.height || x_index == self.maze.width 
                            { true }
                        else { false };

                    // Create the RigidBody
                    let rb = if outer { wall_rb_desc.clone().position(pos).set_status(np::object::BodyStatus::Static).build() }
                                 else { wall_rb_desc.clone().position(pos).set_status(np::object::BodyStatus::Dynamic).build() };

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
                    
                    let outer = if x_index == 0 ||
                        y_index == self.maze.height || x_index == self.maze.width 
                            { true }
                        else { false };

                    // Create the RigidBody
                    let rb = if outer { wall_rb_desc.clone().position(pos).set_status(np::object::BodyStatus::Static).build() }
                                 else { wall_rb_desc.clone().position(pos).set_status(np::object::BodyStatus::Dynamic).build() };

                    w_pos_rb_h.push((pos, rb, false));
                }
            }
        }

        for (index, (pos, rb, horizontal)) in w_pos_rb_h.drain(..).enumerate() {
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

            let wall_collider = 
                if horizontal {
                    np::object::ColliderDesc::new(nc::shape::ShapeHandle::new(
                        nc::shape::Cuboid::new(na::Vector2::new(
                            CELL_WIDTH * 0.5 - RB_MARGIN,
                            W_THICKNESS * 0.5,
                        ))
                    ))
                    .user_data(format!("wall_h_{}", index))
                    .density(W_DENSITY)
                } else {
                    np::object::ColliderDesc::new(nc::shape::ShapeHandle::new(
                        nc::shape::Cuboid::new(na::Vector2::new(
                            W_THICKNESS * 0.5,
                            CELL_HEIGHT * 0.5 - RB_MARGIN
                        ))
                    ))
                    .user_data(format!("wall_v_{}", index))
                    .density(W_DENSITY)
                };

            let wall_body = physics::Body { handle: physics.add_rigid_body(rb) };
            let wall_collider = physics::Collider { 
                handle: physics.add_collider(wall_collider.build(np::object::BodyPartHandle(wall_body.handle.clone(), 0))) 
            };

            // Create the entity
            entities
                .build_entity()
                .with(sprite_render, &mut sprite_renders)
                .with(wall_transform, &mut transforms)
                .with(TempMarker, &mut temp_markers)
                .with(wall_body, &mut bodies)
                .with(wall_collider, &mut colliders)
                .build();
        }
    }

    pub fn reset_level(
        &mut self,
        entities: &Entities,
        ss_handle: &Read<SpriteSheetRes>,
        sprite_renders: &mut WriteStorage<SpriteRender>,
        transforms: &mut WriteStorage<Transform>,
        mut physics: &mut WriteExpect<physics::Physics>,
        mut bodies: WriteStorage<physics::Body>,
        mut colliders: WriteStorage<physics::Collider>,
        screen_dimensions: &ScreenDimensions,
        mut temp_markers: WriteStorage<TempMarker>,
        tanks: &ReadStorage<Tank>,
    ) {
        // Remove bodies and colliders belonging to entities with a TempMarker Component
        for (body, collider, _) in (&mut bodies, &mut colliders, &temp_markers).join() {
            physics.remove_collider(collider.handle);
            physics.remove_rigid_body(body.handle);
        }
        // Remove all entities with a TempMarker Component (like projectiles)
        for (entity, _) in (entities, &mut temp_markers).join() {
            entities.delete(entity).expect("Couldn't remove the entity");
        }
        // Rebuild the maze
        self.rebuild(entities, ss_handle, sprite_renders, transforms, &mut physics, &mut bodies, &mut colliders, &mut temp_markers, screen_dimensions);
        // Move the tanks to new starting positions
        for (index, (_, body)) in (tanks, &mut bodies).join().enumerate() {
            let body = physics.get_rigid_body_mut(body.handle).unwrap();
            body.set_position(na::Isometry2::new(
                // TODO: Why can't we easily convert between Point2 and Vector2 here?
                na::Vector2::new(self.starting_positions[index].x, self.starting_positions[index].y),
                0.0
            ));
        }
    }
}