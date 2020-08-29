use amethyst::ecs::{Component, DenseVecStorage};

mod pod;

mod flat2d;
pub use flat2d::*;
mod shapes;
pub use shapes::*;

use amethyst::assets::Handle;
use amethyst::renderer::types::Mesh;
use amethyst::renderer::resources::Tint;

// Default triangle mesh for systems to clone
pub struct TriangleMesh {
    pub handle: Handle<Mesh>
}

// Default quad mesh for systems to clone
pub struct QuadMesh {
    pub handle: Handle<Mesh>
}

// Default circle mesh for systems to clone
pub struct CircleMesh {
    pub handle: Handle<Mesh>
}

pub struct TintBox(pub [f32; 4]);
impl Component for TintBox {
    type Storage = DenseVecStorage<Self>;
}

pub struct SecondaryColor(pub Tint);
impl Component for SecondaryColor {
    type Storage = DenseVecStorage<Self>;
}
// Here goes the longest doc comment i have ever written in comparison to how long the actual function is

/// Map a number from one range of numbers to another
///
/// # Arguments
/// * x: Reference number from the first range
/// * ob: Original range bottom boundary
/// * ot: Original range top boundary
/// * nb: New range bottom boundary
/// * nt: New range top boundary
///
/// # Notice
/// `map_range` works with everything that implements the `Add`, `Sub`, `Mul`, `Div` and `Copy` traits,
/// but that doesn't mean it *should* be used with every such type. On some types this can be completely
/// meaningless at best, and if used carelessly can cause unidentified behavior.
/// Either way, it's best used only with core types such as floats or unsigned integers
///
/// # Notice 2
/// `map_range` can be used with integers but uses division, so it can cause unexpected results
/// because of integer rounding, especially when used with signed integers:
/// ```
/// // Because (x < 0)/(n > x | n < -x) (assuming x and n are both integers)
/// // always rounds towards zero, this function will return zero when used with signed integers
/// assert_eq!(map_range(-5, 0, 10, 0, 100), 0);
/// // When used with floats, it will work correctly
/// assert_eq!(map_range(-5., 0., 10., 0., 100.), -50);
/// ```
///
/// # Notice 3
/// `map_range`'s x argument is called a "Reference number" not a "Number from original range", because
/// this function supports numbers from *outside* the original range, but the returned number will (obviously)
/// be outside the new range, so take care to avoid overflows in such cases.
/// ```
/// assert_eq!(map_range(20, 0, 10, 0, 100), 200);
/// ```
use core::ops::{Add, Sub, Mul, Div};
pub fn map_range<T> (x: T, ob: T, ot: T, nb: T, nt: T) -> T
where
    T:  Add<Output = T> +
        Sub<Output = T> +
        Mul<Output = T> +
        Div<Output = T> +
        Copy
{
    (x - ob) / (ot - ob) * (nt - nb) + nb
}

#[test]
fn test_map_range() {
    assert_eq!(map_range(5., 0., 10., 0., 100.), 50.);
    assert_eq!(map_range(10., 0., 10., 0., 100.), 100.);

    assert_eq!(map_range(30, 0, 10, 0, 100), 300);

    assert_eq!(map_range(-10, 0, 10, 0, 100), -100);

    map_range(-5., 0., 10., 0., 100.);
}