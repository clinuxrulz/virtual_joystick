use bevy::{ecs::{component::Component, reflect::ReflectComponent}, prelude::Vec2, reflect::{std_traits::ReflectDefault, Reflect}};

#[derive(Component, Copy, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
pub struct VirtualJoystickUIKnob;

#[derive(Component, Copy, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
pub struct VirtualJoystickUIBackground;


#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickState {
    pub base_pos: Vec2,
    pub knob_pos: Vec2,
    pub delta: Vec2,
    pub touch_state: Option<TouchState>,
}

#[derive(Clone, Debug, Default, Reflect)]
#[reflect(Default)]
pub struct TouchState {
    pub id: u64,
    pub is_mouse: bool,
    pub start: Vec2,
    pub current: Vec2,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickDeadZone(pub f32);

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickHorizontalOnly;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickVerticalOnly;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickInvisible;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickFixed;


#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickFloating;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickDynamic;
