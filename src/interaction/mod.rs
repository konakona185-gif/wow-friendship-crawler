use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use benilla_world::interact::{PickBlob, PickBox, PickMesh, WorldObject, WorldPick};
use benilla_world::view::{CameraPoseSet, WorldCamera};

use crate::player::CharacterInputSet;

const POINTER_DRAG_THRESHOLD: f32 = 5.0;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldInteractionSet;

#[derive(Resource, Default)]
pub struct WorldHover {
    pub hit: Option<HoveredWorldObject>,
}

#[derive(Clone, Debug)]
pub struct HoveredWorldObject {
    pub entity: Option<Entity>,
    pub object: WorldObject,
    pub point: Vec3,
    pub distance: f32,
}

#[derive(Message, Clone, Debug)]
pub struct WorldObjectClicked {
    pub button: MouseButton,
    pub entity: Option<Entity>,
    pub object: WorldObject,
    pub point: Vec3,
    pub distance: f32,
}

#[derive(Default)]
struct PointerPressTracker {
    active: bool,
    travelled: f32,
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldHover>()
            .add_message::<WorldObjectClicked>()
            .add_systems(
                Update,
                world_pointer_system
                    .after(CharacterInputSet::Process)
                    .after(CameraPoseSet)
                    .in_set(WorldInteractionSet),
            );
    }
}

fn world_pointer_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<WorldCamera>>,
    pickable_query: Query<Entity, Or<(With<PickMesh>, With<PickBox>, With<PickBlob>)>>,
    world_pick: WorldPick,
    mut hover: ResMut<WorldHover>,
    mut clicked: MessageWriter<WorldObjectClicked>,
    mut left_press: Local<PointerPressTracker>,
    mut right_press: Local<PointerPressTracker>,
) {
    let frame_motion = if mouse_motion.delta.is_finite() {
        mouse_motion.delta.length()
    } else {
        0.0
    };

    let left_clicked = update_pointer_press(
        &mut left_press,
        &mouse_button,
        MouseButton::Left,
        frame_motion,
    );

    let right_clicked = update_pointer_press(
        &mut right_press,
        &mouse_button,
        MouseButton::Right,
        frame_motion,
    );

    hover.hit = None;

    if mouse_button.pressed(MouseButton::Left) || mouse_button.pressed(MouseButton::Right) {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let pickable: HashSet<Entity> = pickable_query.iter().collect();

    let Some(hit) = world_pick.at_cursor(cursor, camera, camera_transform, &pickable) else {
        return;
    };

    let Some(object) = hit.object else {
        return;
    };

    let hovered_object = HoveredWorldObject {
        entity: hit.entity,
        object,
        point: hit.hit.point,
        distance: hit.hit.distance,
    };

    if left_clicked || right_clicked {
        clicked.write(WorldObjectClicked {
            button: if right_clicked {
                MouseButton::Right
            } else {
                MouseButton::Left
            },
            entity: hovered_object.entity,
            object: hovered_object.object.clone(),
            point: hovered_object.point,
            distance: hovered_object.distance,
        });
    }

    hover.hit = Some(hovered_object);
}

fn update_pointer_press(
    tracker: &mut PointerPressTracker,
    input: &ButtonInput<MouseButton>,
    button: MouseButton,
    frame_motion: f32,
) -> bool {
    if input.just_pressed(button) {
        tracker.active = true;
        tracker.travelled = 0.0;
    }

    if tracker.active && (input.pressed(button) || input.just_released(button)) {
        tracker.travelled += frame_motion;
    }

    if input.just_released(button) {
        let clicked = tracker.active && tracker.travelled <= POINTER_DRAG_THRESHOLD;

        *tracker = PointerPressTracker::default();

        return clicked;
    }

    if !input.pressed(button) && !input.just_pressed(button) {
        *tracker = PointerPressTracker::default();
    }

    false
}
