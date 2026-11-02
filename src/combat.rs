use std::collections::HashMap;

use bevy::prelude::*;

use crate::interaction::WorldInteractionSet;
use crate::player::{
    Character,
    CharacterCommand,
    CharacterInputSet,
    FellowshipRole,
};

mod damage;
mod death;
mod enemy_ai;
mod enemy_attacks;
mod events;
mod health;
mod healing;
mod player_attacks;
mod resources;
mod stats;
mod target_indicator;
mod target_selection;

pub use damage::{
    DamageMitigationError,
    DamageMitigationResult,
    DamageRequest,
    DamageRequestError,
    DamageResolution,
    DamageResolutionError,
};
pub use events::{
    CombatEventContext,
    CombatEventContextError,
    CombatEventOutcome,
};
pub use health::{
    HealthPool,
    HealthPoolError,
};
pub use healing::{
    HealingRequest,
    HealingRequestError,
    HealingResolution,
    HealingResolutionError,
};
pub use resources::{
    CombatResources,
    ResourceOperation,
    ResourcePool,
    ResourcePoolError,
    ResourceRequest,
    ResourceRequestError,
    ResourceResolution,
    ResourceResolutionError,
};
pub use stats::{
    CombatAttributeError,
    CombatAttributes,
    CombatStats,
};

const DEFAULT_ENEMY_AGGRO_RANGE: f32 = 20.0;
const DEFAULT_ENEMY_LEASH_RANGE: f32 = 40.0;
const ENEMY_HOME_ARRIVAL_DISTANCE: f32 = 1.0;
const DEFAULT_ENEMY_MELEE_RANGE: f32 = 2.5;
const DEFAULT_ENEMY_ATTACK_DAMAGE: f32 = 5.0;
const DEFAULT_ENEMY_ATTACK_INTERVAL: f32 = 2.0;
const DEFAULT_TANK_THREAT_MULTIPLIER: f32 = 3.0;

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

#[derive(Component, Default)]
pub struct EnemyThreat {
    values: HashMap<Entity, f32>,
}

impl EnemyThreat {
    fn add_threat(&mut self, player: Entity, amount: f32) {
        if !amount.is_finite() || amount <= 0.0 {
            return;
        }

        let value = self.values.entry(player).or_insert(0.0);
        *value = (*value + amount).min(f32::MAX);
    }

    fn clear(&mut self) {
        self.values.clear();
    }
}

#[derive(Component)]
pub struct Target {
    pub entity: Option<Entity>,
}

#[derive(Component)]
pub struct TargetIndicator {
    pub enemy: Entity,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                target_selection::target_tab_system,
                target_selection::target_click_system,
                player_attacks::player_cast_system,
                death::enemy_death_system,
                enemy_ai::enemy_aggro_system,
                enemy_attacks::enemy_attack_system,
                target_indicator::target_indicator_system,
            )
                .chain()
                .after(CharacterInputSet::Collect)
                .after(WorldInteractionSet),
        );
    }
}
