
use super::*;
use benilla_world::interact::WorldObject;
use benilla_world::model_render::ModelKind;

fn setup_target_test_app() -> App {
    let mut app = App::new();

    app.add_message::<WorldObjectClicked>();
    app.add_systems(
        Update,
        (target_tab_system, target_click_system).chain(),
    );

    app
}

fn test_world_object() -> WorldObject {
    WorldObject {
        kind: ModelKind::Creature,
        label: "Test creature".to_string(),
        id: 1,
        detail: String::new(),
    }
}

mod click_tests;
mod tab_tests;
