use bevy::prelude::*;

mod movement;
mod movement_helpers;
mod targets;

pub struct EnemyNavigationPlugin;

impl Plugin for EnemyNavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                movement::spawn_enemy_navigation_agents,
                movement::move_enemy_agents,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            targets::sync_enemy_navigation_targets,
        );
    }
}
