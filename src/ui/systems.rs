use bevy::{prelude::*, render::Extract, ui::ExtractedUiNodes};

use crate::{
    behaviour::Behaviour, VirtualJoystickID, VirtualJoystickNode, VirtualJoystickUIBackground, VirtualJoystickUIKnob
};

use super::VirtualJoystickData;

#[allow(clippy::type_complexity)]
pub fn extract_joystick_node<S: VirtualJoystickID, B: Behaviour>(
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    knob_ui_query: Extract<Query<(Entity, &Parent), With<VirtualJoystickUIKnob>>>,
    bg_ui_query: Extract<Query<(Entity, &Parent), With<VirtualJoystickUIBackground>>>,
    uinode_query: Extract<
        Query<(
            &Node,
            &GlobalTransform,
            &VirtualJoystickNode<S, B>,
            &Visibility,
            &InheritedVisibility,
            &ViewVisibility,
            &VirtualJoystickData,
        )>,
    >,
) {
    for (entity, parent) in &knob_ui_query {
        if let Ok((
            uinode,
            global_transform,
            joystick_node,
            visibility,
            inherited_visibility,
            view_visibility,
            data,
        )) = uinode_query.get(**parent)
        {
            if visibility == Visibility::Hidden
                || !inherited_visibility.get()
                || !view_visibility.get()
                || uinode.size().x == 0.
                || uinode.size().y == 0.
                || data.id_drag.is_none() && joystick_node.behaviour.skip_reset_base_pos_on_no_drag()
            {
                continue;
            }
            let base_pos = get_base_pos(uinode, joystick_node.behaviour.clone(), data, global_transform);
            let radius = uinode.size().x / 2.;
            // ui is y down, so we flip
            let pos = -data.delta * radius;
            let knob_pos = base_pos + joystick_node.behaviour.project_to_axis(pos).extend(0.);

            extracted_uinodes.uinodes.entry(entity).and_modify(|node| {
                node.transform = Mat4::from_translation(knob_pos);
            });
        }
    }

    for (entity, parent) in &bg_ui_query {
        if let Ok((
            uinode,
            global_transform,
            joystick_node,
            visibility,
            inherited_visibility,
            view_visibility,
            data,
        )) = uinode_query.get(**parent)
        {
            if visibility == Visibility::Hidden
                || !inherited_visibility.get()
                || !view_visibility.get()
                || uinode.size().x == 0.
                || uinode.size().y == 0.
                || data.id_drag.is_none() && joystick_node.behaviour.skip_reset_base_pos_on_no_drag()
            {
                continue;
            }
            let pos = get_base_pos(uinode, joystick_node.behaviour.clone(), data, global_transform);
            extracted_uinodes.uinodes.entry(entity).and_modify(|node| {
                node.transform = Mat4::from_translation(pos);
            });
        }
    }
}

fn get_base_pos<B: Behaviour>(
    uinode: &Node,
    behaviour: B,
    joystick: &VirtualJoystickData,
    global_transform: &GlobalTransform,
) -> Vec3 {
    let container_rect = Rect {
        max: uinode.size(),
        ..default()
    };
    behaviour
        .get_base_pos(
            uinode,
            joystick.id_drag.is_some(),
            joystick.base_pos,
            joystick.start_pos,
            global_transform
        )
        .extend(0.)
}
