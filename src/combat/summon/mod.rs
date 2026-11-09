use bevy::prelude::*;

use crate::content::{ContentDatabase, ContentId};
use crate::player::Character;

mod ai;
mod attacks;
mod lifecycle;

#[cfg(test)]
mod tests;

pub(super) use ai::ally_summon_aggro_system;
pub(super) use attacks::{ally_summon_attack_system, AllySummonAttackState};
pub(super) use lifecycle::summon_lifecycle_system;

use crate::combat::{
    CombatAttributes, CombatLevel, CombatStats, Enemy, EnemyAi, EnemyHomePosition, HealthPool,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonFaction {
    Allies,
    Enemies,
}

impl SummonFaction {
    pub fn opposing(self) -> Self {
        match self {
            Self::Allies => Self::Enemies,
            Self::Enemies => Self::Allies,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct SummonedCreature {
    pub owner: Entity,
    pub creature: ContentId,
    pub faction: SummonFaction,
    remaining_duration_seconds: Option<f32>,
}

impl SummonedCreature {
    pub fn new(
        owner: Entity,
        creature: ContentId,
        faction: SummonFaction,
        duration_seconds: Option<f32>,
    ) -> Self {
        Self {
            owner,
            creature,
            faction,
            remaining_duration_seconds: duration_seconds,
        }
    }

    pub fn remaining_duration(&self) -> Option<f32> {
        self.remaining_duration_seconds
    }

    pub(super) fn tick_lifetime(&mut self, delta: f32) -> bool {
        let Some(remaining) = self.remaining_duration_seconds.as_mut() else {
            return false;
        };

        if !delta.is_finite() || delta <= 0.0 {
            return false;
        }

        *remaining = (*remaining - delta).max(0.0);

        *remaining <= 0.0
    }
}

#[derive(Component)]
#[require(crate::combat::ActiveAuras, crate::combat::PendingDisplacement)]
pub struct AllySummon;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct AllySummonAi {
    pub target: Option<Entity>,
    pub aggro_range: f32,
}

impl AllySummonAi {
    pub fn new(aggro_range: f32) -> Self {
        Self {
            target: None,
            aggro_range,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::combat) struct PendingSummon {
    pub owner: Entity,
    pub creature: ContentId,
    pub spawn_position: Vec3,
    pub count: u32,
    pub duration_seconds: Option<f32>,
}

#[derive(Resource, Default)]
pub(in crate::combat) struct PendingSummons {
    pending: Vec<PendingSummon>,
}

impl PendingSummons {
    pub(super) fn enqueue(&mut self, summon: PendingSummon) {
        self.pending.push(summon);
    }

    pub(super) fn take_pending(&mut self) -> Vec<PendingSummon> {
        std::mem::take(&mut self.pending)
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct SummonCombatProfile {
    pub attack_damage: f32,
    pub attack_interval_seconds: f32,
    pub melee_range: f32,
    pub aggro_range: f32,
    pub leash_range: f32,
    pub movement_speed: f32,
}

pub(super) fn spawn_pending_summons(
    mut commands: Commands,
    mut pending: ResMut<PendingSummons>,
    database: Res<ContentDatabase>,
    owners: Query<(
        &CombatLevel,
        Has<Character>,
        Has<Enemy>,
        Option<&SummonedCreature>,
    )>,
) {
    let requests = pending.take_pending();

    for request in requests {
        if request.count == 0
            || !request.spawn_position.is_finite()
            || request
                .duration_seconds
                .is_some_and(|duration| !duration.is_finite() || duration <= 0.0)
        {
            warn!(
                owner = ?request.owner,
                creature = %request.creature.as_str(),
                count = request.count,
                duration_seconds = ?request.duration_seconds,
                spawn_position = ?request.spawn_position,
                "Impossibile creare l'evocazione: richiesta non valida."
            );
            continue;
        }

        let Ok((owner_level, is_character, is_enemy, parent_summon)) = owners.get(request.owner)
        else {
            warn!(
                owner = ?request.owner,
                creature = %request.creature.as_str(),
                "Impossibile creare l'evocazione: proprietario mancante o privo di CombatLevel."
            );
            continue;
        };

        let faction = if let Some(parent_summon) = parent_summon {
            parent_summon.faction
        } else if is_character && !is_enemy {
            SummonFaction::Allies
        } else if is_enemy && !is_character {
            SummonFaction::Enemies
        } else {
            warn!(
                owner = ?request.owner,
                creature = %request.creature.as_str(),
                "Impossibile determinare la fazione runtime del proprietario."
            );
            continue;
        };

        let Some(creature) = database.creature(&request.creature) else {
            warn!(
                owner = ?request.owner,
                creature = %request.creature.as_str(),
                "Impossibile creare l'evocazione: creatura non registrata."
            );
            continue;
        };

        let health_pool = match HealthPool::new(creature.health, creature.health) {
            Ok(pool) => pool,
            Err(error) => {
                warn!(
                    creature = %request.creature.as_str(),
                    %error,
                    "Impossibile inizializzare la salute dell'evocazione."
                );
                continue;
            }
        };

        let level = *owner_level;

        let combat_profile = SummonCombatProfile {
            attack_damage: creature.attack_damage,
            attack_interval_seconds: creature.attack_interval_seconds,
            melee_range: creature.melee_range,
            aggro_range: creature.aggro_range,
            leash_range: creature.leash_range,
            movement_speed: creature.movement_speed,
        };

        for _ in 0..request.count {
            let mut entity_commands = commands.spawn((
                SummonedCreature::new(
                    request.owner,
                    request.creature.clone(),
                    faction,
                    request.duration_seconds,
                ),
                combat_profile,
                CombatStats {
                    current_health: creature.health,
                    max_health: creature.health,
                    resource: 0.0,
                },
                health_pool,
                level,
                CombatAttributes::default(),
                Transform::from_translation(request.spawn_position),
            ));

            match faction {
                SummonFaction::Allies => {
                    entity_commands.insert((AllySummon, AllySummonAi::new(creature.aggro_range)));
                }
                SummonFaction::Enemies => {
                    entity_commands.insert((
                        Enemy,
                        EnemyAi {
                            target: None,
                            aggro_range: creature.aggro_range,
                            leash_range: creature.leash_range,
                            returning_home: false,
                        },
                        EnemyHomePosition(request.spawn_position),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod unit_tests;
