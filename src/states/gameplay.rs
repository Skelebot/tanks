use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{
        transform::Transform,
    },
    input::{is_close_requested, is_key_down, VirtualKeyCode, get_key, ElementState},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};

use crate::utils::SpriteSheetRes;
use crate::level::MazeLevel;
use crate::config::TankConfig;
use crate::tank::{Tank, Team};

use crate::physics;

pub struct GameplayState {
    pub maze_r: bool,
}
impl SimpleState for GameplayState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.insert(physics::Physics::new());

        // Get the screen dimensions so we can initialize the camera and
        // place our sprites correctly later. We'll clone this since we'll
        // pass the world mutably to the following functions
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

        // Place the camera
        init_camera(world, &dimensions);

        // Load our sprite sheet and insert it into the World
        // as a Resource, so that Systems can use it
        let sprite_sheet_handle = load_sprite_sheet(world);
        let sprite_sheet_res = SpriteSheetRes {
            handle: Some(sprite_sheet_handle.clone())
        };
        world.insert(sprite_sheet_res);

        // Initialize the level
        init_level(world, &dimensions);
        // Initialize players
        init_players(world, sprite_sheet_handle.clone(), &dimensions);
    }

    // Handle keyboard and window events,
    // Exit the state if window close was requested,
    // or if user is holding the ESC key
    // TODO: Handle window resizing (reposition the camera, players and maze)
    fn handle_event(&mut self, data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit
            }
            if let Some(event) = get_key(&event) {
                if event.0 == VirtualKeyCode::B && event.1 == ElementState::Pressed && !self.maze_r {
                    let mut level = data.world.write_resource::<MazeLevel>();
                    level.should_be_reset = true;
                }
            }
        }
        
        Trans::None
    }
}

/// Initialize the level in the middle of the game's screen
fn init_level(world: &mut World, dimensions: &ScreenDimensions) {
    // It's up to this function which type and what size of level we should create
    let maze = MazeLevel::new(world, dimensions);
    world.insert(maze);
}

/// Load the texture and sprite definition metatdata 
/// for our sprites and combine them into a SpriteSheet.
/// Return handle to this SpriteSheet that can be used to
/// create SpriteRender's for entities and/or
/// be inserted into the World in a SpriteSheetRes.
fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    let loader = world.read_resource::<Loader>();
    // Load the texture
    let texture_handle = {
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "sprites/tanks.png",
            ImageFormat::default(),
            (),
            &texture_storage
        )
    };
    // Load the spritesheet definition file
    let sheet_handle = {
        let sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
        loader.load(
            "sprites/tanks.ron",
            SpriteSheetFormat(texture_handle),
            (),
            &sheet_storage
        )
    };
    return sheet_handle;
}

/// Initialize a 2D orhographic camera placed in the middle
/// of the screen, and covering the entire screen
fn init_camera(world: &mut World, dimensions: &ScreenDimensions) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(
        dimensions.width() * 0.5,
        dimensions.height() * 0.5,
        2.0);
    world
        .create_entity()
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build();
}

/// Create entities for both player's tanks
fn init_players(world: &mut World, sheet_handle: Handle<SpriteSheet>, _dimensions: &ScreenDimensions) {

    // Fetch the config for tank's entities, it should be loaded on game data creation
    let tank_config = (*world.read_resource::<TankConfig>()).clone();

    //TODO: Create a trait for levels and change to <Level> (or Box<Level>?)
    // Fetch the level's starting positions (the Level should be crated before initializing players)
    let starting_positions = (world.read_resource::<MazeLevel>().starting_positions).clone();

    // Create the SpriteRenders
    //TODO: Determine players' sprite numbers from a config (prefab?)
    let sprites: Vec<SpriteRender> = tank_config.sprite_nums.iter()
        .map(|i| SpriteRender {
            sprite_sheet: sheet_handle.clone(),
            sprite_number: *i,
        }).collect();
    
    // Set tanks' transforms to level's starting positions
    // Red tank's Transform
    let mut red_transform = Transform::default();
    red_transform.set_translation(
        amethyst::core::math::Vector3::new(
            starting_positions[0].x,
            starting_positions[0].y,
            1.0
        )
    );
    // Amethyst's Transform is in 3D, to create a 2D RigidBody we have to determine it's 2D translation + rotation
    let red_position: na::Isometry2<f32> = 
        na::Isometry2::new(
            na::Vector2::new(red_transform.translation().x, red_transform.translation().y),
            red_transform.rotation().angle(),
        );

    // Blue tank's Transform
    let mut blue_transform = Transform::default();
    blue_transform.set_translation(
        amethyst::core::math::Vector3::new(
            starting_positions[1].x,
            starting_positions[1].y,
            1.0
        )
    );
    // Amethyst's Transform is in 3D, to create a 2D RigidBody we have to determine it's 2D translation + rotation
    let blue_position: na::Isometry2<f32> = 
        na::Isometry2::new(
            na::Vector2::new(blue_transform.translation().x, blue_transform.translation().y),
            blue_transform.rotation().angle(),
        );

    // Create the shape for tanks
    let tank_shape = nc::shape::ShapeHandle::new(nc::shape::Cuboid::new(na::Vector2::new(
        tank_config.size_x as f32 * 0.5,
        tank_config.size_y as f32 * 0.5,
    )));

    let tank_col_desc = np::object::ColliderDesc::new(tank_shape)
        .density(tank_config.density);

    // Create the RigidBody description to be cloned for both tanks
    let mut tank_rb_desc = np::object::RigidBodyDesc::new();
    tank_rb_desc
        .set_max_linear_velocity(tank_config.max_linear_vel)
        .set_max_angular_velocity(tank_config.max_angular_vel)
        .set_linear_damping(tank_config.linear_damping)
        .set_angular_damping(tank_config.angular_damping);

    let red_body = physics::Body {
        handle: world.fetch_mut::<physics::Physics>().add_rigid_body(
            tank_rb_desc.clone()
                .position(red_position)
                .build()
        )
    };
    let red_collider = physics::Collider {
        handle: world.fetch_mut::<physics::Physics>().add_collider(
            tank_col_desc.build(np::object::BodyPartHandle(red_body.handle.clone(), 0))
        )
    };
    let blue_body = physics::Body {
        handle: world.fetch_mut::<physics::Physics>().add_rigid_body(
            tank_rb_desc
                .position(blue_position)
                .build()
        )
    };
    let blue_collider = physics::Collider {
        handle: world.fetch_mut::<physics::Physics>().add_collider(
            tank_col_desc.build(np::object::BodyPartHandle(blue_body.handle.clone(), 0))
        )
    };

    // Create the red tank
    world.create_entity()
        .with(Tank::new(Team::Red))
        .with(sprites[0].clone())
        .with(red_body)
        .with(red_collider)
        .with(red_transform)
        .build();
    
    // Create the blue tank
    world.create_entity()
       .with(Tank::new(Team::Blue))
       .with(sprites[1].clone())
       .with(blue_body)
       .with(blue_collider)
       .with(blue_transform)
       .build();
}