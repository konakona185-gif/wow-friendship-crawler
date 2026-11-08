mod custom;
mod dispatcher;
mod execution;
mod magnitude;
mod resolution;
mod runtime;
mod targeting;

pub use custom::{
    CustomEffectHandler,
    CustomEffectHandlers,
    CustomEffectRegistryError,
};
pub use execution::{
    execute_resolved_effect,
    EffectExecutionContext,
    EffectExecutionError,
    EffectExecutionResult,
};
pub use magnitude::{
    calculate_effect_magnitude,
    calculate_signed_effect_magnitude,
    EffectMagnitudeError,
};
pub use resolution::{
    resolve_effect,
    EffectResolutionError,
    EffectTargetSelection,
    ResolvedEffect,
    ResolvedEffectOperation,
};
pub use runtime::PendingCombatEffects;

pub(super) use dispatcher::dispatch_combat_events;
pub(super) use runtime::execute_pending_effects;
pub(super) use targeting::acquire_area_targets_from_world;
