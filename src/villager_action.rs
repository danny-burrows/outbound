use raylib::ease::{linear_in, Tween};
use raylib::math::Vector2;

const MOVE_SPEED: f32 = 0.5;

pub(crate) enum VillagerAction {
    VillagerMoveAction(VillagerMoveAction),
    VillagerBasicAction(VillagerBasicAction),
}

impl VillagerAction {
    fn update(&mut self, delta_time: f32) -> bool {
        match self {
            VillagerAction::VillagerMoveAction(a) => a.update(delta_time).is_some(),
            VillagerAction::VillagerBasicAction(a) => a.update(delta_time).is_some(),
        }
    }
}

pub(crate) struct VillagerMoveAction {
    tween_step: Tween,
    villager_position: Vector2,
    villager_step_path: Vec<(i64, i64)>,
    villager_step_index: usize,
}

impl VillagerMoveAction {
    pub(crate) fn new(villager_start: (f32, f32), villager_step_path: &[(i64, i64)]) -> Self {
        let tween_step = Tween::new(linear_in, 0.0, 1.0, MOVE_SPEED);

        Self {
            tween_step,
            villager_position: villager_start.into(),
            villager_step_path: villager_step_path.to_owned(),
            villager_step_index: 0,
        }
    }

    fn has_completed(&self) -> bool {
        self.villager_step_index >= self.villager_step_path.len() - 1
    }

    pub(crate) fn update(&mut self, delta_time: f32) -> Option<Vector2> {
        if self.has_completed() {
            return None;
        }

        let _change = self.tween_step.apply(delta_time);
        if self.tween_step.has_completed() {
            self.villager_step_index += 1;
            // let (tx, ty) = self.villager_step_path[self.villager_step_index];
            // self.villager_position = Vector2 {
            //     x: tx as f32,
            //     y: ty as f32,
            // };
            self.tween_step = Tween::new(linear_in, 0.0, 1.0, MOVE_SPEED);
        }

        let (tx, ty) = self.villager_step_path[self.villager_step_index];
        self.villager_position = self.villager_position.lerp(
            Vector2 {
                x: tx as f32,
                y: ty as f32,
            },
            self.tween_step.current_time(),
        );

        Some(self.villager_position)
    }
}

pub(crate) struct VillagerBasicAction {
    tween_step: Tween,
    action: Box<dyn FnMut()>,
}

impl VillagerBasicAction {
    pub(crate) fn new<'a>(action: Box<dyn FnMut()>) -> Self {
        let tween_step = Tween::new(linear_in, 0.0, 1.0, MOVE_SPEED);
        Self { tween_step, action }
    }

    pub(crate) fn update(&mut self, delta_time: f32) -> Option<()> {
        if self.tween_step.has_completed() {
            (self.action)();
            return None;
        }

        let _change = self.tween_step.apply(delta_time);
        Some(())
    }
}
