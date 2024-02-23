use bevy::{
    ecs::{
        event::EventWriter,
        query::With,
        system::{Query, Res},
    },
    hierarchy::Children,
    input::{mouse::MouseButton, touch::Touches, Input},
    math::Vec2,
    transform::components::GlobalTransform,
    ui::{Node, PositionType, Style, Val},
    window::{PrimaryWindow, Window},
};

use crate::{
    components::{TouchState, VirtualJoystickUIBackground, VirtualJoystickUIKnob},
    JoystickDeadZone, JoystickDynamic, JoystickFixed, JoystickFloating, JoystickHorizontalOnly,
    JoystickVerticalOnly, VirtualJoystickEvent, VirtualJoystickEventType, VirtualJoystickID,
    VirtualJoystickNode,
};
use bevy::ecs::query::Without;

pub fn update_input<S: VirtualJoystickID>(
    mut joysticks: Query<(&Node, &GlobalTransform, &mut VirtualJoystickNode<S>)>,
    mouse_buttons: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    for (joystick_node, joystick_global_transform, mut joystick_state) in &mut joysticks {
        joystick_state.just_released = false;
        if let Some(touch_state) = &mut joystick_state.touch_state {
            touch_state.just_pressed = false;
        }
        if joystick_state.touch_state.is_none() {
            let rect = joystick_node.logical_rect(joystick_global_transform);
            for touch in touches.iter() {
                if rect.contains(touch.position()) {
                    joystick_state.touch_state = Some(TouchState {
                        id: touch.id(),
                        is_mouse: false,
                        start: touch.position(),
                        current: touch.position(),
                        just_pressed: true,
                    });
                    break;
                }
            }
            if joystick_state.touch_state.is_none() && mouse_buttons.just_pressed(MouseButton::Left)
            {
                if let Some(mouse_pos) = q_windows.single().cursor_position() {
                    if rect.contains(mouse_pos) {
                        joystick_state.touch_state = Some(TouchState {
                            id: 0,
                            is_mouse: true,
                            start: mouse_pos,
                            current: mouse_pos,
                            just_pressed: true,
                        });
                    }
                }
            }
        } else {
            let mut clear_touch_state = false;
            if let Some(touch_state) = &joystick_state.touch_state {
                if touch_state.is_mouse {
                    if mouse_buttons.just_released(MouseButton::Left) {
                        clear_touch_state = true;
                    }
                } else if touches.just_released(touch_state.id) {
                    clear_touch_state = true;
                }
            }
            if clear_touch_state {
                joystick_state.touch_state = None;
                joystick_state.just_released = true;
            } else if let Some(touch_state) = &mut joystick_state.touch_state {
                if touch_state.is_mouse {
                    let new_current = q_windows.single().cursor_position().unwrap();
                    if new_current != touch_state.current {
                        touch_state.current = new_current;
                    }
                } else if let Some(touch) = touches.get_pressed(touch_state.id) {
                    let touch_position = touch.position();
                    if touch_position != touch_state.current {
                        touch_state.current = touch_position;
                    }
                }
            }
        }
    }
}

pub fn update_fixed<S: VirtualJoystickID>(
    mut joystick: Query<
        (&Node, &GlobalTransform, &mut VirtualJoystickNode<S>),
        With<JoystickFixed>,
    >,
) {
    for (joystick_node, joystick_global_transform, mut joystick_state) in &mut joystick {
        let joystick_rect = joystick_node.logical_rect(joystick_global_transform);
        joystick_state.base_offset = Vec2::ZERO;
        let new_delta: Vec2;
        if let Some(touch_state) = &joystick_state.touch_state {
            let mut new_delta2 = ((touch_state.current - touch_state.start)
                / joystick_rect.half_size())
            .clamp(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
            new_delta2.y = -new_delta2.y;
            new_delta = new_delta2;
        } else {
            new_delta = Vec2::ZERO;
        }
        joystick_state.delta = new_delta;
    }
}

pub fn update_floating<S: VirtualJoystickID>(
    mut joystick: Query<
        (&Node, &GlobalTransform, &mut VirtualJoystickNode<S>),
        With<JoystickFloating>,
    >,
) {
    for (joystick_node, joystick_global_transform, mut joystick_state) in &mut joystick {
        let joystick_rect = joystick_node.logical_rect(joystick_global_transform);
        let base_offset: Vec2;
        let mut assign_base_offset = false;
        if let Some(touch_state) = &joystick_state.touch_state {
            if touch_state.just_pressed {
                base_offset = touch_state.start - joystick_rect.center();
                assign_base_offset = true;
            } else {
                base_offset = joystick_state.base_offset;
            }
        } else if joystick_state.just_released {
            base_offset = Vec2::ZERO;
            assign_base_offset = true;
        } else {
            base_offset = joystick_state.base_offset;
        }
        if assign_base_offset {
            joystick_state.base_offset = base_offset;
        }
        let new_delta: Vec2;
        if let Some(touch_state) = &joystick_state.touch_state {
            let mut new_delta2 = ((touch_state.current - (joystick_rect.center() + base_offset))
                / joystick_rect.half_size())
            .clamp(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
            new_delta2.y = -new_delta2.y;
            new_delta = new_delta2;
        } else {
            new_delta = Vec2::ZERO;
        }
        joystick_state.delta = new_delta;
    }
}

pub fn update_dynamic<S: VirtualJoystickID>(
    mut joysticks: Query<
        (&Node, &GlobalTransform, &mut VirtualJoystickNode<S>),
        With<JoystickDynamic>,
    >,
) {
    for (joystick_node, joystick_global_transform, mut joystick_state) in &mut joysticks {
        let joystick_rect = joystick_node.logical_rect(joystick_global_transform);
        let joystick_rect_center = joystick_rect.center();
        let joystick_rect_half_size = joystick_rect.half_size();
        let base_offset: Vec2;
        let mut assign_base_offset = false;
        if let Some(touch_state) = &joystick_state.touch_state {
            if touch_state.just_pressed {
                base_offset = touch_state.start - joystick_rect.center();
                assign_base_offset = true;
            } else {
                base_offset = joystick_state.base_offset;
            }
        } else if joystick_state.just_released {
            base_offset = Vec2::ZERO;
            assign_base_offset = true;
        } else {
            base_offset = joystick_state.base_offset;
        }
        if assign_base_offset {
            joystick_state.base_offset = base_offset;
        }
        let new_delta: Vec2;
        let mut new_base_offset: Option<Vec2> = None;
        if let Some(touch_state) = &joystick_state.touch_state {
            let mut offset = touch_state.current - (joystick_rect_center + base_offset);
            if offset.length_squared() > joystick_rect_half_size.x * joystick_rect_half_size.x {
                let adjustment = offset - offset * (joystick_rect_half_size.x / offset.length());
                offset += adjustment;
                new_base_offset = Some(joystick_state.base_offset + adjustment);
            }
            let mut new_delta2 = (offset / joystick_rect.half_size())
                .clamp(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
            new_delta2.y = -new_delta2.y;
            new_delta = new_delta2;
        } else {
            new_delta = Vec2::ZERO;
        }
        joystick_state.delta = new_delta;
        if let Some(base_offset) = new_base_offset {
            joystick_state.base_offset = base_offset;
        }
    }
}

pub fn update_dead_zone<S: VirtualJoystickID>(
    mut joystick: Query<(&JoystickDeadZone, &mut VirtualJoystickNode<S>)>,
) {
    for (joystick_dead_zone, mut joystick_state) in &mut joystick {
        let dead_zone = joystick_dead_zone.0;
        if joystick_state.delta.x.abs() < dead_zone {
            joystick_state.delta.x = 0.0;
        }
        if joystick_state.delta.y.abs() < dead_zone {
            joystick_state.delta.y = 0.0;
        }
    }
}

pub fn update_horizontal_only<S: VirtualJoystickID>(
    mut joystick: Query<&mut VirtualJoystickNode<S>, With<JoystickHorizontalOnly>>,
) {
    for mut joystick_state in &mut joystick {
        joystick_state.delta.y = 0.0;
    }
}

pub fn update_vertical_only<S: VirtualJoystickID>(
    mut joystick: Query<&mut VirtualJoystickNode<S>, With<JoystickVerticalOnly>>,
) {
    for mut joystick_state in &mut joystick {
        joystick_state.delta.x = 0.0;
    }
}

pub fn update_fire_events<S: VirtualJoystickID>(
    joysticks: Query<&VirtualJoystickNode<S>>,
    mut send_values: EventWriter<VirtualJoystickEvent<S>>,
) {
    for joystick in &joysticks {
        if joystick.just_released {
            send_values.send(VirtualJoystickEvent {
                id: joystick.id.clone(),
                event: VirtualJoystickEventType::Up,
                value: Vec2::ZERO,
                delta: joystick.delta,
            });
            continue;
        }
        if let Some(touch_state) = &joystick.touch_state {
            if touch_state.just_pressed {
                send_values.send(VirtualJoystickEvent {
                    id: joystick.id.clone(),
                    event: VirtualJoystickEventType::Press,
                    value: touch_state.current,
                    delta: joystick.delta,
                });
            }
            send_values.send(VirtualJoystickEvent {
                id: joystick.id.clone(),
                event: VirtualJoystickEventType::Drag,
                value: touch_state.current,
                delta: joystick.delta,
            });
        }
    }
}

#[allow(clippy::complexity)]
pub fn update_ui<S: VirtualJoystickID>(
    joysticks: Query<(&Node, &VirtualJoystickNode<S>, &GlobalTransform, &Children)>,
    mut joystick_bases: Query<&mut Style, With<VirtualJoystickUIBackground>>,
    mut joystick_knobs: Query<
        (&mut Style, &Node, &GlobalTransform),
        (
            With<VirtualJoystickUIKnob>,
            Without<VirtualJoystickUIBackground>,
        ),
    >,
) {
    for (joystick_node, joystick_state, joystick_global_transform, children) in &joysticks {
        let joystick_rect = joystick_node.logical_rect(joystick_global_transform);
        let joystick_rect_half_size = joystick_rect.half_size();
        for child in children.iter() {
            if joystick_bases.contains(*child) {
                let mut joystick_base = joystick_bases.get_mut(*child).unwrap();
                joystick_base.position_type = PositionType::Absolute;
                joystick_base.left = Val::Px(joystick_state.base_offset.x);
                joystick_base.top = Val::Px(joystick_state.base_offset.y);
            }
        }
        for child in children.iter() {
            if joystick_knobs.contains(*child) {
                let (mut joystick_knob_style, joystick_knob_node, joystick_knob_global_transform) =
                    joystick_knobs.get_mut(*child).unwrap();
                let joystick_knob_rect =
                    joystick_knob_node.logical_rect(joystick_knob_global_transform);
                let joystick_knob_half_size = joystick_knob_rect.half_size();
                joystick_knob_style.position_type = PositionType::Absolute;
                joystick_knob_style.left = Val::Px(
                    joystick_state.base_offset.x
                        + joystick_rect_half_size.x
                        + joystick_knob_half_size.x
                        + (joystick_state.delta.x - 1.0) * joystick_rect_half_size.x,
                );
                joystick_knob_style.top = Val::Px(
                    joystick_state.base_offset.y
                        + joystick_rect_half_size.y
                        + joystick_knob_half_size.y
                        + (-joystick_state.delta.y - 1.0) * joystick_rect_half_size.y,
                );
            }
        }
    }
}
