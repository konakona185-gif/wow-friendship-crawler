
use bevy::prelude::*;

use crate::interaction::{
    WorldInteractionSet,
    WorldObjectClicked,
};
use crate::player::{
    Character,
    CharacterCommand,
    CharacterInputSet,
    FellowshipRole,
};

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Target {
    pub entity: Option<Entity>,
}

#[derive(Component)]
pub struct TargetIndicator {
    pub enemy: Entity,
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
        app.add_systems(
            Update,
            (
                target_tab_system,
                target_click_system,
                player_cast_system,
                enemy_death_system,
                target_indicator_system,
            )
                .chain()
                .after(CharacterInputSet::Collect)
                .after(WorldInteractionSet),
        );
    }
}

fn target_tab_system(
    mut player_query: Query<
        (&Transform, &mut Target, &mut CharacterCommand),
        With<Character>,
    >,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (player_transform, mut player_target, mut command) in &mut player_query {
        if !std::mem::take(&mut command.select_target) {
            continue;
        }

        let nearest = enemy_query
            .iter()
            .map(|(entity, transform)| {
                let dist = player_transform.translation.distance(transform.translation);
                (entity, dist)
            })
            .min_by(|a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((enemy_entity, _)) = nearest {
            player_target.entity = Some(enemy_entity);
            info!("Target selezionato: {:?}", enemy_entity);
        }
    }
}

fn target_click_system(
    mut clicked: MessageReader<WorldObjectClicked>,
    enemy_query: Query<(), With<Enemy>>,
    mut player_query: Query<&mut Target, With<Character>>,
) {
    for click in clicked.read() {
        if click.button != MouseButton::Left {
            continue;
        }

        let Some(entity) = click.entity else {
            continue;
        };

        if enemy_query.get(entity).is_err() {
            continue;
        }

        for mut target in &mut player_query {
            target.entity = Some(entity);
            info!("Target selezionato tramite click: {:?}", entity);
        }
    }
}

fn player_cast_system(
    mut player_query: Query<
        (&FellowshipRole, &mut CombatStats, &Target, &mut CharacterCommand),
        With<Character>,
    >,
    mut enemy_query: Query<
        &mut CombatStats,
        (With<Enemy>, Without<Character>),
    >,
) {
    for (role, mut player_stats, target, mut command) in &mut player_query {
        if !std::mem::take(&mut command.ability_1) {
            continue;
        }

        let Some(target_entity) = target.entity else {
            info!("Nessun target selezionato! Premi TAB per agganciare il nemico.");
            continue;
        };

        let Ok(mut enemy_stats) = enemy_query.get_mut(target_entity) else {
            continue;
        };

        match role {
            FellowshipRole::Tank => {
                enemy_stats.current_health -= 15.0;
                player_stats.resource += 20.0;

                info!(
                    "Tank colpisce! Danno: 15. Vita Nemico: {}/{}. Rabbia generata: {}",
                    enemy_stats.current_health,
                    enemy_stats.max_health,
                    player_stats.resource
                );
            }
            FellowshipRole::Dps => {
                if player_stats.resource >= 30.0 {
                    player_stats.resource -= 30.0;
                    enemy_stats.current_health -= 40.0;

                    info!(
                        "DPS scatta l'abilità! Danno pesante: 40. Vita Nemico: {}/{}. Energia residua: {}",
                        enemy_stats.current_health,
                        enemy_stats.max_health,
                        player_stats.resource
                    );
                } else {
                    enemy_stats.current_health -= 10.0;
                    player_stats.resource += 15.0;

                    info!(
                        "DPS attacco base! Danno: 10. Vita Nemico: {}/{}. Energia rigenerata: {}",
                        enemy_stats.current_health,
                        enemy_stats.max_health,
                        player_stats.resource
                    );
                }
            }
            FellowshipRole::Healer => {
                if player_stats.resource >= 20.0 {
                    player_stats.resource -= 20.0;
                    enemy_stats.current_health -= 25.0;

                    info!(
                        "Healer lancia incantesimo! Danno: 25. Vita Nemico: {}/{}. Mana residuo: {}",
                        enemy_stats.current_health,
                        enemy_stats.max_health,
                        player_stats.resource
                    );
                } else {
                    info!("Mana insufficiente per curare/attaccare!");
                }
            }
        }
    }
}

fn enemy_death_system(
    mut commands: Commands,
    enemy_query: Query<(Entity, &CombatStats), With<Enemy>>,
    indicator_query: Query<(Entity, &TargetIndicator)>,
    mut player_target_query: Query<&mut Target, With<Character>>,
) {
    for (enemy_entity, stats) in &enemy_query {
        if stats.current_health <= 0.0 {
            info!(
                "Nemico {:?} sconfitto. Rimozione dal mondo ECS.",
                enemy_entity
            );

            commands.entity(enemy_entity).despawn();

            for (indicator_entity, indicator) in &indicator_query {
                if indicator.enemy == enemy_entity {
                    commands.entity(indicator_entity).despawn();
                }
            }

            for mut player_target in &mut player_target_query {
                if player_target.entity == Some(enemy_entity) {
                    player_target.entity = None;
                    info!("Target del giocatore resettato.");
                }
            }
        }
    }
}

fn target_indicator_system(
    target_query: Query<&Target, With<Character>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut indicator_query: Query<
        (&TargetIndicator, &mut Transform, &mut Visibility),
    >,
) {
    for (indicator, mut indicator_transform, mut visibility) in &mut indicator_query {
        let selected = target_query
            .iter()
            .any(|target| target.entity == Some(indicator.enemy));

        if !selected {
            *visibility = Visibility::Hidden;
            continue;
        }

        let Ok(enemy_transform) = enemy_query.get(indicator.enemy) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        indicator_transform.translation =
            enemy_transform.translation + Vec3::new(0.0, -0.47, 0.0);

        *visibility = Visibility::Visible;
    }
}
