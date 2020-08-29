use amethyst::{
    ecs::{System, ReadStorage, WriteStorage, Read, ReadExpect, Join},
    assets::AssetStorage,
    renderer::resources::Tint,
};

use crate::graphics::SecondaryColor;

use crate::utils::color::{Colorscheme, ColorschemeSet};
use crate::markers::{DynamicColorMarker, DynamicSecondaryColorMarker};

pub struct ColorSystem;
impl<'s> System<'s> for ColorSystem {
    type SystemData = (
        WriteStorage<'s, Tint>,
        WriteStorage<'s, SecondaryColor>,
        ReadStorage<'s, DynamicColorMarker>,
        ReadStorage<'s, DynamicSecondaryColorMarker>,

        ReadExpect<'s, ColorschemeSet>,
        Read<'s, AssetStorage<Colorscheme>>,
    );
    fn run(&mut self, (mut tints, mut sec_colors, dyn_color_markers, dyn_sec_color_markers, colorscheme_set, colorschemes_assets): Self::SystemData) {

        // Get the current colorscheme
        let colorscheme = colorschemes_assets.get(&colorscheme_set.get_current()).unwrap();
        
        // For every entity with a Tint and a DynamicColorMarker, get the coresponding color
        // from the colorcheme, and change it
        // This allows us to change colorschemes on-the-fly and we don't have to care about 
        // exact colors when creating entities, because this will match colors every time
        // before rendering.
        for (mut tint, dyn_color_marker) in (&mut tints, &dyn_color_markers).join() {
            let mut color = colorscheme.get_by_key(&dyn_color_marker.0);
            // Respect previous alpha (for example when hiding sprites)
            color.alpha = tint.0.alpha;
            tint.0 = color;
        }
        for (mut sec_color, dyn_sec_color_marker) in (&mut sec_colors, &dyn_sec_color_markers).join() {
            let mut color = colorscheme.get_by_key(&dyn_sec_color_marker.0);
            // Respect previous alpha (for example when hiding sprites)
            color.alpha = (sec_color.0).0.alpha;
            (sec_color.0).0 = color;
        }
    }
}