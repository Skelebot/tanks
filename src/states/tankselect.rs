use nalgebra as na;
use amethyst::{
    core::Transform,
    assets::AssetStorage,
    prelude::*,
    ui::{UiFinder, UiCreator, UiEvent, UiEventType, UiImage, UiText, UiTransform},
    input::{VirtualKeyCode, get_key, ElementState, is_close_requested, is_key_down},
    renderer::resources::Tint,
    ecs::{Entity, ReadStorage, WriteStorage},
};
//use crate::graphics::{TintBox, ShapeRender, CircleMesh, QuadMesh};
use crate::utils::{color::Colorscheme, color::ColorschemeSet};
use crate::markers::{DynamicColorMarker, ColorKey};
use crate::graphics::{TriangleMesh, ShapeRender};
use super::GameplayState;

pub struct TankSelect {
    ui_root: Option<Entity>,
}

impl TankSelect {
    pub fn new() -> Self {
        TankSelect {
            ui_root: None,
        }
    }
}

impl SimpleState for TankSelect {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        self.ui_root = Some(
            data.world.exec(|mut creator: UiCreator|
                creator.create("ui/tank_select_screen.ron", ())
            )
        )
    }
    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }
    fn handle_event(
            &mut self,
            _data: StateData<'_, GameData<'_, '_>>,
            event: StateEvent,
        ) -> SimpleTrans {
        match event {
            StateEvent::Ui(UiEvent {event_type: UiEventType::Click, target: Entity }) => {

            },
            StateEvent::Window(win_event) => {

            },
            StateEvent::Input(amethyst::input::InputEvent::ActionPressed(action)) => {

            },
            _ => (),
        }
    }
}