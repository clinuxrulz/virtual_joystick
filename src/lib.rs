use std::{hash::Hash, marker::PhantomData};

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::RenderApp,
    ui::{RenderUiSystem, UiSystem},
};

mod behaviour;
mod input;
mod ui;
mod utils;

pub use behaviour::{Behaviour, AxisBoth, AxisHoritonalOnly, AxisVerticalOnly, StickFixed, StickFloating, StickDynamic};
use input::{update_input, update_joystick, update_joystick_by_mouse, InputEvent};
pub use ui::{
    VirtualJoystickBundle, VirtualJoystickInteractionArea, VirtualJoystickNode,
    VirtualJoystickUIBackground, VirtualJoystickUIKnob,
};
pub use utils::create_joystick;

use ui::{extract_joystick_node, VirtualJoystickData};

#[derive(Default)]
pub struct VirtualJoystickPlugin<S, B> {
    _marker1: PhantomData<S>,
    _marker2: PhantomData<B>,
}

pub trait VirtualJoystickID:
    Hash + Sync + Send + Clone + Default + Reflect + TypePath + FromReflect + 'static
{
}

impl<S: Hash + Sync + Send + Clone + Default + Reflect + FromReflect + TypePath + 'static>
    VirtualJoystickID for S
{
}

impl<S: VirtualJoystickID, B: Behaviour> Plugin for VirtualJoystickPlugin<S, B> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<VirtualJoystickInteractionArea>()
            .register_type::<VirtualJoystickNode<S, B>>()
            .register_type::<VirtualJoystickData>()
            .register_type::<AxisBoth>()
            .register_type::<AxisHoritonalOnly>()
            .register_type::<AxisVerticalOnly>()
            .register_type::<StickFixed>()
            .register_type::<StickFloating>()
            .register_type::<StickDynamic>()
            .register_type::<VirtualJoystickEventType>()
            .add_event::<VirtualJoystickEvent<S,B>>()
            .add_event::<InputEvent>()
            .add_systems(PreUpdate, update_joystick.before(update_input::<S,B>))
            .add_systems(
                PreUpdate,
                update_joystick_by_mouse.before(update_input::<S,B>),
            )
            .add_systems(PreUpdate, update_input::<S,B>)
            .add_systems(
                PostUpdate,
                joystick_image_node_system::<S,B>.before(UiSystem::Layout),
            );

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else { return; };
        render_app.add_systems(
            ExtractSchedule,
            extract_joystick_node::<S,B>.after(RenderUiSystem::ExtractNode),
        );
    }
}

fn joystick_image_node_system<S: VirtualJoystickID, B: Behaviour>(
    interaction_area: Query<(&Node, With<VirtualJoystickInteractionArea>)>,
    mut joystick: Query<(
        &Transform,
        &VirtualJoystickNode<S, B>,
        &mut VirtualJoystickData,
    )>,
) {
    let interaction_area = interaction_area
        .iter()
        .map(|(node, _)| node.size())
        .collect::<Vec<Vec2>>();

    for (i, (j_pos, data, mut knob)) in joystick.iter_mut().enumerate() {
        let j_pos = j_pos.translation.truncate();
        let Some(size) = interaction_area.get(i) else {
            return;
        };
        let interaction_area = Rect::from_center_size(j_pos, *size);
        knob.dead_zone = data.dead_zone;
        knob.interactable_zone_rect = interaction_area;
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
pub struct VirtualJoystickEvent<S: VirtualJoystickID, B: Behaviour> {
    id: S,
    event: VirtualJoystickEventType,
    value: Vec2,
    delta: Vec2,
    behavour: B
}

impl<S: VirtualJoystickID, B: Behaviour> VirtualJoystickEvent<S, B> {
    /// Get ID of joystick throw event
    pub fn id(&self) -> S {
        self.id.clone()
    }
    /// Raw position of point (Mouse or Touch)
    pub fn value(&self) -> Vec2 {
        self.value
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
        let mut pt = self.behavour.project_to_axis(self.delta);
        if pt.x > dead_zone {
            pt.x = 1.;
        } else if pt.x < -dead_zone {
            pt.x = -1.;
        } else {
            pt.x = 0.;
        }
        if pt.y > dead_zone {
            pt.y = 1.;
        } else if pt.y < -dead_zone {
            pt.y = -1.;
        } else {
            pt.y = 0.;
        }
        pt
    }
}
