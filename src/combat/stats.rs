use bevy::prelude::*;

#[derive(Component)]
pub struct CombatStats {
    pub current_health: f32,
    pub max_health: f32,
    pub resource: f32,
}
