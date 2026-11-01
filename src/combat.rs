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

const DEFAULT_ENEMY_AGGRO_RANGE: f32 = 20.0;
const DEFAULT_ENEMY_LEASH_RANGE: f32 = 40.0;
const ENEMY_HOME_ARRIVAL_DISTANCE: f32 = 1.0;
const DEFAULT_ENEMY_MELEE_RANGE: f32 = 2.5;
const DEFAULT_ENEMY_ATTACK_DAMAGE: f32 = 5.0;
const DEFAULT_ENEMY_ATTACK_INTERVAL: f32 = 2.0;

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct EnemyAi {
    pub target: Option<Entity>,
    pub aggro_range: f32,
    pub leash_range: f32,
    pub returning_home: bool,
}

impl Default for EnemyAi {
    fn default() -> Self {
        Self {
            target: None,
            aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
            leash_range: DEFAULT_ENEMY_LEASH_RANGE,
            returning_home: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct EnemyHomePosition(pub Vec3);

#[derive(Component)]
struct EnemyAttackState {
    target: Entity,
    cooldown: Timer,
}

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
                enemy_aggro_system,
                enemy_attack_system,
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
    enemy_query: Query<
        (Entity, &Transform, Option<&CombatStats>),
        With<Enemy>,
    >,
) {
    for (player_transform, mut player_target, mut command) in &mut player_query {
        if !std::mem::take(&mut command.select_target) {
            continue;
        }

        let mut enemies: Vec<(Entity, f32)> = enemy_query
            .iter()
            .filter_map(|(entity, transform, stats)| {
                if stats.is_some_and(|stats| stats.current_health <= 0.0) {
                    return None;
                }

                let distance_squared = player_transform
                    .translation
                    .distance_squared(transform.translation);

                Some((entity, distance_squared))
            })
            .collect();

        enemies.sort_by(|a, b| a.1.total_cmp(&b.1));

        if enemies.is_empty() {
            player_target.entity = None;
            continue;
        }

        let next_index = player_target
            .entity
            .and_then(|current| {
                enemies.iter().position(|(entity, _)| *entity == current)
            })
            .map_or(0, |index| (index + 1) % enemies.len());

        let enemy_entity = enemies[next_index].0;

        player_target.entity = Some(enemy_entity);

        info!("Target selezionato: {:?}", enemy_entity);
    }
}

fn target_click_system(
    mut clicked: MessageReader<WorldObjectClicked>,
    enemy_query: Query<Option<&CombatStats>, With<Enemy>>,
    mut player_query: Query<&mut Target, With<Character>>,
) {
    for click in clicked.read() {
        if click.button != MouseButton::Left {
            continue;
        }

        let Some(entity) = click.entity else {
            continue;
        };

        let Ok(stats) = enemy_query.get(entity) else {
            continue;
        };

        if stats.is_some_and(|stats| stats.current_health <= 0.0) {
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
        (&mut CombatStats, Option<&EnemyAi>),
        (With<Enemy>, Without<Character>),
    >,
) {
    for (role, mut player_stats, target, mut command) in &mut player_query {
        if !std::mem::take(&mut command.ability_1) {
            continue;
        }

        if player_stats.current_health <= 0.0 {
            continue;
        }

        let Some(target_entity) = target.entity else {
            info!("Nessun target selezionato! Premi TAB per agganciare il nemico.");
            continue;
        };

        let Ok((mut enemy_stats, enemy_ai)) = enemy_query.get_mut(target_entity) else {
            continue;
        };

        if enemy_stats.current_health <= 0.0
            || enemy_ai.is_some_and(|ai| ai.returning_home)
        {
            continue;
        }

        match role {
            FellowshipRole::Tank => {
                enemy_stats.current_health =
                    (enemy_stats.current_health - 15.0).max(0.0);
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
                    enemy_stats.current_health =
                        (enemy_stats.current_health - 40.0).max(0.0);

                    info!(
                        "DPS scatta l'abilità! Danno pesante: 40. Vita Nemico: {}/{}. Energia residua: {}",
                        enemy_stats.current_health,
                        enemy_stats.max_health,
                        player_stats.resource
                    );
                } else {
                    enemy_stats.current_health =
                        (enemy_stats.current_health - 10.0).max(0.0);
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
                    enemy_stats.current_health =
                        (enemy_stats.current_health - 25.0).max(0.0);

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

fn enemy_aggro_system(
    player_query: Query<
        (Entity, &Transform, &CombatStats),
        (With<Character>, Without<Enemy>),
    >,
    mut enemy_query: Query<
        (
            &Transform,
            &mut CombatStats,
            &mut EnemyAi,
            Option<&EnemyHomePosition>,
        ),
        (With<Enemy>, Without<Character>),
    >,
) {
    for (enemy_transform, mut enemy_stats, mut enemy_ai, home_position) in
        &mut enemy_query
    {
        if enemy_stats.current_health <= 0.0 {
            enemy_ai.target = None;
            enemy_ai.returning_home = false;
            continue;
        }

        if let Some(home_position) = home_position {
            let distance_from_home_squared = enemy_transform
                .translation
                .distance_squared(home_position.0);

            let home_arrival_distance_squared =
                ENEMY_HOME_ARRIVAL_DISTANCE * ENEMY_HOME_ARRIVAL_DISTANCE;

            if enemy_ai.returning_home {
                if distance_from_home_squared > home_arrival_distance_squared {
                    enemy_ai.target = None;
                    continue;
                }

                enemy_ai.returning_home = false;

                info!("Nemico tornato alla posizione iniziale.");
            } else {
                let leash_range = if enemy_ai.leash_range.is_finite() {
                    enemy_ai.leash_range.max(0.0)
                } else {
                    DEFAULT_ENEMY_LEASH_RANGE
                };

                let leash_range_squared = leash_range * leash_range;

                if distance_from_home_squared > leash_range_squared {
                    enemy_ai.target = None;
                    enemy_ai.returning_home = true;
                    enemy_stats.current_health =
                        enemy_stats.max_health.max(0.0);

                    info!(
                        "Nemico fuori dal raggio di combattimento: ritorno alla posizione iniziale."
                    );

                    continue;
                }
            }
        }

        let current_target_is_alive = enemy_ai.target.is_some_and(|target_entity| {
            player_query
                .get(target_entity)
                .is_ok_and(|(_, _, stats)| stats.current_health > 0.0)
        });

        if current_target_is_alive {
            continue;
        }

        let aggro_range = if enemy_ai.aggro_range.is_finite() {
            enemy_ai.aggro_range.max(0.0)
        } else {
            DEFAULT_ENEMY_AGGRO_RANGE
        };

        let aggro_range_squared = aggro_range * aggro_range;

        let new_target = player_query
            .iter()
            .filter(|(_, _, stats)| stats.current_health > 0.0)
            .filter_map(|(entity, player_transform, _)| {
                let distance_squared = enemy_transform
                    .translation
                    .distance_squared(player_transform.translation);

                (distance_squared <= aggro_range_squared)
                    .then_some((entity, distance_squared))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(entity, _)| entity);

        enemy_ai.target = new_target;

        if let Some(target_entity) = new_target {
            info!(
                "Nemico ha acquisito il giocatore {:?} come bersaglio.",
                target_entity
            );
        }
    }
}

fn enemy_attack_system(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_query: Query<
        (
            Entity,
            &Transform,
            &EnemyAi,
            Option<&mut EnemyAttackState>,
        ),
        With<Enemy>,
    >,
    mut player_query: Query<
        (&Transform, &mut CombatStats),
        (With<Character>, Without<Enemy>),
    >,
) {
    let melee_range_squared =
        DEFAULT_ENEMY_MELEE_RANGE * DEFAULT_ENEMY_MELEE_RANGE;

    for (enemy_entity, enemy_transform, enemy_ai, attack_state) in
        &mut enemy_query
    {
        if enemy_ai.returning_home {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        }

        let Some(target_entity) = enemy_ai.target else {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        };

        let Ok((player_transform, mut player_stats)) =
            player_query.get_mut(target_entity)
        else {
            continue;
        };

        if player_stats.current_health <= 0.0 {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        }

        let Some(mut state) = attack_state else {
            commands.entity(enemy_entity).insert(EnemyAttackState {
                target: target_entity,
                cooldown: Timer::from_seconds(
                    DEFAULT_ENEMY_ATTACK_INTERVAL,
                    TimerMode::Repeating,
                ),
            });

            continue;
        };

        if state.target != target_entity {
            state.target = target_entity;
            state.cooldown.reset();
        }

        let distance_squared = enemy_transform
            .translation
            .distance_squared(player_transform.translation);

        if distance_squared > melee_range_squared {
            state.cooldown.reset();
            continue;
        }

        state.cooldown.tick(time.delta());

        if state.cooldown.just_finished() {
            player_stats.current_health =
                (player_stats.current_health - DEFAULT_ENEMY_ATTACK_DAMAGE)
                    .max(0.0);

            info!(
                "Nemico {:?} attacca il giocatore {:?}. Danno: {}. Vita giocatore: {}/{}",
                enemy_entity,
                target_entity,
                DEFAULT_ENEMY_ATTACK_DAMAGE,
                player_stats.current_health,
                player_stats.max_health
            );
        }
    }
}

fn target_indicator_system(
    target_query: Query<&Target, With<Character>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut indicator_query: Query<
        (&TargetIndicator, &mut Transform, &mut Visibility),
        Without<Enemy>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use benilla_world::interact::WorldObject;
    use benilla_world::model_render::ModelKind;
    use std::time::Duration;

    fn setup_target_test_app() -> App {
        let mut app = App::new();

        app.add_message::<WorldObjectClicked>();
        app.add_systems(
            Update,
            (target_tab_system, target_click_system).chain(),
        );

        app
    }

    fn setup_enemy_attack_test_app() -> App {
        let mut app = App::new();

        app.insert_resource(Time::default());
        app.add_systems(Update, enemy_attack_system);

        app
    }

    fn advance_test_time(app: &mut App, seconds: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(seconds));
    }

    fn test_enemy_attack_state(target: Entity) -> EnemyAttackState {
        EnemyAttackState {
            target,
            cooldown: Timer::from_seconds(
                DEFAULT_ENEMY_ATTACK_INTERVAL,
                TimerMode::Repeating,
            ),
        }
    }

    fn test_world_object() -> WorldObject {
        WorldObject {
            kind: ModelKind::Creature,
            label: "Test creature".to_string(),
            id: 1,
            detail: String::new(),
        }
    }

    fn spawn_living_player(
        world: &mut World,
        position: Vec3,
    ) -> Entity {
        world.spawn((
            Character,
            Transform::from_translation(position),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id()
    }

    fn spawn_enemy_ai(
        world: &mut World,
        position: Vec3,
    ) -> Entity {
        world.spawn((
            Enemy,
            EnemyAi::default(),
            EnemyHomePosition(position),
            Transform::from_translation(position),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id()
    }

    #[test]
    fn tab_selects_the_nearest_enemy() {
        let mut app = setup_target_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::from_translation(Vec3::ZERO),
            Target { entity: None },
            CharacterCommand {
                select_target: true,
                ..default()
            },
        )).id();

        app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 8.0),
        ));

        let nearest_enemy = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 3.0),
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            Some(nearest_enemy)
        );

        assert!(
            !app.world()
                .get::<CharacterCommand>(player)
                .unwrap()
                .select_target
        );
    }

    #[test]
    fn tab_does_not_select_defeated_enemies() {
        let mut app = setup_target_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand {
                select_target: true,
                ..default()
            },
        )).id();

        app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 1.0),
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
        ));

        let living_enemy = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 5.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            Some(living_enemy)
        );
    }

    #[test]
    fn tab_cycles_through_enemies_by_distance_and_wraps() {
        let mut app = setup_target_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand {
                select_target: true,
                ..default()
            },
        )).id();

        let nearest = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 3.0),
        )).id();

        let middle = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 8.0),
        )).id();

        let furthest = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 13.0),
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            Some(nearest)
        );

        for expected in [middle, furthest, nearest] {
            app.world_mut()
                .get_mut::<CharacterCommand>(player)
                .unwrap()
                .select_target = true;

            app.update();

            assert_eq!(
                app.world().get::<Target>(player).unwrap().entity,
                Some(expected)
            );
        }
    }

    #[test]
    fn tab_clears_target_when_no_enemies_exist() {
        let mut app = setup_target_test_app();

        let stale_target = app.world_mut().spawn_empty().id();

        let player = app.world_mut().spawn((
            Character,
            Transform::default(),
            Target {
                entity: Some(stale_target),
            },
            CharacterCommand {
                select_target: true,
                ..default()
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            None
        );
    }

    #[test]
    fn left_click_selects_the_exact_clicked_enemy() {
        let mut app = setup_target_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        )).id();

        app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 2.0),
        ));

        let clicked_enemy = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 12.0),
        )).id();

        app.world_mut().write_message(WorldObjectClicked {
            button: MouseButton::Left,
            entity: Some(clicked_enemy),
            object: test_world_object(),
            point: Vec3::ZERO,
            distance: 12.0,
        });

        app.update();

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            Some(clicked_enemy)
        );
    }

    #[test]
    fn clicking_a_defeated_enemy_does_not_change_target() {
        let mut app = setup_target_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        )).id();

        let defeated_enemy = app.world_mut().spawn((
            Enemy,
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
            Transform::from_xyz(0.0, 0.0, 2.0),
        )).id();

        app.world_mut().write_message(WorldObjectClicked {
            button: MouseButton::Left,
            entity: Some(defeated_enemy),
            object: test_world_object(),
            point: Vec3::ZERO,
            distance: 2.0,
        });

        app.update();

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            None
        );
    }

    #[test]
    fn right_click_and_clicking_non_enemies_do_not_change_target() {
        let mut app = setup_target_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        )).id();

        let enemy = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 2.0),
        )).id();

        let non_enemy = app.world_mut().spawn_empty().id();

        app.world_mut().write_message(WorldObjectClicked {
            button: MouseButton::Right,
            entity: Some(enemy),
            object: test_world_object(),
            point: Vec3::ZERO,
            distance: 2.0,
        });

        app.world_mut().write_message(WorldObjectClicked {
            button: MouseButton::Left,
            entity: Some(non_enemy),
            object: test_world_object(),
            point: Vec3::ZERO,
            distance: 1.0,
        });

        app.update();

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            None
        );
    }

    #[test]
    fn target_indicator_is_visible_only_for_the_selected_enemy() {
        let mut app = App::new();
        app.add_systems(Update, target_indicator_system);

        let selected_position = Vec3::new(4.0, 2.0, -3.0);
        let selected_enemy = app.world_mut().spawn((
            Enemy,
            Transform::from_translation(selected_position),
        )).id();

        let other_enemy = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(8.0, 0.0, 1.0),
        )).id();

        app.world_mut().spawn((
            Character,
            Target {
                entity: Some(selected_enemy),
            },
        ));

        let selected_indicator = app.world_mut().spawn((
            TargetIndicator {
                enemy: selected_enemy,
            },
            Transform::default(),
            Visibility::Hidden,
        )).id();

        let other_indicator = app.world_mut().spawn((
            TargetIndicator {
                enemy: other_enemy,
            },
            Transform::default(),
            Visibility::Hidden,
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<Visibility>(selected_indicator),
            Some(&Visibility::Visible)
        );

        assert_eq!(
            app.world().get::<Visibility>(other_indicator),
            Some(&Visibility::Hidden)
        );

        let expected_position =
            selected_position + Vec3::new(0.0, -0.47, 0.0);

        let actual_position = app
            .world()
            .get::<Transform>(selected_indicator)
            .unwrap()
            .translation;

        assert!((actual_position - expected_position).length() < 1.0e-6);
    }

    #[test]
    fn defeating_an_enemy_removes_its_indicator_and_clears_target() {
        let mut app = App::new();
        app.add_systems(Update, enemy_death_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let indicator = app.world_mut().spawn(TargetIndicator {
            enemy,
        }).id();

        let player = app.world_mut().spawn((
            Character,
            Target {
                entity: Some(enemy),
            },
        )).id();

        app.update();

        assert!(app.world().get_entity(enemy).is_err());
        assert!(app.world().get_entity(indicator).is_err());

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            None
        );
    }

    #[test]
    fn defeated_targets_cannot_receive_additional_player_casts() {
        let mut app = App::new();
        app.add_systems(Update, player_cast_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = app.world_mut().spawn((
            Character,
            FellowshipRole::Dps,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 100.0,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(enemy).unwrap().current_health,
            0.0
        );

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().resource,
            100.0
        );
    }

    #[test]
    fn returning_enemies_are_immune_to_player_abilities() {
        let mut app = App::new();
        app.add_systems(Update, player_cast_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: None,
                aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
                leash_range: DEFAULT_ENEMY_LEASH_RANGE,
                returning_home: true,
            },
            EnemyHomePosition(Vec3::ZERO),
            Transform::from_xyz(41.0, 0.0, 0.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = app.world_mut().spawn((
            Character,
            FellowshipRole::Dps,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 100.0,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(enemy).unwrap().current_health,
            100.0
        );

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().resource,
            100.0
        );
    }

    #[test]
    fn enemy_attacks_a_living_player_after_the_melee_cooldown() {
        let mut app = setup_enemy_attack_test_app();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 1.0),
        );

        app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: Some(player),
                ..EnemyAi::default()
            },
            EnemyAttackState {
                target: player,
                cooldown: Timer::from_seconds(
                    DEFAULT_ENEMY_ATTACK_INTERVAL,
                    TimerMode::Repeating,
                ),
            },
            Transform::default(),
        ));

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
        );
    }

    #[test]
    fn enemy_does_not_attack_outside_melee_range() {
        let mut app = setup_enemy_attack_test_app();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, DEFAULT_ENEMY_MELEE_RANGE + 1.0),
        );

        app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: Some(player),
                ..EnemyAi::default()
            },
            test_enemy_attack_state(player),
            Transform::default(),
        ));

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0
        );

        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation = Vec3::new(0.0, 0.0, 1.0);

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL - 0.1);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0
        );

        advance_test_time(&mut app, 0.1);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
        );
    }

    #[test]
    fn returning_enemies_do_not_attack_players() {
        let mut app = setup_enemy_attack_test_app();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 1.0),
        );

        app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: Some(player),
                returning_home: true,
                ..EnemyAi::default()
            },
            test_enemy_attack_state(player),
            Transform::default(),
        ));

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0
        );
    }

    #[test]
    fn defeated_players_cannot_be_damaged_by_enemy_attacks() {
        let mut app = setup_enemy_attack_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::from_xyz(0.0, 0.0, 1.0),
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: Some(player),
                ..EnemyAi::default()
            },
            test_enemy_attack_state(player),
            Transform::default(),
        ));

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            0.0
        );
    }

    #[test]
    fn defeated_players_cannot_cast_abilities() {
        let mut app = App::new();
        app.add_systems(Update, player_cast_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = app.world_mut().spawn((
            Character,
            FellowshipRole::Tank,
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(enemy).unwrap().current_health,
            100.0
        );

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().resource,
            0.0
        );
    }

    #[test]
    fn player_damage_never_reduces_enemy_health_below_zero() {
        let mut app = App::new();
        app.add_systems(Update, player_cast_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            CombatStats {
                current_health: 5.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = app.world_mut().spawn((
            Character,
            FellowshipRole::Tank,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(enemy).unwrap().current_health,
            0.0
        );

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().resource,
            20.0
        );
    }

    fn setup_enemy_aggro_test_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, enemy_aggro_system);
        app
    }

    #[test]
    fn enemy_acquires_the_nearest_living_player_in_range() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

        let farther_player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 15.0),
        );

        let nearest_player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 8.0),
        );

        let dead_player = app.world_mut().spawn((
            Character,
            Transform::from_xyz(0.0, 0.0, 2.0),
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            Some(nearest_player)
        );

        assert_ne!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            Some(farther_player)
        );

        assert_ne!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            Some(dead_player)
        );
    }

    #[test]
    fn enemy_does_not_acquire_a_player_outside_aggro_range() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

        spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, DEFAULT_ENEMY_AGGRO_RANGE + 1.0),
        );

        app.update();

        assert_eq!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            None
        );
    }

    #[test]
    fn enemy_keeps_its_current_target_while_that_player_is_alive() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

        let first_player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 12.0),
        );

        app.update();

        assert_eq!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            Some(first_player)
        );

        let closer_player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 2.0),
        );

        app.update();

        assert_eq!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            Some(first_player)
        );

        assert_ne!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            Some(closer_player)
        );
    }

    #[test]
    fn enemy_reacquires_a_new_player_when_its_target_dies() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

        let first_player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 8.0),
        );

        app.update();

        assert_eq!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            Some(first_player)
        );

        app.world_mut()
            .get_mut::<CombatStats>(first_player)
            .unwrap()
            .current_health = 0.0;

        let next_player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 12.0),
        );

        app.update();

        assert_eq!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            Some(next_player)
        );
    }

    #[test]
    fn defeated_enemies_do_not_acquire_or_keep_targets() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = app.world_mut().spawn((
            Enemy,
            EnemyAi::default(),
            Transform::default(),
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 2.0),
        );

        app.world_mut()
            .get_mut::<EnemyAi>(enemy)
            .unwrap()
            .target = Some(player);

        app.update();

        assert_eq!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            None
        );

        assert!(!app.world().get::<EnemyAi>(enemy).unwrap().returning_home);
    }

    #[test]
    fn non_finite_aggro_range_uses_the_default_range() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: None,
                aggro_range: f32::NAN,
                leash_range: DEFAULT_ENEMY_LEASH_RANGE,
                returning_home: false,
            },
            Transform::default(),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 10.0),
        );

        app.update();

        assert_eq!(
            app.world().get::<EnemyAi>(enemy).unwrap().target,
            Some(player)
        );
    }

    #[test]
    fn enemy_starts_returning_home_and_recovers_health_after_exceeding_leash() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: None,
                aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
                leash_range: DEFAULT_ENEMY_LEASH_RANGE,
                returning_home: false,
            },
            EnemyHomePosition(Vec3::ZERO),
            Transform::from_xyz(41.0, 0.0, 0.0),
            CombatStats {
                current_health: 25.0,
                max_health: 100.0,
                resource: 30.0,
            },
        )).id();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(41.0, 0.0, 1.0),
        );

        app.world_mut()
            .get_mut::<EnemyAi>(enemy)
            .unwrap()
            .target = Some(player);

        app.update();

        let ai = app.world().get::<EnemyAi>(enemy).unwrap();
        let stats = app.world().get::<CombatStats>(enemy).unwrap();

        assert_eq!(ai.target, None);
        assert!(ai.returning_home);
        assert_eq!(stats.current_health, stats.max_health);
    }

    #[test]
    fn returning_enemy_does_not_reacquire_players() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: None,
                aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
                leash_range: DEFAULT_ENEMY_LEASH_RANGE,
                returning_home: true,
            },
            EnemyHomePosition(Vec3::ZERO),
            Transform::from_xyz(10.0, 0.0, 0.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(10.0, 0.0, 1.0),
        );

        app.update();

        let ai = app.world().get::<EnemyAi>(enemy).unwrap();

        assert_eq!(ai.target, None);
        assert!(ai.returning_home);

        assert_ne!(ai.target, Some(player));
    }

    #[test]
    fn enemy_can_acquire_targets_again_after_returning_home() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: None,
                aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
                leash_range: DEFAULT_ENEMY_LEASH_RANGE,
                returning_home: true,
            },
            EnemyHomePosition(Vec3::ZERO),
            Transform::from_xyz(0.5, 0.0, 0.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 5.0),
        );

        app.update();

        let ai = app.world().get::<EnemyAi>(enemy).unwrap();

        assert!(!ai.returning_home);
        assert_eq!(ai.target, Some(player));
    }

    #[test]
    fn enemy_does_not_start_returning_at_the_exact_leash_distance() {
        let mut app = setup_enemy_aggro_test_app();

        let enemy = app.world_mut().spawn((
            Enemy,
            EnemyAi::default(),
            EnemyHomePosition(Vec3::ZERO),
            Transform::from_xyz(DEFAULT_ENEMY_LEASH_RANGE, 0.0, 0.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        app.update();

        let ai = app.world().get::<EnemyAi>(enemy).unwrap();

        assert!(!ai.returning_home);
    }
}
