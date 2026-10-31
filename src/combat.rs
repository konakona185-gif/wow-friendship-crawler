use bevy::prelude::*;
use crate::player::{Character, FellowshipRole};

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Target {
    pub entity: Option<Entity>,
}

#[derive(Component)]
pub struct CombatStats {
    pub current_health: f32,
    pub max_health: f32,
    pub resource: f32,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (target_tab_system, player_cast_system, enemy_death_system));
    }
}

fn target_tab_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Character>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut target_query: Query<&mut Target, With<Character>>,
) {
    if !keyboard_input.just_pressed(KeyCode::Tab) {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else { return; };
    let Ok(mut player_target) = target_query.get_single_mut() else { return; };
    
    let mut nearest_enemy: Option<(Entity, f32)> = None;

    for (enemy_entity, enemy_transform) in &enemy_query {
        let distance = player_transform.translation.distance(enemy_transform.translation);
        
        if let Some((_, nearest_distance)) = nearest_enemy {
            if distance ,
    mut player_query: Query<(&FellowshipRole, &mut CombatStats, &Target), With<Character>>,
    mut enemy_query: Query<&mut CombatStats, (With<Enemy>, Without<Character>)>,
) {
    if !keyboard_input.just_pressed(KeyCode::Digit1) {
        return;
    }

    let Ok((role, mut player_stats, target)) = player_query.get_single_mut() else { return; };
    let Some(target_entity) = target.entity else {
        info!("Nessun target selezionato! Premi TAB per agganciare il nemico.");
        return;
    };

    let Ok(mut enemy_stats) = enemy_query.get_component_mut::<CombatStats>(target_entity) else { return; };

    match role {
        FellowshipRole::Tank => {
            enemy_stats.current_health -= 15.0;
            player_stats.resource += 20.0;
            info!("Tank colpisce! Danno: 15. Vita Nemico: {}/{}. Rabbia generata: {}", enemy_stats.current_health, enemy_stats.max_health, player_stats.resource);
        }
        FellowshipRole::Dps => {
            if player_stats.resource >= 30.0 {
                player_stats.resource -= 30.0;
                enemy_stats.current_health -= 40.0;
                info!("DPS scatta l'abilità! Danno pesante: 40. Vita Nemico: {}/{}. Energia residua: {}", enemy_stats.current_health, enemy_stats.max_health, player_stats.resource);
            } else {
                enemy_stats.current_health -= 10.0;
                player_stats.resource += 15.0;
                info!("DPS attacco base! Danno: 10. Vita Nemico: {}/{}. Energia rigenerata: {}", enemy_stats.current_health, enemy_stats.max_health, player_stats.resource);
            }
        }
        FellowshipRole::Healer => {
            if player_stats.resource >= 20.0 {
                player_stats.resource -= 20.0;
                enemy_stats.current_health -= 25.0;
                info!("Healer lancia incantesimo! Danno: 25. Vita Nemico: {}/{}. Mana residuo: {}", enemy_stats.current_health, enemy_stats.max_health, player_stats.resource);
            } else {
                info!("Mana insufficiente per curare/attaccare!");
            }
        }
    }
}

fn enemy_death_system(
    mut commands: Commands,
    enemy_query: Query<(Entity, &CombatStats), With<Enemy>>,
    mut player_target_query: Query<&mut Target, With<Character>>,
) {
    for (enemy_entity, stats) in &enemy_query {
        if stats.current_health <= 0.0 {
            info!("Nemico {:?} sconfitto. Rimozione dal mondo ECS.", enemy_entity);
            commands.entity(enemy_entity).despawn_recursive();

            if let Ok(mut player_target) = player_target_query.get_single_mut() {
                if player_target.entity == Some(enemy_entity) {
                    player_target.entity = None;
                    info!("Target del giocatore resettato.");
                }
            }
        }
    }
}
