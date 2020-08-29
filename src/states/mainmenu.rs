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

macro_rules! dyn_color_text_logic {
    ($text:expr, $data:expr, $color:expr) => {
        if let Some(entity) = $text {
            let mut storage = $data.world.system_data::<WriteStorage<UiText>>();
            let t = storage.get_mut(entity);
            if let Some(text) = t {
                text.color = $color.into_pod();
            }
        }
    };
}
macro_rules! dyn_color_container_logic {
    ($container:expr, $data:expr, $color:expr) => {
        if let Some(pane) = $container {
            let mut storage = $data.world.system_data::<WriteStorage<UiImage>>();
            let img = storage.get_mut(pane);
            if let Some(UiImage::SolidColor(color)) = img {
                *color = $color.into_pod();
            }
        }
    }
}

pub struct MainMenuState {
    // The root pane of the UI
    // When removed, deletes the whole main_menu ui
    // Entity the cursor is next to (usually UiText)
    cursor: Option<Entity>,
    cursor_pos: Option<u8>,
    controlling_player: crate::tank::Team,
    ui_root: Option<Entity>,
    main_menu_pane: Option<Entity>,
    top_color_box: Option<Entity>,
    bottom_color_box: Option<Entity>,
    play_text: Option<Entity>,
    settings_text: Option<Entity>,
    exit_text: Option<Entity>,
}

impl MainMenuState {
    pub fn new(controlling_player: crate::tank::Team) -> Self {
        Self {
            cursor: None,
            cursor_pos: None,
            controlling_player,
            ui_root: None,
            main_menu_pane: None,
            top_color_box: None,
            bottom_color_box: None,
            play_text: None,
            settings_text: None,
            exit_text: None,
        }
    }
    fn update_cursor_pos(&mut self, world: &mut World, next_to: Option<u8>) {
        if let Some(next_to_num) =  next_to {
            let next_to = match next_to_num {
                0 => self.play_text.unwrap(),
                1 => self.settings_text.unwrap(),
                2 => self.exit_text.unwrap(),
                _ => unreachable!()
            };
            let storage = world.system_data::<ReadStorage<UiTransform>>();
            let trans = storage.get(next_to).unwrap();
            let mut storage = world.system_data::<WriteStorage<Transform>>();
            let cursor_trans = storage.get_mut(self.cursor.unwrap()).unwrap();
            cursor_trans.set_translation_xyz(
                trans.pixel_x() - 20.,
                trans.pixel_y() + (trans.height / 2.),
                1.1,//trans.global_z(),
            );
        }
    }
}

impl SimpleState for MainMenuState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        // Create loading ui
        self.ui_root = Some(
            data.world.exec(|mut creator: UiCreator| 
                creator.create("ui/main_menu_text.ron", ())
            )
        );

        let mut color = {
            let colorscheme_assets = data.world.fetch::<AssetStorage<Colorscheme>>();
            let colorscheme_set = data.world.fetch::<ColorschemeSet>();
            let colorscheme = colorscheme_assets.get(&colorscheme_set.get_current()).unwrap();
            colorscheme.get_by_team(&self.controlling_player).clone()
        };
        color.alpha = 1.0;

        let triangle_mesh = (*data.world.fetch::<TriangleMesh>()).handle.clone();
        let mut transform = Transform::default();
        transform.set_scale(na::Vector3::new(18.0, 18.0, 1.0));
        transform.set_rotation_z_axis(-90.0_f32.to_radians());
        transform.set_translation(na::Vector3::new(500.0, 500.0, 1.1));
        // Create the cursor
        self.cursor = Some(data.world
            .create_entity()
            .with(transform)
            .with(ShapeRender {
                mesh: triangle_mesh
            })
            .with(DynamicColorMarker(ColorKey::from(self.controlling_player)))
            .with(Tint(color))
            .build());
    }

    fn handle_event(
            &mut self,
            data: StateData<'_, GameData<'_, '_>>,
            event: StateEvent,
        ) -> SimpleTrans {
        match event {
            StateEvent::Ui(UiEvent { event_type: UiEventType::Click, target }) => {
                if Some(target) == self.play_text  {
                    Trans::Push(Box::new(GameplayState::default()))
                } else if Some(target) == self.settings_text {
                    Trans::None
                } else if Some(target) == self.exit_text {
                    Trans::Quit
                } else {
                    Trans::None
                }
            },
            StateEvent::Window(win_event) => {
                if let Some(event) = get_key(&win_event) {
                    if is_close_requested(&win_event) || is_key_down(&win_event, VirtualKeyCode::Escape) {
                        return Trans::Quit
                    }
                    if event.0 == VirtualKeyCode::H && event.1 == ElementState::Pressed {
                        data.world.write_resource::<ColorschemeSet>().cycle_schemes();
                    }
                    // Move the cursor down
                    if event.0 == VirtualKeyCode::S && event.1 == ElementState::Pressed {
                        if let Some(ref mut pos) = self.cursor_pos {
                            *pos += 1;
                            if *pos > 2 { *pos = 0 }
                        }
                    }
                    // Move the cursor up
                    if event.0 == VirtualKeyCode::W && event.1 == ElementState::Pressed {
                        if let Some(ref mut pos) = self.cursor_pos {
                            if *pos == 0 { *pos = 3 }
                            *pos -= 1;
                        }
                    }
                    // "Press" the button under the cursor
                    if (event.0 == VirtualKeyCode::D && event.1 == ElementState::Pressed)
                        || (event.0 == VirtualKeyCode::Return && event.1 == ElementState::Pressed) {
                            if let Some(loc) = self.cursor_pos {
                            match loc {
                                // Play
                                0 => return Trans::Push(Box::new(GameplayState::default())),
                                // Settings
                                1 => return Trans::None,
                                // Exit
                                2 => return Trans::Quit,
                                _ => unreachable!(),
                            }
                        }
                    }
                }
                Trans::None
            },
            // TODO_H: Proper controlling player handling
            // For some reason StateEvent::Input is broken in my version of Amethyst
            // so we have to do everything manually
            _ => Trans::None
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.play_text.is_none()
            || self.main_menu_pane.is_none()
            || self.settings_text.is_none()
            || self.exit_text.is_none()
            || self.top_color_box.is_none()
            || self.bottom_color_box.is_none()
        {
            data.world.exec(|ui_finder: UiFinder<'_>| {
                self.main_menu_pane = ui_finder.find("main_menu_pane");
                self.top_color_box = ui_finder.find("top_color_box");
                self.bottom_color_box = ui_finder.find("bottom_color_box");
                self.play_text = ui_finder.find("play_text");
                self.settings_text = ui_finder.find("settings_text");
                self.exit_text = ui_finder.find("exit_text");
            });
        } else if self.cursor_pos.is_none() {
            self.cursor_pos = Some(0);
        }

        self.update_cursor_pos(&mut data.world, self.cursor_pos);

        let colorscheme_assets = data.world.fetch::<AssetStorage<Colorscheme>>();
        let colorscheme_set = data.world.fetch::<ColorschemeSet>();
        let colorscheme = colorscheme_assets.get(&colorscheme_set.get_current()).unwrap();

        use amethyst::renderer::pod::IntoPod;

        dyn_color_text_logic!(self.play_text, data, colorscheme.text);
        dyn_color_text_logic!(self.settings_text, data, colorscheme.text);
        dyn_color_text_logic!(self.exit_text, data, colorscheme.text);

        //dyn_color_container_logic!(self.main_menu_pane, data, colorscheme.background);
        dyn_color_container_logic!(self.top_color_box, data, colorscheme.p1);
        dyn_color_container_logic!(self.bottom_color_box, data, colorscheme.p2);

        Trans::None
    }

    fn on_pause(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(root_entity) = self.ui_root {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove main_menu")
        }
        if let Some(cursor) = self.cursor {
            data.world
                .delete_entity(cursor)
                .expect("Couldn't remove cursor")
        }
        self.ui_root = None;
        self.main_menu_pane = None;
        self.top_color_box = None;
        self.bottom_color_box = None;
        self.play_text = None;
        self.settings_text = None;
        self.exit_text = None;
    }
}