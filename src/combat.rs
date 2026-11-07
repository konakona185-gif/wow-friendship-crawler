use std::collections::HashMap;

use bevy::prelude::*;

use crate::interaction::WorldInteractionSet;
use crate::player::{
    Character,
    CharacterCommand,
    CharacterInputSet,
    FellowshipRole,
};

mod aura_execution;
mod aura_runtime;
mod aura_systems;
mod custom_effects;
mod damage;
mod death;
mod displacement;
mod effect_dispatcher;
mod effect_execution;
mod effect_magnitude;
mod effect_resolution;
mod effect_targeting;
mod effect_runtime;
mod enemy_ai;
mod enemy_attacks;
mod events;
mod health;
mod healing;
mod player_attacks;
mod resources;
mod stats;
mod summon_ai;
mod summon_attacks;
mod summon_lifecycle;
mod summons;
mod target_indicator;
mod target_selection;

pub use aura_runtime::{
    ActiveAura,
    ActiveAuras,
    AuraRuntimeError,
    ExpiredAura,
};
pub use custom_effects::{
    CustomEffectHandler,
    CustomEffectHandlers,
    CustomEffectRegistryError,
};
pub use displacement::{
    DisplacementError,
    PendingDisplacement,
};
pub use damage::{
    calculate_armor_mitigation,
    calculate_armor_mitigation_from_attributes,
    calculate_critical_strike,
    calculate_damage_mitigation,
    ArmorMitigationError,
    CriticalStrikeError,
    CriticalStrikeResult,
    DamageMitigationError,
    DamageMitigationResult,
    DamageRequest,
    DamageRequestError,
    DamageResolution,
    DamageResolutionError,
    DamageSchoolMitigationError,
};
pub use crate::content::EffectTrigger;
pub use effect_execution::{
    execute_resolved_effect,
    EffectExecutionContext,
    EffectExecutionError,
    EffectExecutionResult,
};
pub use effect_magnitude::{
    calculate_effect_magnitude,
    calculate_signed_effect_magnitude,
    EffectMagnitudeError,
};
pub use effect_resolution::{
    resolve_effect,
    EffectResolutionError,
    EffectTargetSelection,
    ResolvedEffect,
    ResolvedEffectOperation,
};
pub use effect_runtime::PendingCombatEffects;
pub use events::{
    CombatEffectSubscriptions,
    CombatEventContext,
    CombatEventContextError,
    CombatEventOutcome,
    CombatEventQueue,
    MAX_REACTIVE_EFFECT_GENERATIONS,
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
    ResourceApplicationError,
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
    CombatLevel,
    CombatLevelError,
    CombatStats,
};
pub use summons::{
    AllySummon,
    AllySummonAi,
    SummonFaction,
    SummonedCreature,
};
pub(crate) use summons::SummonCombatProfile;

const DEFAULT_ENEMY_AGGRO_RANGE: f32 = 20.0;
const DEFAULT_ENEMY_LEASH_RANGE: f32 = 40.0;
const ENEMY_HOME_ARRIVAL_DISTANCE: f32 = 1.0;
const DEFAULT_ENEMY_MELEE_RANGE: f32 = 2.5;
const DEFAULT_ENEMY_ATTACK_DAMAGE: f32 = 5.0;
const DEFAULT_ENEMY_ATTACK_INTERVAL: f32 = 2.0;
const DEFAULT_TANK_THREAT_MULTIPLIER: f32 = 3.0;

#[derive(Component)]
#[require(
    crate::combat::ActiveAuras,
    crate::combat::EnemyThreat,
    crate::combat::PendingDisplacement,
)]
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

        self.modify_threat(player, amount);
    }

    fn modify_threat(&mut self, player: Entity, amount: f32) {
        if !amount.is_finite() || amount == 0.0 {
            return;
        }

        let current = self
            .values
            .get(&player)
            .copied()
            .filter(|value| value.is_finite())
            .unwrap_or(0.0)
            .max(0.0);

        let updated = (current + amount).clamp(0.0, f32::MAX);

        if updated > 0.0 {
            self.values.insert(player, updated);
        } else {
            self.values.remove(&player);
        }
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
        app.init_resource::<PendingCombatEffects>()
            .init_resource::<CombatEventQueue>()
            .init_resource::<CustomEffectHandlers>()
            .init_resource::<summons::PendingSummons>()
            .add_systems(
                Update,
                (
                    target_selection::target_tab_system,
                    target_selection::target_click_system,
                    player_attacks::player_cast_system,
                    death::enemy_death_system,
                    summon_lifecycle::summon_lifecycle_system,
                    enemy_ai::enemy_aggro_system,
                    summon_ai::ally_summon_aggro_system,
                    enemy_attacks::enemy_attack_system,
                    summon_attacks::ally_summon_attack_system,
                    target_indicator::target_indicator_system,
                    aura_systems::tick_active_auras,
                    effect_runtime::execute_pending_effects,
                    summons::spawn_pending_summons,
                    effect_dispatcher::dispatch_combat_events,
                )
                    .chain()
                    .after(CharacterInputSet::Collect)
                    .after(WorldInteractionSet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threat_modification_adds_and_reduces_values() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let mut threat = EnemyThreat::default();

        threat.modify_threat(player, 100.0);

        assert_eq!(
            threat.values.get(&player),
            Some(&100.0)
        );

        threat.modify_threat(player, -35.0);

        assert_eq!(
            threat.values.get(&player),
            Some(&65.0)
        );
    }

    #[test]
    fn threat_modification_never_goes_below_zero() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let mut threat = EnemyThreat::default();

        threat.modify_threat(player, 25.0);
        threat.modify_threat(player, -50.0);

        assert!(
            !threat.values.contains_key(&player)
        );
    }

    #[test]
    fn threat_modification_ignores_invalid_values() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let mut threat = EnemyThreat::default();

        threat.modify_threat(player, 50.0);
        threat.modify_threat(player, f32::NAN);
        threat.modify_threat(player, f32::INFINITY);
        threat.modify_threat(player, 0.0);

        assert_eq!(
            threat.values.get(&player),
            Some(&50.0)
        );
    }

    #[test]
    fn threat_modification_clamps_overflow() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let mut threat = EnemyThreat::default();

        threat.modify_threat(player, f32::MAX);
        threat.modify_threat(player, f32::MAX);

        assert_eq!(
            threat.values.get(&player),
            Some(&f32::MAX)
        );
    }

    #[test]
    fn add_threat_preserves_positive_only_behavior() {
        let mut world = World::new();
        let player = world.spawn_empty().id();
        let mut threat = EnemyThreat::default();

        threat.add_threat(player, 20.0);
        threat.add_threat(player, -10.0);

        assert_eq!(
            threat.values.get(&player),
            Some(&20.0)
        );
    }

    #[test]
    fn enemy_requires_combat_runtime_components() {
        let mut world = World::new();

        let enemy = world.spawn(Enemy).id();

        assert!(world.get::<EnemyThreat>(enemy).is_some());
        assert!(world.get::<ActiveAuras>(enemy).is_some());
        assert!(world.get::<PendingDisplacement>(enemy).is_some());
    }

    #[test]
    fn combat_plugin_initializes_custom_effect_registry() {
        let mut app = App::new();

        app.add_plugins(CombatPlugin);

        assert!(
            app.world().contains_resource::<CustomEffectHandlers>()
        );

        assert!(
            app.world().contains_resource::<summons::PendingSummons>()
        );
    }
}
