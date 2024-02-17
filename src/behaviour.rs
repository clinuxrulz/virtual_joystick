use bevy::prelude::*;

#[cfg(feature = "inspect")]
use bevy_inspector_egui::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait Behaviour: Clone + Default + FromReflect + TypePath + std::marker::Send + std::marker::Sync + 'static {
    fn project_to_axis(&self, pos: Vec2) -> Vec2 { pos }
    fn skip_reset_base_pos_on_no_drag(&self) -> bool { false }
    fn get_base_pos(&self, _uinode: &Node, _is_dragging: bool, _base_pos: Vec2, start_pos: Vec2, _global_transform: &GlobalTransform) -> Vec2 {
        return start_pos;
    }
    fn dragging(&self, pos: Vec2, half: Vec2, base_pos: &mut Vec2, start_pos: &mut Vec2, current_pos: Vec2) {}
}

impl<A: Behaviour + FromReflect + TypePath, B: Behaviour + FromReflect + TypePath> Behaviour for (A, B) {
    fn project_to_axis(&self, pos: Vec2) -> Vec2 {
        return self.1.project_to_axis(self.0.project_to_axis(pos));
    }
    fn skip_reset_base_pos_on_no_drag(&self) -> bool {
        return self.0.skip_reset_base_pos_on_no_drag() || self.1.skip_reset_base_pos_on_no_drag();
    }
    fn get_base_pos(&self, uinode: &Node, is_dragging: bool, base_pos: Vec2, start_pos: Vec2, global_transform: &GlobalTransform) -> Vec2 {
        return self.1.get_base_pos(
            uinode,
            is_dragging,
            base_pos,
            self.0.get_base_pos(uinode, is_dragging, base_pos, start_pos, global_transform),
            global_transform
        );
    }
    fn dragging(&self, pos: Vec2, half: Vec2, base_pos: &mut Vec2, start_pos: &mut Vec2, current_pos: Vec2) {
        self.0.dragging(pos, half, base_pos, start_pos, current_pos);
        self.1.dragging(pos, half, base_pos, start_pos, current_pos);
    }
}

impl<
    A: Behaviour + FromReflect + TypePath,
    B: Behaviour + FromReflect + TypePath,
    C: Behaviour + FromReflect + TypePath
> Behaviour for (A, B, C) {
    fn project_to_axis(&self, pos: Vec2) -> Vec2 {
        return ((self.0.clone(), self.1.clone()), self.2.clone()).project_to_axis(pos)
    }
    fn skip_reset_base_pos_on_no_drag(&self) -> bool {
        return ((self.0.clone(), self.1.clone()), self.2.clone()).skip_reset_base_pos_on_no_drag()
    }
    fn get_base_pos(&self, uinode: &Node, is_dragging: bool, base_pos: Vec2, start_pos: Vec2, global_transform: &GlobalTransform) -> Vec2 {
        return ((self.0.clone(), self.1.clone()), self.2.clone()).get_base_pos(uinode, is_dragging, base_pos, start_pos, global_transform)
    }
    fn dragging(&self, pos: Vec2, half: Vec2, base_pos: &mut Vec2, start_pos: &mut Vec2, current_pos: Vec2) {
        return ((self.0.clone(), self.1.clone()), self.2.clone()).dragging(pos, half, base_pos, start_pos, current_pos);
    }
}


#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct AxisBoth;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct AxisHoritonalOnly;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct AxisVerticalOnly;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct StickFixed;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct StickFloating;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct StickDynamic;

impl Behaviour for AxisBoth {}

impl Behaviour for AxisHoritonalOnly {
    fn project_to_axis(&self, pos: Vec2) -> Vec2 {
        Vec2::new(pos.x, 0.)
    }
}

impl Behaviour for AxisVerticalOnly {
    fn project_to_axis(&self, pos: Vec2) -> Vec2 {
        Vec2::new(0., pos.y)
    }
}

impl Behaviour for StickFixed {
    fn get_base_pos(&self, uinode: &Node, _is_dragging: bool, _base_pos: Vec2, _start_pos: Vec2, global_transform: &GlobalTransform) -> Vec2 {
        let container_rect = Rect {
            max: uinode.size(),
            ..default()
        };
        return global_transform
            .compute_matrix()
            .transform_point3((container_rect.center() - (uinode.size() / 2.)).extend(0.)).xy();
    }
}

impl Behaviour for StickFloating {
    fn get_base_pos(&self, uinode: &Node, is_dragging: bool, _base_pos: Vec2, start_pos: Vec2, global_transform: &GlobalTransform) -> Vec2 {
        let container_rect = Rect {
            max: uinode.size(),
            ..default()
        };
        if !is_dragging {
            global_transform
                .compute_matrix()
                .transform_point3((container_rect.center() - (uinode.size() / 2.)).extend(0.))
                .xy()
        } else {
            start_pos
        }
    }
}

impl Behaviour for StickDynamic {
    fn skip_reset_base_pos_on_no_drag(&self) -> bool {
        true
    }
    fn get_base_pos(&self, _uinode: &Node, _is_dragging: bool, base_pos: Vec2, _start_pos: Vec2, global_transform: &GlobalTransform) -> Vec2 {
        base_pos
    }
    fn dragging(&self, pos: Vec2, half: Vec2, base_pos: &mut Vec2, start_pos: &mut Vec2, current_pos: Vec2) {
        *base_pos = pos;
        let to_knob = current_pos - *start_pos;
        let distance_to_knob = to_knob.length();
        if distance_to_knob > half.x {
            let excess_distance = distance_to_knob - half.x;
            *start_pos += to_knob.normalize() * excess_distance;
        }
    }
}
