use amethyst::{
    prelude::*,
    ui::{UiFinder, UiCreator},
};
use amethyst::{
    assets::{AssetStorage, Handle, Loader, AssetLoaderSystemData, ProgressCounter, Completion},
    core::transform::Transform,
    renderer::{
        shape::Shape,

        types::Mesh,
        rendy::mesh::MeshBuilder,
        rendy::core::types::vertex::Position,

        Camera, ImageFormat, SpriteSheet, SpriteSheetFormat, Texture
    },
    window::ScreenDimensions,
    utils::application_dir
};
use crate::graphics::{TintBox, ShapeRender, CircleMesh, QuadMesh};
use crate::utils::{TanksSpriteSheet, SpawnsSpriteSheet, color::Colorscheme, color::ColorschemeSet};
use crate::systems::camshake::CameraShake;

use crate::config;

use crate::physics;
use super::GameplayState;

pub struct LoadingState {
    progress: ProgressCounter,
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // Create loading ui
        data.world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/loading_text.ron", &mut self.progress);
        });

        // Load stuff
        let world = data.world;

        world.register::<TintBox>();
        world.register::<ShapeRender>();
        world.register::<physics::Body>();
        world.register::<physics::Collider>();

        // Initialize the CameraShake resource
        world.insert(CameraShake::default());
        world.insert(physics::Physics::new());

        let mut mesh_builder = MeshBuilder::new();
        mesh_builder
            .add_vertices::<Position, _>(vec![
                Position::from([-0.5, 0.5, 0.0]),
                Position::from([-0.5, -0.5, 0.0]),
                Position::from([0.5, -0.5, 0.0]),
                Position::from([0.5, 0.5, 0.0]),
            ]);
        mesh_builder.set_indices(vec![0_u16, 1, 2, 0, 2, 3]);

        let (quad, circle) = world.exec(|loader: AssetLoaderSystemData<'_, Mesh>| {
            let quad = loader.load_from_data(
                mesh_builder.into(),
                &mut self.progress
            );
            let circle = loader.load_from_data(
                Shape::Circle(16).generate::<Vec<Position>>(None).into(),
                &mut self.progress
            );
            (quad, circle)
        });

        let base = world.read_resource::<Loader>().load(
            "colors/base.ron",
            amethyst::assets::RonFormat,
            &mut self.progress,
            &world.read_resource::<AssetStorage<Colorscheme>>(),
        );

        let dark = world.read_resource::<Loader>().load(
            "colors/dark.ron",
            amethyst::assets::RonFormat,
            &mut self.progress,
            &world.read_resource::<AssetStorage<Colorscheme>>(),
        );

        let mut colorscheme_set = ColorschemeSet::new();
        colorscheme_set.add_current_scheme("base".to_string(), base);
        colorscheme_set.add_scheme("dark".to_string(), dark);
        colorscheme_set.set_current("dark".to_string());

        load_resources(world);

        world.insert(colorscheme_set);

        world.insert(QuadMesh { handle: quad });
        world.insert(CircleMesh { handle: circle });

        // Get the screen dimensions so we can initialize the camera and
        // place our sprites correctly later. We'll clone this since we'll
        // pass the world mutably to the following functions
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

        // Place the camera
        init_camera(world, &dimensions);

        // Load our sprite sheets
        let tanks_sprite_sheet = TanksSpriteSheet::new(load_sprite_sheet(world, "tanks", &mut self.progress));
        let spawns_sprite_sheet = SpawnsSpriteSheet::new(load_sprite_sheet(world, "spawns", &mut self.progress));

        // Insert the sprite sheets into the world as a `Resource`,
        // so that systems can use them
        world.insert(tanks_sprite_sheet);
        world.insert(spawns_sprite_sheet);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        match self.progress.complete() {
            Completion::Failed => {
                println!("Failed loading assets: {:?}", self.progress.errors());
                Trans::Quit
            },
            Completion::Complete => {
                println!("Assets loaded, transitioning to Gameplay");

                // Delete loading text
                let loading_text_entity = data.world.exec(|finder: UiFinder<'_>| finder.find("loading_text").unwrap());
                data.world.delete_entity(loading_text_entity).unwrap();

                Trans::Switch(Box::new(GameplayState::default()))
            }
            Completion::Loading => Trans::None
        }
    }
}

/// Load the texture and sprite definition metatdata 
/// for our sprites and combine them into a SpriteSheet.
/// Return handle to this SpriteSheet that can be used to
/// create SpriteRender's for entities and/or
/// be inserted into the World in a SpriteSheetRes.
/// 
/// # Arguments
/// 
/// * `name` - The name of the spritesheet and spritesheet metadata file;
///            the files should be called `{name}.png` and `{name}.ron` respectively
fn load_sprite_sheet(world: &mut World, name: &str, progress: &mut ProgressCounter) -> Handle<SpriteSheet> {
    let loader = world.read_resource::<Loader>();
    // Load the spritesheet texture
    let texture_handle = 
        loader.load(
            format!("sprites/{}.png", name),
            ImageFormat::default(),
            &mut *progress,
            &world.read_resource::<AssetStorage<Texture>>()
        );
    // Load the spritesheet definition file
    loader.load(
        format!("sprites/{}.ron", name),
        SpriteSheetFormat(texture_handle),
        &mut *progress,
        &world.read_resource::<AssetStorage<SpriteSheet>>()
    )
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

fn load_resources(world: &mut World) {
    let config = application_dir("res/config").unwrap();
    let tank_config         = config::TankConfig    ::load(&config.join("tank.ron"      )).unwrap();
    let maze_config         = config::MazeConfig    ::load(&config.join("maze.ron"      )).unwrap();
    let beamer_config       = config::BeamerConfig  ::load(&config.join("beamer.ron"    )).unwrap();
    let cannon_config       = config::CannonConfig  ::load(&config.join("cannon.ron"    )).unwrap();
    let spawn_config        = config::SpawnConfig   ::load(&config.join("spawn.ron"     )).unwrap();
    let destroy_config      = config::DestroyConfig ::load(&config.join("destroy.ron"   )).unwrap();

    let performance_config  = config::PerformanceConfig::load(&config.join( "performance.ron")).unwrap();

    world.insert(tank_config);
    world.insert(maze_config);
    world.insert(beamer_config);
    world.insert(cannon_config);
    world.insert(spawn_config);
    world.insert(destroy_config);
    world.insert(performance_config);
}

impl Default for LoadingState {
    fn default() -> Self {
        LoadingState {
            progress: ProgressCounter::new()
        }
    }
}