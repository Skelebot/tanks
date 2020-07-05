use amethyst::ecs::{Component, DenseVecStorage};

mod pod;

mod flat2d;
pub use flat2d::*;
mod shapes;
pub use shapes::*;

use amethyst::assets::Handle;
use amethyst::renderer::types::Mesh;

// All default meshes are loaded in loading state
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

    assert_eq!(map_range(30, 0, 10, 0, 100), 3000);

    assert_eq!(map_range(-1, 0, 10, 0, 100), -10);
}