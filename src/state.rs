use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::math as na,
    core::transform::Transform,
    input::{is_close_requested, is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};

use crate::config::TankConfig;
use crate::physics::Physics;
use crate::tank::{Tank, Team};
use crate::level::MazeLevel;

use amethyst_physics::prelude::*;

#[derive(Default)]
pub struct SpriteSheetRes {
    pub handle: Option<Handle<SpriteSheet>>,
}

pub struct MyState;
impl SimpleState for MyState {
    // On start will run when this state is initialized.
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Get the screen dimensions so we can initialize the camera and
        // place our sprites correctly later. We'll clone this since we'll
        // pass the world mutably to the following functions.
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

        // Place the camera
        init_camera(world, &dimensions);

        // Load our sprites and display them
        let sprite_sheet_handle = load_sprite_sheet(world);
        let ss_handle_res = SpriteSheetRes {
            handle: Some(sprite_sheet_handle.clone()),
        };
        world.insert(ss_handle_res);

        //Initialize players
        init_tanks(world, sprite_sheet_handle, &dimensions);

        //Initialize the maze
        init_maze(world);
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }

            // Listen to any key events
            //if let Some(event) = get_key(&event) {
            //    info!("handling key event: {:?}", event);
            //}
        }
        // Keep going
        Trans::None
    }
}

fn init_camera(world: &mut World, dimensions: &ScreenDimensions) {
    // Center the camera in the middle of the screen, and let it cover
    // the entire screen
    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);

    world
        .create_entity()
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build();
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    // Load the texture for our sprites. We'll later need to
    // add a handle to this texture to our `SpriteRender`s, so
    // we need to keep a reference to it.
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "sprites/tanks.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    // Load the spritesheet definition file, which contains metadata on our
    // spritesheet texture.
    let loader = world.read_resource::<Loader>();
    let sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
    return loader.load(
        "sprites/tanks.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &sheet_storage,
    );
}

fn init_maze(world: &mut World) {
    let maze_res = MazeLevel::new(world);

    world.insert(maze_res);
}

fn init_tanks(world: &mut World, sheet_handle: Handle<SpriteSheet>, dimensions: &ScreenDimensions) {
    let sprites: Vec<SpriteRender> = (0..2)
        .map(|i| SpriteRender {
            sprite_sheet: sheet_handle.clone(),
            sprite_number: i,
        })
        .collect();

    let x = dimensions.width() * 0.5;
    let y = dimensions.height() * 0.5;
    let mut red_trans = Transform::default();
    red_trans.set_translation_xyz(x, y, 0.);
    let x = dimensions.width() * 0.5;
    let y = dimensions.height() * 0.5;
    let mut blue_trans = Transform::default();
    blue_trans.set_translation_xyz(x + 50.0, y, 0.);
    let red_pos = na::Isometry3::from_parts(
        na::Translation::from(na::Vector3::new(x, y, 0.)),
        na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), 0.0),
    );
    let blue_pos = na::Isometry3::from_parts(
        na::Translation::from(na::Vector3::new(x + 50.0, y, 0.)),
        na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), 0.0),
    );

    //TODO: change to Default::default();
    //and impl Default for Physics
    let red_physics = Physics { rb_handle: None };

    let blue_physics = Physics { rb_handle: None };

    //Create the Red Tank
    let red = world
        .create_entity()
        .with(Tank::new(Team::Red))
        .with(sprites[0].clone())
        .with(red_physics)
        .with(red_trans)
        .build();

    //Create the Blue Tank
    let blue = world
        .create_entity()
        .with(Tank::new(Team::Blue))
        .with(sprites[1].clone())
        .with(blue_physics)
        .with(blue_trans)
        .build();

    //Add rigidbodies to tanks
    let tank_config = world.fetch::<TankConfig>();
    let phys_world = world.fetch::<PhysicsWorld<f32>>();
    //Set the gravity to zero
    phys_world
        .world_server()
        .set_gravity(&na::Vector3::repeat(0.0));

    let tank_rb_desc = RigidBodyDesc {
        mode: BodyMode::Dynamic,
        mass: tank_config.mass,
        friction: tank_config.friction,
        bounciness: tank_config.bounciness,
        lock_rotation_x: true,
        lock_rotation_y: true,
        lock_translation_z: true,
        ..Default::default()
    };

    //Set the tank's positions
    let mut storage = world.write_storage::<Physics>();

    let mut phys = storage.get_mut(blue).expect("Failed to get tank physics");
    let blue_rb_handle = phys_world.rigid_body_server().create(&tank_rb_desc);
    phys.rb_handle = Some(blue_rb_handle.clone());
    phys_world
        .rigid_body_server()
        .set_transform(phys.rb_handle.as_ref().unwrap().get(), &blue_pos);

    let mut phys = storage.get_mut(red).expect("Failed to get tank physics");
    let red_rb_handle = phys_world.rigid_body_server().create(&tank_rb_desc);
    phys.rb_handle = Some(red_rb_handle.clone());
    phys_world
        .rigid_body_server()
        .set_transform(phys.rb_handle.as_ref().unwrap().get(), &red_pos);

    //Set the tank's shapes
    let shape_desc = ShapeDesc::Cube {
        half_extents: na::Vector3::new(
            tank_config.size_x as f32 / 2.0,
            tank_config.size_y as f32 / 2.0,
            3.0,
        ),
    };

    let shape_tag = phys_world.shape_server().create(&shape_desc);
    phys_world
        .rigid_body_server()
        .set_shape(blue_rb_handle.get(), Some(shape_tag.get()));
    phys_world
        .rigid_body_server()
        .set_shape(red_rb_handle.get(), Some(shape_tag.get()));
}
