use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use amethyst::{
    prelude::*,
    ecs::{Entities, Read, WriteStorage, Join, WriteExpect},
    core::Transform,
    renderer::SpriteRender,
    window::ScreenDimensions,
};
use crate::utils::mazegen::Maze;
use crate::utils::SpriteSheetRes;
use crate::markers::TempMarker;
use crate::tank::{Tank, TankState};
use crate::physics;
use crate::config::MazeConfig;
use crate::weapons::Weapon;

pub struct MazeLevel {
    pub maze: Maze,
    pub starting_positions: [na::Point2<f32>; 2],
    pub reset_timer: Option<f32>,
}

impl MazeLevel {

    pub fn new(world: &mut World, dimensions: &ScreenDimensions) -> Self {
        let maze_config = world.fetch::<MazeConfig>();

        let mut maze = Maze::new(maze_config.maze_width, maze_config.maze_height);
        maze.build();
        
        let mut level = MazeLevel {
            maze,
            starting_positions: [na::Point::origin(); 2],
            reset_timer: None,
        };

        //Actually create wall entities
        level.rebuild(
            &maze_config,
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

        level
    }

    pub fn rebuild(
        &mut self, 
        maze_config: &MazeConfig,
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
        //TODO_VL: Scaling, if the maze cannot fit on the screen or is too small
        let x_shift = (screen_dimensions.width() / 2.0) - ((self.maze.width as f32 * maze_config.cell_width) / 2.0);
        let y_shift = (screen_dimensions.height() / 2.0) - ((self.maze.height as f32 * maze_config.cell_height) / 2.0);

        // Every wall entity has a TempMarker Component, so it will be removed every level change
        // Reset and regenerate the maze
        self.maze.reset();
        self.maze.build();

        // Determine the starting positions for players, which are the first cell
        // where the maze generation started, and the last cell it reached, resulting
        // in pretty balanced starting positions
        self.starting_positions = [
            na::Point2::<f32>::new(
                self.maze.start_cell.col as f32 * maze_config.cell_width + (maze_config.cell_width * 0.5) + x_shift, 
                self.maze.start_cell.row as f32 * maze_config.cell_height + (maze_config.cell_height) * 0.5 + y_shift
            ),
            na::Point2::<f32>::new(
                self.maze.end_cell.col as f32 * maze_config.cell_width + (maze_config.cell_width * 0.5) + x_shift, 
                self.maze.end_cell.row as f32 * maze_config.cell_height + (maze_config.cell_height) * 0.5 + y_shift
            ),
        ];

        // Wall position, rigid body, whether the wall is horizontal
        let mut w_pos_rb_h: Vec<(na::Isometry2<f32>, np::object::RigidBody<f32>, bool)> = Vec::new();

        // The RigidBody description to be cloned for every wall
        let mut wall_rb_desc = np::object::RigidBodyDesc::new();
        wall_rb_desc
            .set_linear_damping(maze_config.w_damping)
            .set_angular_damping(maze_config.w_damping);

        // Determine the position and create a rigidbody for every horizontal wall
        for (y_index, h_row) in self.maze.walls_h.iter().enumerate() {
            for (x_index, h_wall) in h_row.iter().enumerate() {
                if *h_wall {
                    //Determine the position
                    let pos = na::Isometry2::from_parts(
                        na::Translation::from(na::Vector2::new(
                            (maze_config.cell_width * 0.5) + (x_index as f32 * maze_config.cell_width) + x_shift,
                            (y_index as f32 * maze_config.cell_height) + y_shift
                        )),
                        na::UnitComplex::new(0.0)
                    );

                    let outer = y_index == 0 ||
                        y_index == self.maze.height || 
                        x_index == self.maze.width;

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
                            (x_index as f32 * maze_config.cell_width) + x_shift,
                            (maze_config.cell_height * 0.5) + (y_index as f32 * maze_config.cell_height) + y_shift
                        )),
                        na::UnitComplex::new(90.0_f32.to_radians())
                    );
                    
                    let outer = x_index == 0 ||
                        y_index == self.maze.height ||
                        x_index == self.maze.width;

                    // Create the RigidBody
                    let rb = if outer { wall_rb_desc.clone().position(pos).set_status(np::object::BodyStatus::Static).build() }
                                 else { wall_rb_desc.clone().position(pos).set_status(np::object::BodyStatus::Dynamic).build() };

                    w_pos_rb_h.push((pos, rb, false));
                }
            }
        }

        for (pos, rb, horizontal) in w_pos_rb_h.drain(..) {
            // Create Physics for the entity
            // Create a renderable sprite
            let sprite_render = SpriteRender {
                sprite_sheet: ss_handle.handle.as_ref().expect("SpriteSheet is None").clone(),
                sprite_number: maze_config.sprite_num
            };

            // Sprite's transform
            let mut wall_transform = Transform::default();
            wall_transform.set_translation_xyz(
                pos.translation.vector.x,
                pos.translation.vector.y,
                0.0
            );
            wall_transform.set_rotation_2d(-pos.rotation.angle());

            //Scale the wall's sprite if it's size doesn't match the cell size
            let width_scale = maze_config.cell_width / maze_config.sprite_width;
            let height_scale = maze_config.cell_height / maze_config.sprite_width;
            wall_transform.set_scale(amethyst::core::math::Vector3::new(
                if horizontal { width_scale } else { height_scale },
                1.0, 1.0
            ));

            let wall_collider = 
                np::object::ColliderDesc::new(nc::shape::ShapeHandle::new(
                    nc::shape::Cuboid::new(na::Vector2::new(
                        maze_config.cell_width * 0.5 - maze_config.rb_margin,
                        maze_config.w_thickness * 0.5,
                    ))
                ))
                .density(maze_config.w_density);

            let wall_body = physics::Body { handle: physics.add_rigid_body(rb) };
            let wall_collider = physics::Collider { 
                handle: physics.add_collider(wall_collider.build(np::object::BodyPartHandle(wall_body.handle, 0))) 
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
        maze_config: &MazeConfig,
        entities: &Entities,
        ss_handle: &Read<SpriteSheetRes>,
        sprite_renders: &mut WriteStorage<SpriteRender>,
        transforms: &mut WriteStorage<Transform>,
        mut physics: &mut WriteExpect<physics::Physics>,
        mut bodies: WriteStorage<physics::Body>,
        mut colliders: WriteStorage<physics::Collider>,
        screen_dimensions: &ScreenDimensions,
        mut temp_markers: WriteStorage<TempMarker>,
        tanks: &mut WriteStorage<Tank>,
    ) {
        // TODO_H: Move this to LevelSystem
        // Remove bodies and colliders belonging to entities with a TempMarker Component
        for (body, collider, _) in (&mut bodies, &mut colliders, &temp_markers).join() {
            physics.remove_collider(collider.handle);
            physics.remove_rigid_body(body.handle);
        }
        // Remove all entities with a TempMarker Component (like projectiles)
        for (entity, _) in (entities, &mut temp_markers).join() {
            entities.delete(entity).expect("Couldn't remove the entity");
        }
        // Reset the weapons
        for (tank, body) in (&mut *tanks, &bodies).join() {
            tank.weapon = Weapon::default();
            tank.state = TankState::Alive;
            // Re-enable physics bodies of all (TODO_O: Destroyed only) tanks
            physics.get_body_mut(body.handle).unwrap().set_status(np::object::BodyStatus::Dynamic);
        }
        // Rebuild the maze
        self.rebuild(maze_config, entities, ss_handle, sprite_renders, transforms, &mut physics, &mut bodies, &mut colliders, &mut temp_markers, screen_dimensions);
        // Move the tanks to new starting positions
        for (index, (_, body)) in (tanks, &mut bodies).join().enumerate() {
            let body = physics.get_rigid_body_mut(body.handle).unwrap();
            body.set_position(na::Isometry2::new(
                na::Vector2::new(self.starting_positions[index].x, self.starting_positions[index].y),
                0.0
            ));
        }
    }
}
