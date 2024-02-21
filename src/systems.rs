use bevy::{ecs::{query::With, system::{Query, Res}}, hierarchy::Children, input::{mouse::MouseButton, touch::Touches, Input}, math::Vec2, transform::components::GlobalTransform, ui::Node, window::{PrimaryWindow, Window}};

use crate::{components::{TouchState, VirtualJoystickUIBackground, VirtualJoystickUIKnob}, JoystickDeadZone, JoystickFixed, JoystickHorizontalOnly, JoystickState, JoystickVerticalOnly};

pub fn update_input(
    mut joysticks: Query<(&Node, &GlobalTransform, &mut JoystickState, &Children)>,
    mut joystick_bases: Query<(&Node, &VirtualJoystickUIBackground)>,
    mut joystick_knobs: Query<&VirtualJoystickUIKnob>,
    mouse_buttons: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    for (joystick_node, joystick_global_transform, mut joystick_state, joystick_children) in &mut joysticks {
        if joystick_state.touch_state.is_none() {
            let rect = joystick_node.logical_rect(joystick_global_transform);
            for touch in touches.iter() {
                if rect.contains(touch.position()) {
                    joystick_state.touch_state = Some(TouchState {
                        id: touch.id(),
                        is_mouse: false,
                        start: touch.position(),
                        current: touch.position(),
                    });
                    break;
                }
            }
            if joystick_state.touch_state.is_none() {
                if mouse_buttons.just_pressed(MouseButton::Left) {
                    if let Some(mouse_pos) = q_windows.single().cursor_position() {
                        if rect.contains(mouse_pos) {
                            joystick_state.touch_state = Some(TouchState {
                                id: 0,
                                is_mouse: true,
                                start: mouse_pos,
                                current: mouse_pos,
                            });
                        }
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
                } else {
                    if touches.just_released(touch_state.id) {
                        clear_touch_state = true;
                    }
                }
            }
            if clear_touch_state {
                joystick_state.touch_state = None;
            } else {
                if let Some(touch_state) = &mut joystick_state.touch_state {
                    if touch_state.is_mouse {
                        touch_state.current = q_windows.single().cursor_position();
                    } else {
                        if let Some(touch) = touches.get_pressed(touch_state.id) {
                            touch_state.current = touch.position();
                        }
                    }
                }
            }
        }
    }
}

pub fn update_dead_zone(mut joystick: Query<(&JoystickDeadZone, &mut JoystickState)>) {
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

pub fn update_horizontal_only(mut joystick: Query<&mut JoystickState, With<JoystickHorizontalOnly>>) {
    for mut joystick_state in &mut joystick {
        joystick_state.knob_pos = joystick_state.base_pos + Vec2::new(joystick_state.knob_pos.x - joystick_state.base_pos.x, 0.0);
        joystick_state.delta.y = 0.0;
    }
}

pub fn update_vertical_only(mut joystick: Query<&mut JoystickState, With<JoystickVerticalOnly>>) {
    for mut joystick_state in &mut joystick {
        joystick_state.knob_pos = joystick_state.base_pos + Vec2::new(0.0, joystick_state.knob_pos.y - joystick_state.base_pos.y);
        joystick_state.delta.x = 0.0;
    }
}

pub fn update_fixed(mut joystick: Query<&mut JoystickState, With<JoystickFixed>>) {

}