use nphysics2d as np;
use ncollide2d as nc;
use nalgebra as na;
use amethyst::{
    assets::Loader,
    core::transform::Transform,
    input::{is_close_requested, is_key_down, VirtualKeyCode, get_key, ElementState},
    prelude::*,
    renderer::{
        resources::Tint,

        SpriteRender
    },
    window::ScreenDimensions,
    ui::{Anchor, TtfFormat, UiText, UiTransform},
    ecs::{Dispatcher, DispatcherBuilder},
    core::ArcThreadPool,
};
use crate::graphics::TintBox;
use crate::markers::{DynamicColorMarker, ColorKey};
use crate::utils::TanksSpriteSheet;
use crate::level::MazeLevel;
use crate::config::TankConfig;
use crate::tank::{Tank, Team};
use crate::scoreboard::Scoreboard;
use crate::weapons::Weapon;

use crate::physics;
use crate::systems;

#[derive(Default)]
pub struct GameplayState<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for GameplayState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        let mut dispatcher = 
            DispatcherBuilder::new()
                .with(systems::LevelSystem, "level_system", &[])
                .with(systems::TankSystem, "tank_system", &[/*"input_system",*/ "level_system"])
                //.with(systems::SpawnSystem::default(), "spawn_system", &["level_system"])

                .with_barrier()
                .with(systems::BeamerSystem, "beamer_system", &[])
                .with(systems::CannonSystem, "cannon_system", &[])
                // .with(systems::RocketSystem, "rocket_system", &["spawn_system"])

                .with_barrier()
                .with(systems::DestroySystem, "destroy_system", &[])
                .with(systems::CameraShakeSystem, "shake_system", &["destroy_system"])

                .with_barrier()
                .with(physics::StepperSystem, "stepper_system", &[])
                .with(physics::PTTSystem, "physics_to_transform_system", &["stepper_system"])

                .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
                .build();

        dispatcher.setup(world);
        self.dispatcher = Some(dispatcher);

        let dimensions = (*world.fetch::<ScreenDimensions>()).clone();
        // Initialize the level
        init_level(world, &dimensions);
        // Initialize players
        init_players(world);
        // Initialize the scoreboard
        init_scoreboard(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }
        Trans::None
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
                if event.0 == VirtualKeyCode::B && event.1 == ElementState::Released {
                    // Reset the level
                    data.world.write_resource::<MazeLevel>()
                        .reset_timer.replace(0.1);
                }
                if event.0 == VirtualKeyCode::H && event.1 == ElementState::Pressed {
                    use crate::utils::color::ColorschemeSet;
                    data.world.write_resource::<ColorschemeSet>().cycle_schemes();
                }
            }
        }
        Trans::None
    }
}


/// Initialize the level in the middle of the game's screen
fn init_level(world: &mut World,  dimensions: &ScreenDimensions) {
    // It's up to this function which type and what size of level we should create
    let maze = MazeLevel::new(world, dimensions);
    world.insert(maze);
}

/// Initialize the UI score counters and the Scoreboard Resource
fn init_scoreboard(world: &mut World){
    // TODO_M: Config for text, general design
    let margin = 50.0;
    let padding = 10.0;

    let text_width = 50.0;
    let score_width = 50.0;
    let text_height = 24.0;

    let default_color = [1.0, 1.0, 1.0, 1.0];

    // TODO_H: Move those to ui/ asset files and load with UiLoader
    let p1_text_trans = UiTransform::new(
        "p1_text".to_string(), Anchor::BottomLeft, Anchor::BottomLeft,
        margin,
        margin, 
        1.2,
        text_width,
        text_height,
    );
    let p1_score_trans = UiTransform::new(
        "p1_text".to_string(), Anchor::BottomLeft, Anchor::BottomLeft,
        p1_text_trans.local_x + p1_text_trans.width + padding,
        margin, 
        1.2,
        score_width,
        text_height,
    );
    let p2_text_trans = UiTransform::new(
        "p2_text".to_string(), Anchor::BottomLeft, Anchor::BottomLeft,
        p1_score_trans.local_x + p1_score_trans.width + padding,
        margin, 
        1.2,
        text_width,
        text_height,
    );
    let p2_score_trans = UiTransform::new(
        "p2_score".to_string(), Anchor::BottomLeft, Anchor::BottomLeft,
        p2_text_trans.local_x + p2_text_trans.width + padding,
        margin, 
        1.2,
        score_width,
        text_height,
    );

    // Load the font for the numbers
    let font = world.read_resource::<Loader>().load(
        "fonts/BalooThambi2-Regular.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    ); 

    // Red tank's score counter
    let _p1_text = world
        .create_entity()
        .with(p1_text_trans)
        .with(UiText::new(
            font.clone(),
            "P1: ".to_string(),
            default_color,
            50.,
        ))
        .with(Tint(Default::default()))
        .with(DynamicColorMarker(ColorKey::P1))
        .build();
    let p1_score = world
        .create_entity()
        .with(p1_score_trans)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            default_color,
            50.,
        ))
        .with(Tint(Default::default()))
        .with(DynamicColorMarker(ColorKey::Text))
        .build();

    let _p2_text = world
        .create_entity()
        .with(p2_text_trans)
        .with(UiText::new(
            font.clone(),
            "P2: ".to_string(),
            default_color,
            50.,
        ))
        .with(Tint(Default::default()))
        .with(DynamicColorMarker(ColorKey::P2))
        .build();
    let p2_score = world
        .create_entity()
        .with(p2_score_trans)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            default_color,
            50.,
        ))
        .with(Tint(Default::default()))
        .with(DynamicColorMarker(ColorKey::Text))
        .build();
    
    // Scoreboard resource
    let mut scoreboard = Scoreboard::new();
    scoreboard.texts.push(p1_score);
    scoreboard.texts.push(p2_score);
    world.insert(scoreboard);
}

/// Create entities for both player's tanks
fn init_players(world: &mut World) {

    // Fetch the config for tank's entities, it should be loaded on game data creation
    let tank_config = (*world.read_resource::<TankConfig>()).clone();

    //TODO: Create a trait for levels and change to <Level> (or Box<Level>?)
    // Fetch the level's starting positions (the Level should be crated before initializing players)
    let starting_positions = world.read_resource::<MazeLevel>().starting_positions;

    // Create the SpriteRenders
    let sprites: Vec<SpriteRender> = tank_config.sprite_nums.iter()
        .map(|i| SpriteRender {
            sprite_sheet: world.fetch::<TanksSpriteSheet>().handle.clone(),
            sprite_number: *i,
        }).collect();
    
    // Set tanks' transforms to level's starting positions
    // Red tank's Transform
    let mut p1_transform = Transform::default();
    p1_transform.set_translation(starting_positions[0].translation.vector.push(0.0));
    p1_transform.set_rotation_z_axis(starting_positions[0].rotation.angle());
    let p1_position = starting_positions[0];

    // Blue tank's Transform
    let mut p2_transform = Transform::default();
    p2_transform.set_translation(starting_positions[1].translation.vector.push(0.0));
    p2_transform.set_rotation_z_axis(starting_positions[1].rotation.angle());
    let p2_position = starting_positions[1];

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

    let p1_body = physics::Body {
        handle: world.fetch_mut::<physics::Physics>().add_rigid_body(
            tank_rb_desc.clone()
                .position(p1_position)
                .build()
        )
    };
    let p1_collider = physics::Collider {
        handle: world.fetch_mut::<physics::Physics>().add_collider(
            tank_col_desc.build(np::object::BodyPartHandle(p1_body.handle, 0))
        )
    };
    let p2_body = physics::Body {
        handle: world.fetch_mut::<physics::Physics>().add_rigid_body(
            tank_rb_desc
                .position(p2_position)
                .build()
        )
    };
    let p2_collider = physics::Collider {
        handle: world.fetch_mut::<physics::Physics>().add_collider(
            tank_col_desc.build(np::object::BodyPartHandle(p2_body.handle, 0))
        )
    };

    use crate::graphics::map_range;
    // Create TintBoxes for the tanks, so that only their bodies are colored.
    // TintBoxes are in texture coordinates (for now?) so we have to translate coords
    // using the map_range function
    // This can be easily done in fragment shader
    // TODO_H: Make TintBoxes use texture coords
    let x = map_range(4., 0., tank_config.size_x as f32, -0.5, 0.5);
    let y = map_range(0., 0., tank_config.size_y as f32, -0.5, 0.5);
    let width = map_range(16., 0., tank_config.size_x as f32, 0.0, 1.0);
    let height = map_range(tank_config.size_y as f32, 0., tank_config.size_y as f32, 0.0, 1.0);

    // Create the p1 tank
    world.create_entity()
        .with(Tank::new(Team::P1, Weapon::default()))
        .with(sprites[0].clone())
        .with(Tint(Default::default()))
        .with(DynamicColorMarker(ColorKey::P1))
        .with(TintBox([x, y, width, height]))
        .with(p1_body)
        .with(p1_collider)
        .with(p1_transform)
        .build();

    // Create the p2 tank
    world.create_entity()
        .with(Tank::new(Team::P2, Weapon::default()))
        .with(sprites[1].clone())
        .with(Tint(Default::default()))
        .with(DynamicColorMarker(ColorKey::P2))
        .with(TintBox([x, y, width, height]))
        .with(p2_body)
        .with(p2_collider)
        .with(p2_transform)
        .build();
}

// Don't even look at this
// UPDATE: I had to fork the entire amethyst repo just to NOT do this
/*
use na::{Scalar, Dim, storage::Storage};
use math::{Scalar as Scalar1, Dim as Dim1, storage::Storage as Storage1};
fn translate_vector_versions<N: Scalar + Scalar1, D: Dim + Dim1, S: Storage<N, D> + Storage1<N, D>>(vec: na::Vector<N, D, S>) -> math::Vector<N, D, S> {
    math::Vector::from_data(vec.data)
}

#[test]
fn test_vector_version_translation() {
    let vector = na::Vector2::new(2.0, 3.0);
    let vector1 = translate_vector_versions(vector);

    assert!(vector1 = math::Vector2::new(2.0, 3.0))
}
*/