use bevy::prelude::*;
use crate::player::Character;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Target {
    pub entity: Option<Entity>,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, target_tab_system);
    }
}

fn target_tab_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Character>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut target_query: Query<&mut Target, With<Character>>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        if let Ok(player_transform) = player_query.get_single() {
            if let Ok(mut player_target) = target_query.get_single_mut() {
                let mut nearest_enemy: Option<(Entity, f32)> = None;

                for (enemy_entity, enemy_transform) in &enemy_query {
                    let distance = player_transform.translation.distance(enemy_transform.translation);
                    
                    if let Some((_, nearest_distance)) = nearest_enemy {
                        if distance < nearest_distance {
                            nearest_enemy = Some((enemy_entity, distance));
                        }
                    } else {
                        nearest_enemy = Some((enemy_entity, distance));
                    }
                }

                if let Some((enemy_entity, _)) = nearest_enemy {
                    player_target.entity = Some(enemy_entity);
                    info!("Target selezionato: {:?}", enemy_entity);
                }
            }
        }
    }
}
