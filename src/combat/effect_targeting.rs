use std::collections::HashSet;

use bevy::prelude::{
    Entity,
    Or,
    Query,
    Transform,
    Vec3,
    With,
};

use crate::content::EffectTarget;

use super::{
    effect_resolution::EffectTargetSelection,
    AllySummon,
    Character,
    CombatStats,
    Enemy,
    HealthPool,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CombatSide {
    Characters,
    Enemies,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AreaTargetCandidate {
    pub entity: Entity,
    pub position: Vec3,
    pub side: CombatSide,
    pub alive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AreaTargetingError {
    UnsupportedTarget,
    InvalidRadius,
    InvalidCenter,
    MissingSource,
    InvalidSourceSide,
}

pub(super) fn acquire_area_targets(
    effect_target: &EffectTarget,
    center: Vec3,
    source_side: CombatSide,
    candidates: impl IntoIterator<Item = AreaTargetCandidate>,
) -> Result<EffectTargetSelection, AreaTargetingError> {
    let (radius, relation) = match effect_target {
        EffectTarget::AlliesInRadius { radius } => {
            (*radius, TargetRelation::Allies)
        }
        EffectTarget::EnemiesInRadius { radius } => {
            (*radius, TargetRelation::Enemies)
        }
        EffectTarget::UnitsInRadius { radius } => {
            (*radius, TargetRelation::Any)
        }
        _ => {
            return Err(AreaTargetingError::UnsupportedTarget);
        }
    };

    if !radius.is_finite() || radius <= 0.0 {
        return Err(AreaTargetingError::InvalidRadius);
    }

    if !center.x.is_finite()
        || !center.y.is_finite()
        || !center.z.is_finite()
    {
        return Err(AreaTargetingError::InvalidCenter);
    }

    let radius_squared = (radius as f64) * (radius as f64);
    let mut unique_targets = HashSet::new();
    let mut targets = Vec::new();

    for candidate in candidates {
        if !candidate.alive {
            continue;
        }

        if !candidate.position.x.is_finite()
            || !candidate.position.y.is_finite()
            || !candidate.position.z.is_finite()
        {
            continue;
        }

        let relation_matches = match relation {
            TargetRelation::Allies => candidate.side == source_side,
            TargetRelation::Enemies => candidate.side != source_side,
            TargetRelation::Any => true,
        };

        if !relation_matches {
            continue;
        }

        let dx = candidate.position.x as f64 - center.x as f64;
        let dy = candidate.position.y as f64 - center.y as f64;
        let dz = candidate.position.z as f64 - center.z as f64;

        let distance_squared = dx * dx + dy * dy + dz * dz;

        if distance_squared > radius_squared {
            continue;
        }

        if unique_targets.insert(candidate.entity) {
            targets.push(candidate.entity);
        }
    }

    targets.sort_by_key(|entity| entity.to_bits());

    Ok(EffectTargetSelection::Area(targets))
}

pub(super) fn acquire_area_targets_from_world(
    effect_target: &EffectTarget,
    source: Entity,
    combatants: &Query<(
        Entity,
        &Transform,
        Option<&CombatStats>,
        Option<&HealthPool>,
        Option<&Character>,
        Option<&Enemy>,
        Option<&AllySummon>,
    ), Or<(With<Character>, With<Enemy>, With<AllySummon>)>>,
) -> Result<EffectTargetSelection, AreaTargetingError> {
    let (
        _,
        source_transform,
        _,
        _,
        source_character,
        source_enemy,
        source_ally_summon,
    ) = combatants
        .get(source)
        .map_err(|_| AreaTargetingError::MissingSource)?;

    let source_side = combat_side(
        source_character,
        source_enemy,
        source_ally_summon,
    )
    .ok_or(AreaTargetingError::InvalidSourceSide)?;

    let candidates = combatants.iter().filter_map(
        |(
            entity,
            transform,
            stats,
            health_pool,
            character,
            enemy,
            ally_summon,
        )| {
            let side = combat_side(
                character,
                enemy,
                ally_summon,
            )?;

            let current_health = health_pool
                .map(|pool| pool.current())
                .or_else(|| {
                    stats.map(|stats| stats.current_health)
                });

            let alive = current_health.map_or(
                true,
                |health| health.is_finite() && health > 0.0,
            );

            Some(AreaTargetCandidate {
                entity,
                position: transform.translation,
                side,
                alive,
            })
        },
    );

    acquire_area_targets(
        effect_target,
        source_transform.translation,
        source_side,
        candidates,
    )
}

fn combat_side(
    character: Option<&Character>,
    enemy: Option<&Enemy>,
    ally_summon: Option<&AllySummon>,
) -> Option<CombatSide> {
    match (
        character.is_some(),
        enemy.is_some(),
        ally_summon.is_some(),
    ) {
        (true, false, false) | (false, false, true) => {
            Some(CombatSide::Characters)
        }
        (false, true, false) => Some(CombatSide::Enemies),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum TargetRelation {
    Allies,
    Enemies,
    Any,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        id: u64,
        position: Vec3,
        side: CombatSide,
        alive: bool,
    ) -> AreaTargetCandidate {
        AreaTargetCandidate {
            entity: Entity::from_bits(id),
            position,
            side,
            alive,
        }
    }

    fn characters() -> CombatSide {
        CombatSide::Characters
    }

    fn enemies() -> CombatSide {
        CombatSide::Enemies
    }

    #[test]
    fn allies_in_radius_select_only_matching_side() {
        let candidates = vec![
            candidate(1, Vec3::ZERO, characters(), true),
            candidate(2, Vec3::X, characters(), true),
            candidate(3, Vec3::X, enemies(), true),
        ];

        let result = acquire_area_targets(
            &EffectTarget::AlliesInRadius { radius: 5.0 },
            Vec3::ZERO,
            characters(),
            candidates,
        )
        .unwrap();

        assert_eq!(
            result,
            EffectTargetSelection::Area(vec![
                Entity::from_bits(1),
                Entity::from_bits(2),
            ])
        );
    }

    #[test]
    fn enemies_in_radius_select_opposing_side() {
        let candidates = vec![
            candidate(1, Vec3::ZERO, characters(), true),
            candidate(2, Vec3::X, enemies(), true),
            candidate(3, Vec3::Y, enemies(), true),
        ];

        let result = acquire_area_targets(
            &EffectTarget::EnemiesInRadius { radius: 5.0 },
            Vec3::ZERO,
            characters(),
            candidates,
        )
        .unwrap();

        assert_eq!(
            result,
            EffectTargetSelection::Area(vec![
                Entity::from_bits(2),
                Entity::from_bits(3),
            ])
        );
    }

    #[test]
    fn units_in_radius_select_both_sides() {
        let candidates = vec![
            candidate(1, Vec3::ZERO, characters(), true),
            candidate(2, Vec3::X, enemies(), true),
        ];

        let result = acquire_area_targets(
            &EffectTarget::UnitsInRadius { radius: 5.0 },
            Vec3::ZERO,
            characters(),
            candidates,
        )
        .unwrap();

        assert_eq!(
            result,
            EffectTargetSelection::Area(vec![
                Entity::from_bits(1),
                Entity::from_bits(2),
            ])
        );
    }

    #[test]
    fn area_targeting_includes_radius_boundary() {
        let candidates = vec![
            candidate(1, Vec3::new(5.0, 0.0, 0.0), enemies(), true),
            candidate(2, Vec3::new(5.1, 0.0, 0.0), enemies(), true),
        ];

        let result = acquire_area_targets(
            &EffectTarget::EnemiesInRadius { radius: 5.0 },
            Vec3::ZERO,
            characters(),
            candidates,
        )
        .unwrap();

        assert_eq!(
            result,
            EffectTargetSelection::Area(vec![
                Entity::from_bits(1),
            ])
        );
    }

    #[test]
    fn area_targeting_excludes_dead_units() {
        let candidates = vec![
            candidate(1, Vec3::ZERO, enemies(), true),
            candidate(2, Vec3::X, enemies(), false),
        ];

        let result = acquire_area_targets(
            &EffectTarget::EnemiesInRadius { radius: 5.0 },
            Vec3::ZERO,
            characters(),
            candidates,
        )
        .unwrap();

        assert_eq!(
            result,
            EffectTargetSelection::Area(vec![
                Entity::from_bits(1),
            ])
        );
    }

    #[test]
    fn area_targeting_rejects_invalid_radius() {
        assert_eq!(
            acquire_area_targets(
                &EffectTarget::EnemiesInRadius { radius: 0.0 },
                Vec3::ZERO,
                characters(),
                Vec::new(),
            ),
            Err(AreaTargetingError::InvalidRadius)
        );
    }

    #[test]
    fn area_targeting_rejects_invalid_center() {
        assert_eq!(
            acquire_area_targets(
                &EffectTarget::EnemiesInRadius { radius: 5.0 },
                Vec3::splat(f32::NAN),
                characters(),
                Vec::new(),
            ),
            Err(AreaTargetingError::InvalidCenter)
        );
    }

    #[test]
    fn area_targeting_rejects_non_area_effects() {
        assert_eq!(
            acquire_area_targets(
                &EffectTarget::Caster,
                Vec3::ZERO,
                characters(),
                Vec::new(),
            ),
            Err(AreaTargetingError::UnsupportedTarget)
        );
    }

    #[test]
    fn combat_side_requires_exactly_one_team_component() {
        let character = Character;
        let enemy = Enemy;
        let ally_summon = AllySummon;

        assert_eq!(
            combat_side(Some(&character), None, None),
            Some(CombatSide::Characters)
        );

        assert_eq!(
            combat_side(None, Some(&enemy), None),
            Some(CombatSide::Enemies)
        );

        assert_eq!(
            combat_side(None, None, Some(&ally_summon)),
            Some(CombatSide::Characters)
        );

        assert_eq!(combat_side(None, None, None), None);
        assert_eq!(
            combat_side(Some(&character), Some(&enemy), None),
            None
        );
        assert_eq!(
            combat_side(Some(&character), None, Some(&ally_summon)),
            None
        );
        assert_eq!(
            combat_side(None, Some(&enemy), Some(&ally_summon)),
            None
        );
    }
}
