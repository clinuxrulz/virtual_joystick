use bevy::prelude::*;

#[cfg(feature = "inspect")]
use bevy_inspector_egui::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
pub enum VirtualJoystickAxis {
    #[default]
    Both,
    Horizontal,
    Vertical,
}

#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
pub enum VirtualJoystickType {
    /// Static position
    Fixed,
    #[default]
    /// Spawn at point click
    Floating,
    /// Follow point on drag
    Dynamic,
}
