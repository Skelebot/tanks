
use serde::{Serialize, Deserialize};
use amethyst::{
    assets::{Handle, Asset, ProcessableAsset, ProcessingState},
    ecs::VecStorage,
    renderer::palette::Srgba,
    Error
};
use crate::markers::ColorKey;
#[derive(Serialize, Deserialize)]
pub struct Colorscheme {
    pub background: Srgba,
    pub text: Srgba,
    pub walls: Srgba,
    pub p1: Srgba,
    pub p2: Srgba,
    pub p3: Srgba,
    pub p4: Srgba,
}

impl Colorscheme {
    pub fn get_by_key(&self, key: &ColorKey) -> Srgba {
        use ColorKey::*;
        match key {
            Background  => self.background,
            Text        => self.text,
            Walls       => self.walls,
            P1          => self.p1,
            P2          => self.p2,
            P3          => self.p3,
            P4          => self.p4,
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
    pub fn add_scheme<S: Into<String>>(&mut self, name: S, scheme: Handle<Colorscheme>) {
        self.schemes.push((name.into(), scheme))
    }

    /// Add a colorscheme and make it current
    #[allow(dead_code)]
    pub fn add_current_scheme<S: Into<String>>(&mut self, name: S, scheme: Handle<Colorscheme>) {
        self.add_scheme(name.into(), scheme.clone());
        self.current = self.schemes.len() - 1;
    }

    #[allow(dead_code)]
    pub fn get_scheme<S: Into<String>>(&self, name: S) -> Handle<Colorscheme> {
        let s = name.into();
        self.schemes.iter()
            .find(|(item_name, _)| item_name == &s)
            .map(|(_, handle)| handle)
            .expect(&format!("Colorscheme \"{}\" not found", s)).clone()
    }

    pub fn get_current(&self) -> Handle<Colorscheme> {
        self.schemes[self.current].1.clone()
    }

    pub fn set_current<S: Into<String>>(&mut self, name: S) {
        let s = name.into();
        self.current = self.schemes.iter().enumerate()
            .find(|(_, (item_name, _))| item_name == &s)
            .map(|(index, _)| index)
            .expect(&format!("Colorscheme \"{}\" not found", s));
    }

    pub fn cycle_schemes(&mut self) {
        self.current += 1;
        if self.current == self.schemes.len() {
            self.current = 0;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorschemeData {
    pub background: u32,
    pub text: u32,
    pub walls: u32,
    pub p1: u32,
    pub p2: u32,
    pub p3: u32,
    pub p4: u32,
}
impl Asset for Colorscheme {
    const NAME: &'static str = "tanks::Colorscheme";
    type Data = ColorschemeData;
    type HandleStorage = VecStorage<Handle<Colorscheme>>;
}
impl ProcessableAsset for Colorscheme {
    fn process(data: Self::Data) -> Result<ProcessingState<Self>, Error> {
        Ok(ProcessingState::Loaded(Self {
            background: hex_to_rgba(data.background),
            text: hex_to_rgba(data.text),
            walls: hex_to_rgba(data.walls),
            p1: hex_to_rgba(data.p1),
            p2: hex_to_rgba(data.p2),
            p3: hex_to_rgba(data.p3),
            p4: hex_to_rgba(data.p4),
        }))

    }
}

/// Convert an integer representation of a hexadecimal color to Srgba
/// The alpha component is filled with 1.0
///
/// # Example
///
/// ```
/// let hex: u32 = 0xff8000;
/// let converted = hex_to_rgba(hex);
/// assert_eq!(converted, Srgba::new(1.0, 0.5, 0.0, 1.0));
/// ```
fn hex_to_rgba(hex: u32) -> Srgba {
    // Because Amethyst wants colors in non-linear space, we need to gamma-correct them first
    let gamma = 2.2;
    Srgba::from_components((
        (((hex >> 16) & 0xFF) as f32 / 255.0).powf(gamma),
        (((hex >> 8) & 0xFF) as f32 / 255.0).powf(gamma),
        ((hex & 0xFF) as f32 / 255.0).powf(gamma),
        1.0
    ))
}
