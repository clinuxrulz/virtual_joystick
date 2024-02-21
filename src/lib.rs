use std::{hash::Hash, marker::PhantomData};

use bevy::{
    ecs::schedule::ScheduleLabel, prelude::*, reflect::TypePath, render::RenderApp, ui::{RenderUiSystem, UiSystem}
};

mod behaviour;
mod components;
mod input;
mod systems;
mod ui;
mod utils;

pub use behaviour::{VirtualJoystickAxis, VirtualJoystickType};
pub use components::{
    JoystickState, JoystickDeadZone, JoystickHorizontalOnly, JoystickVerticalOnly, JoystickInvisible, JoystickFixed, JoystickFloating, JoystickDynamic,
};
use systems::{update_dead_zone, update_dynamic, update_fixed, update_floating, update_horizontal_only, update_input, update_ui, update_vertical_only};
pub use utils::create_joystick;

use ui::{extract_joystick_node, VirtualJoystickData};


#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UpdateKnobDelta;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstrainKnobDelta;

#[derive(Default)]
pub struct VirtualJoystickPlugin<S> {
    _marker: PhantomData<S>,
}

pub trait VirtualJoystickID:
    Hash + Sync + Send + Clone + Default + Reflect + TypePath + FromReflect + 'static
{
}

impl<S: Hash + Sync + Send + Clone + Default + Reflect + FromReflect + TypePath + 'static>
    VirtualJoystickID for S
{
}

impl<S: VirtualJoystickID> Plugin for VirtualJoystickPlugin<S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<VirtualJoystickInteractionArea>()
            .register_type::<VirtualJoystickNode<S>>()
            .register_type::<VirtualJoystickData>()
            .register_type::<VirtualJoystickAxis>()
            .register_type::<VirtualJoystickType>()
            .register_type::<VirtualJoystickEventType>()
            .add_event::<VirtualJoystickEvent<S>>()
            .add_event::<InputEvent>()
            .add_stage_before(Update, UpdateKnobDelta)
            .add_stage_after(UpdateKnobDelta, ConstrainKnobDelta)
            .add_systems(PreUpdate, update_input)
            .add_systems(
                UpdateKnobDelta,
                (
                    update_fixed,
                    update_floating,
                    update_dynamic,
                )
            )
            .add_systems(
                ConstrainKnobDelta,
                (
                    update_dead_zone,
                    update_horizontal_only,
                    update_vertical_only,
                )
            );

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.add_systems(
            ExtractSchedule,
            update_ui.after(RenderUiSystem::ExtractNode),
        );
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum VirtualJoystickEventType {
    Press,
    Drag,
    Up,
}

#[derive(Event, Debug)]
pub struct VirtualJoystickEvent<S: VirtualJoystickID> {
    id: S,
    event: VirtualJoystickEventType,
    value: Vec2,
    delta: Vec2,
    axis: VirtualJoystickAxis,
}

impl<S: VirtualJoystickID> VirtualJoystickEvent<S> {
    /// Get ID of joystick throw event
    pub fn id(&self) -> S {
        self.id.clone()
    }
    /// Raw position of point (Mouse or Touch)
    pub fn value(&self) -> Vec2 {
        self.value
    }

    /// Axis of Joystick see [crate::VirtualJoystickAxis]
    pub fn direction(&self) -> VirtualJoystickAxis {
        self.axis
    }

    /// Delta value ranging from 0 to 1 in each vector (x and y)
    pub fn axis(&self) -> Vec2 {
        self.delta
    }

    /// Return the Type of Joystick Event
    pub fn get_type(&self) -> VirtualJoystickEventType {
        self.event
    }

    /// Delta value snaped
    /// the dead_zone is required for make more customizable
    /// the default of the dead_zone is 0.5
    pub fn snap_axis(&self, dead_zone: Option<f32>) -> Vec2 {
        let dead_zone = dead_zone.unwrap_or(0.5);
        let x = if self.axis == VirtualJoystickAxis::Both
            || self.axis == VirtualJoystickAxis::Horizontal
        {
            if self.delta.x > dead_zone {
                1.
            } else if self.delta.x < -dead_zone {
                -1.
            } else {
                0.
            }
        } else {
            0.
        };
        let y = if self.axis == VirtualJoystickAxis::Both
            || self.axis == VirtualJoystickAxis::Vertical
        {
            if self.delta.y > dead_zone {
                1.
            } else if self.delta.y < -dead_zone {
                -1.
            } else {
                0.
            }
        } else {
            0.
        };

        Vec2::new(x, y)
    }
}
