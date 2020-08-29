use amethyst::{
    input::BindingTypes,
};
use serde::{Deserialize, Serialize};
pub type PlayerId = u8;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisBinding {
    Throttle(PlayerId),
    Steering(PlayerId),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionBinding {
    Shoot(PlayerId),
    Ability(PlayerId),
}

#[derive(Debug)]
pub struct TankBindingTypes;
impl BindingTypes for TankBindingTypes {
    type Axis = AxisBinding;
    type Action = ActionBinding;
}