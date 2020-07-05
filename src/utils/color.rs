
use serde::{Serialize, Deserialize};
use amethyst::{
    assets::{Handle, Asset},
    ecs::VecStorage,
    renderer::palette::rgb::Rgba,
};
use crate::markers::ColorKey;
#[derive(Serialize, Deserialize)]
pub struct Colorscheme {
    pub background: Rgba,
    pub text: Rgba,
    pub walls: Rgba,
    pub p1: Rgba,
    pub p2: Rgba,
    pub p3: Rgba,
    pub p4: Rgba,
}
impl Asset for Colorscheme {
    const NAME: &'static str = "tanks::Colorscheme";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Colorscheme>>;
}
impl Colorscheme {
    pub fn get_by_key(&self, key: &ColorKey) -> Rgba {
        use ColorKey::*;
        match key {
            Background => self.background,
            Text => self.text,
            Walls => self.walls,
            P1 => self.p1,
            P2 => self.p2,
            P3 => self.p3,
            P4 => self.p4,
        }
    }
}

pub struct ColorschemeSet {
    pub schemes: Vec<(String, Handle<Colorscheme>)>,
    // Because we never remove from colorschemes, this should always be valid
    current: usize,
}
impl ColorschemeSet {
    /// Create a new colorscheme set. Remember to add a colorscheme and set it current, or the next system that uses colors will panic.
    pub fn new() -> Self {
        Self { schemes: Vec::new(), current: 0 }
    }
    /// Add a colorscheme
    pub fn add_scheme(&mut self, name: String, scheme: Handle<Colorscheme>) {
        self.schemes.push((name, scheme))
    }
    /// Add a colorscheme and make it current
    pub fn add_current_scheme(&mut self, name: String, scheme: Handle<Colorscheme>) {
        self.add_scheme(name, scheme.clone());
        self.current = self.schemes.len() - 1;
    }
    #[allow(dead_code)]
    pub fn get_scheme(&self, name: String) -> Handle<Colorscheme> {
        self.schemes.iter()
            .find(|(item_name, _)| item_name == &name)
            .map(|(_, handle)| handle)
            .expect(&format!("Colorscheme \"{}\" not found", name)).clone()
    }
    pub fn get_current(&self) -> Handle<Colorscheme> {
        self.schemes[self.current].1.clone()
    }
    pub fn set_current(&mut self, name: String) {
        self.current = self.schemes.iter().enumerate()
            .find(|(_, (item_name, _))| item_name == &name)
            .map(|(index, _)| index)
            .expect(&format!("Colorscheme \"{}\" not found", name));
    }
    pub fn cycle_schemes(&mut self) {
        self.current += 1;
        if self.current == self.schemes.len() {
            self.current = 0;
        }
    }
}