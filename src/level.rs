use amethyst::{
    prelude::*,
    ecs::{Entity, Entities, Read, WriteStorage},
    core::math as na,
    core::Transform,
    renderer::SpriteRender,
    window::ScreenDimensions,
};

use amethyst_physics::prelude::*;

use crate::mazegen::Maze;
use crate::physics::Physics;
use crate::state::SpriteSheetRes;

const CELL_WIDTH: f32 = 64.0;
const CELL_HEIGHT: f32 = 64.0;
const W_THICKNESS: f32 = 2.0;

pub struct MazeLevel {
    pub maze: Maze,
    wall_entities: Vec<Entity>,

    wall_rb_desc: RigidBodyDesc<f32>,
    h_wall_shape: PhysicsHandle<PhysicsShapeTag>,
    v_wall_shape: PhysicsHandle<PhysicsShapeTag>,
}

impl MazeLevel {

    pub fn new(world: &mut World) -> Self {
        let mut maze = Maze::new(5, 4);
        maze.build();

        let phys_world = world.fetch::<PhysicsWorld<f32>>();
        
        let h_wall_shape = ShapeDesc::Cube {
            half_extents: na::Vector3::new(
                CELL_WIDTH * 0.5,
                W_THICKNESS * 0.5,
                8.0)};
        let v_wall_shape = ShapeDesc::Cube {
            half_extents: na::Vector3::new(
                W_THICKNESS * 0.5,
                CELL_HEIGHT * 0.5,
                8.0)};

        let h_shape_tag = phys_world.shape_server().create(&h_wall_shape);
        let v_shape_tag = phys_world.shape_server().create(&v_wall_shape);

        let wall_rb_desc = RigidBodyDesc {
            mode: BodyMode::Static,
            mass: 100.0,
            friction: 0.5,
            bounciness: 0.1,
            ..Default::default()
        };

        let mut level = MazeLevel {
            maze: maze,
            wall_entities: Vec::new(),
            wall_rb_desc: wall_rb_desc,
            h_wall_shape: h_shape_tag,
            v_wall_shape: v_shape_tag,
        };

        //Actually create wall entities
        level.rebuild(
            world.entities(),
            phys_world.into(),
            world.system_data(),
            world.system_data(),
            world.system_data(),
            world.system_data(),
            &world.read_resource::<ScreenDimensions>()
        );

        return level;
    }

    pub fn rebuild(
        &mut self, 
        entities: Entities, 
        phys_world: Read<PhysicsWorld<f32>>,
        mut physics: WriteStorage<Physics>,
        ss_handle: Read<SpriteSheetRes>,
        mut sprite_renders: WriteStorage<SpriteRender>,
        mut transforms: WriteStorage<Transform>,
        screen_dimensions: &ScreenDimensions,
     ) {
        //Remove all existing wall entities (if any)
        for entity in self.wall_entities.iter() {
            entities.delete(*entity).expect("Cannot remove a nonexistent wall");
        }
        self.wall_entities.clear();

        //Reset and regenerate the maze
        self.maze.reset();
        self.maze.build();

        let rb_serv = phys_world.rigid_body_server();

        let mut w_pos_rb_h: Vec<(na::Isometry3<f32>, PhysicsHandle<PhysicsRigidBodyTag>, bool)> = Vec::new();

        //Determine the position and create a rigidbody for every horizontal wall
        for (y_index, h_row) in self.maze.walls_h.iter().enumerate() {
            for (x_index, h_wall) in h_row.iter().enumerate() {
                if *h_wall {
                    //Determine the position
                    let pos = na::Isometry3::from_parts(
                        na::Translation::from(na::Vector3::new(
                            (CELL_WIDTH * 0.5) + (x_index as f32 * CELL_WIDTH) + (screen_dimensions.width()/2.0),
                            (y_index as f32 * CELL_HEIGHT) + (screen_dimensions.height()/2.0),
                            0.2,
                        )),
                        na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), 0.0)
                    );
                    //Set the shape
                    let handle = rb_serv.create(&self.wall_rb_desc);
                    rb_serv.set_transform(handle.get(), &pos);
                    rb_serv.set_shape(handle.get(), Some(self.h_wall_shape.get()));

                    w_pos_rb_h.push((pos, handle, true));
                }
            }
        }

        //Determine the position and create a rigidbody for every vertical wall
        for (y_index, v_row) in self.maze.walls_v.iter().enumerate() {
            for (x_index, v_wall) in v_row.iter().enumerate() {
                if *v_wall {
                    //Determine the position
                    let pos = na::Isometry3::from_parts(
                        na::Translation::from(na::Vector3::new(
                            (x_index as f32 * CELL_WIDTH) + (screen_dimensions.width()/2.0),
                            (CELL_HEIGHT * 0.5) + (y_index as f32 * CELL_HEIGHT) + (screen_dimensions.height()/2.0),
                            0.2
                        )),
                        na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), 0.0)
                    );
                    //Set the shape
                    let handle = rb_serv.create(&self.wall_rb_desc);
                    rb_serv.set_transform(handle.get(), &pos);
                    rb_serv.set_shape(handle.get(), Some(self.v_wall_shape.get()));

                    w_pos_rb_h.push((pos, handle, false));
                }
            }
        }
        for (pos, handle, horizontal) in w_pos_rb_h.iter() {
            //Create Physics for the entity
            let w_phys = Physics {
                rb_handle: Some(handle.clone())
            };

            //Create a renderable sprite
            let sprite_render = SpriteRender {
                sprite_sheet: ss_handle.handle.as_ref().expect("SpriteSheet is None").clone(),
                sprite_number: if *horizontal { 4 } else { 3 },   //TODO: Change to use a config
            };

            //Sprite's position
            let loc_trans = Transform::new(pos.translation, pos.rotation, na::Vector3::repeat(1.0));

            //Create the entity
            let ent = entities
                .build_entity()
                .with(sprite_render, &mut sprite_renders)
                .with(w_phys, &mut physics)
                .with(loc_trans, &mut transforms)
                .build();

            self.wall_entities.push(ent);
        }
    }
}