use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;

use amethyst::{
    prelude::*,
    ecs::{Entities, WriteStorage, WriteExpect},
    core::Transform,
    window::ScreenDimensions,
    renderer::resources::Tint,
};

use crate::utils::mazegen::Maze;
use crate::markers::{DynamicColorMarker, ColorKey};
use crate::markers::TempMarker;
use crate::physics;
use crate::config::MazeConfig;
use crate::graphics::{ShapeRender, QuadMesh};

pub struct MazeLevel {
    pub maze: Maze,
    pub starting_positions: [na::Isometry2<f32>; 2],
    pub reset_timer: Option<f32>,
}

impl MazeLevel {

    pub fn new(world: &mut World, dimensions: &ScreenDimensions) -> Self {
        let maze_config = world.fetch::<MazeConfig>();

        let mut maze = Maze::new(maze_config.maze_width, maze_config.maze_height);
        maze.build();
        
        let mut level = MazeLevel {
            maze,
            starting_positions: [na::Isometry2::identity(); 2],
            reset_timer: None,
        };

        //Actually create wall entities
        level.rebuild(
            &maze_config,
            &world.entities(),
            &world.fetch::<QuadMesh>(),
            &mut world.system_data(),
            &mut world.system_data(),
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

    // This is terrible, perhaps use tuples and a type, just like in a system?
    // TODO_M: Use a tuple and a type for system data
    #[allow(clippy::too_many_arguments)]
    pub fn rebuild(
        &mut self, 
        maze_config: &MazeConfig,
        entities: &Entities, 
        quad_mesh: &QuadMesh,
        mut shape_renders: &mut WriteStorage<ShapeRender>,
        mut tints: &mut WriteStorage<Tint>,
        mut dyn_color_markers: &mut WriteStorage<DynamicColorMarker>,
        mut transforms: &mut WriteStorage<Transform>,
        physics: &mut WriteExpect<physics::Physics>,
        mut bodies: &mut WriteStorage<physics::Body>,
        mut colliders: &mut WriteStorage<physics::Collider>,
        mut temp_markers: &mut WriteStorage<TempMarker>,
        screen_dimensions: &ScreenDimensions,
     ) {
        use np::object::Body;

        //Determine the shift of everything so that the maze sits in the middle of the screen
        //TODO_VL: Scaling, if the maze cannot fit on the screen or is too small
        let x_shift = (screen_dimensions.width() / 2.0) - ((self.maze.width as f32 * maze_config.cell_width) / 2.0);
        let y_shift = (screen_dimensions.height() / 2.0) - ((self.maze.height as f32 * maze_config.cell_height) / 2.0);

        // Every wall entity has a TempMarker Component, so it will be removed every level change
        // Reset and regenerate the maze
        self.maze.reset();
        self.maze.build();

        // Determine the starting positions for players
        // which are the opposite corners of the maze
        self.starting_positions = [
            na::Isometry2::new(
                na::Vector2::<f32>::new(
                    self.maze.start_cell.col as f32 * maze_config.cell_width + (maze_config.cell_width * 0.5) + x_shift, 
                    self.maze.start_cell.row as f32 * maze_config.cell_height + (maze_config.cell_height) * 0.5 + y_shift
                ),
                0.0_f32.to_radians()
            ),
            na::Isometry2::new(
                na::Vector2::<f32>::new(
                    self.maze.end_cell.col as f32 * maze_config.cell_width + (maze_config.cell_width * 0.5) + x_shift, 
                    self.maze.end_cell.row as f32 * maze_config.cell_height + (maze_config.cell_height) * 0.5 + y_shift
                ),
                180.0_f32.to_radians()
            ),
        ];

        // Wall position, rigid body, whether the wall is horizontal
        let mut w_pos_rb_h: Vec<(na::Isometry2<f32>, np::object::RigidBody<f32>, bool)> = Vec::new();

        //------------------------------
        //HORIZONTAL WALLS
        //------------------------------

        // Determine the position and create a rigidbody for every horizontal wall
        for (y_index, h_row) in self.maze.walls_h.iter().enumerate() {
            // Take only indexes of active walls
            // Enumerate before filtering so that we get original indexes, then drop the bool used for filtering from the tuple
            for x_index in h_row.iter().enumerate().filter(|(_, &is_active)| is_active).map(|(index, _)| index) {
                // Position is the middle of the wall
                let translation = na::Translation::from(na::Vector2::new(
                    (maze_config.cell_width / 2.) + (x_index as f32 * maze_config.cell_width) + x_shift,
                    (y_index as f32 * maze_config.cell_height) + y_shift
                ));

                let pos = na::Isometry2::from_parts(
                    translation,
                    // Walls are horizontal by default
                    na::UnitComplex::new(0.0)
                );

                // Create the RigidBody
                let mut rb = np::object::RigidBodyDesc::new().position(pos).build();

                // Walls are always static
                rb.set_status(np::object::BodyStatus::Static);

                w_pos_rb_h.push((pos, rb, true));
            }
        }

        //------------------------------
        //VERTICAL WALLS
        //------------------------------

        // Determine the position and create a rigidbody for every vertical wall
        for (y_index, v_row) in self.maze.walls_v.iter().enumerate() {
            // Take only indexes of active walls
            // Enumerate before filtering so that we get original indexes, then drop the bool used for filtering from the tuple
            for x_index in v_row.iter().enumerate().filter(|(_, &is_active)| is_active).map(|(index, _)| index) {
                let translation = na::Translation::from(na::Vector2::new(
                    (x_index as f32 * maze_config.cell_width) + x_shift,
                    (maze_config.cell_height * 0.5) + (y_index as f32 * maze_config.cell_height) + y_shift
                ));

                let pos = na::Isometry2::from_parts(
                    translation,
                    // Rotate the wall 90 degrees, so that it's vertical
                    na::UnitComplex::new(90.0_f32.to_radians())
                );

                // Create the RigidBody
                let mut rb = np::object::RigidBodyDesc::new().position(pos).build();

                rb.set_status(np::object::BodyStatus::Static);

                w_pos_rb_h.push((pos, rb, false));
            }
        }

        //------------------------------
        //ENTITY CREATION
        //------------------------------

        for (pos, rb, horizontal) in w_pos_rb_h.into_iter() {

            // Sprite's transform
            let mut wall_transform = Transform::default();
            wall_transform.set_translation_xyz(
                pos.translation.vector.x,
                pos.translation.vector.y,
                0.0
            );
            wall_transform.set_rotation_2d(-pos.rotation.angle());

            //Scale the wall's sprite if it's size doesn't match the cell size
            let half_length = (if horizontal { maze_config.cell_width } else { maze_config.cell_height } + maze_config.w_thickness) / 2.;
            let half_width = maze_config.w_thickness / 2.;

            wall_transform.set_scale(na::Vector3::new(
                half_length * 2., half_width * 2., 1.0
            ));

            let shape_render = ShapeRender {
                mesh: quad_mesh.handle.clone()
            };

            let wall_collider = 
                np::object::ColliderDesc::new(nc::shape::ShapeHandle::new(
                    nc::shape::Cuboid::new(na::Vector2::new(
                        half_length,
                        half_width,
                    ))
                ))
                .material(np::material::MaterialHandle::new(
                    //TODO_M: Config for wall restitution
                    np::material::BasicMaterial::new(1.0, 0.0)
                ));

            let wall_body = physics::Body { handle: physics.add_rigid_body(rb) };
            let wall_collider = physics::Collider { 
                handle: physics.add_collider(wall_collider.build(np::object::BodyPartHandle(wall_body.handle, 0))) 
            };

            // Create the entity
            entities
                .build_entity()
                .with(shape_render, &mut shape_renders)
                .with(Tint(Default::default()), &mut tints)
                .with(DynamicColorMarker(ColorKey::Walls), &mut dyn_color_markers)
                .with(wall_transform, &mut transforms)
                .with(TempMarker(None), &mut temp_markers)
                .with(wall_body, &mut bodies)
                .with(wall_collider, &mut colliders)
                .build();
        }
    }
}