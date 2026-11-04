
mod refresh;

use bevy::prelude::{Component, Entity};

use crate::content::{
    AuraDefinition,
    AuraStackingPolicy,
    AuraStackingScope,
    AuraValidationError,
    ContentId,
    PeriodicEffectStackScaling,
};

#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct ActiveAuras {
    instances: Vec<ActiveAura>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveAura {
    aura: ContentId,
    source: Option<Entity>,
    stacks: u32,
    remaining_duration: Option<f32>,
    periodic_effects: Vec<ActivePeriodicEffect>,
}

#[derive(Clone, Debug, PartialEq)]
struct ActivePeriodicEffect {
    effect: ContentId,
    interval_seconds: f64,
    elapsed_seconds: f64,
    stack_scaling: PeriodicEffectStackScaling,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PeriodicAuraTick {
    pub aura: ContentId,
    pub effect: ContentId,
    pub source: Option<Entity>,
    pub stacks: u32,
    pub stack_scaling: PeriodicEffectStackScaling,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuraAdvance {
    pub periodic_ticks: Vec<PeriodicAuraTick>,
    pub expired: Vec<ExpiredAura>,
}

impl ActiveAura {
    pub fn aura(&self) -> &ContentId {
        &self.aura
    }

    pub fn source(&self) -> Option<Entity> {
        self.source
    }

    pub fn stacks(&self) -> u32 {
        self.stacks
    }

    pub fn remaining_duration(&self) -> Option<f32> {
        self.remaining_duration
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExpiredAura {
    pub aura: ContentId,
    pub source: Option<Entity>,
    pub stacks: u32,
}

impl ActiveAuras {
    pub fn apply(
        &mut self,
        definition: &AuraDefinition,
        source: Option<Entity>,
        stacks: u32,
        duration_seconds: Option<f32>,
    ) -> Result<(), AuraRuntimeError> {
        definition
            .validate()
            .map_err(AuraRuntimeError::InvalidDefinition)?;

        if stacks == 0 {
            return Err(AuraRuntimeError::ZeroStacks);
        }

        if duration_seconds.is_some_and(
            |duration| !duration.is_finite() || duration <= 0.0,
        ) {
            return Err(AuraRuntimeError::InvalidDuration);
        }

        let applied_stacks = stacks.min(definition.max_stacks);
        let aura_id = &definition.metadata.id;

        match definition.stacking_policy {
            AuraStackingPolicy::RefreshDuration => {
                if let Some(existing) = self.find_mut(
                    aura_id,
                    source,
                    definition.stacking_scope,
                ) {
                    existing.remaining_duration =
                        refresh::refreshed_duration(
                            existing.remaining_duration,
                            duration_seconds,
                            definition.refresh_policy,
                        )?;
                } else {
                    self.instances.push(ActiveAura {
                        aura: aura_id.clone(),
                        source,
                        stacks: applied_stacks,
                        remaining_duration: duration_seconds,
                        periodic_effects: periodic_effect_states(definition),
                    });
                }
            }
            AuraStackingPolicy::AddStacks => {
                if let Some(existing) = self.find_mut(
                    aura_id,
                    source,
                    definition.stacking_scope,
                ) {
                    existing.stacks = existing
                        .stacks
                        .saturating_add(applied_stacks)
                        .min(definition.max_stacks);

                    existing.remaining_duration =
                        refresh::refreshed_duration(
                            existing.remaining_duration,
                            duration_seconds,
                            definition.refresh_policy,
                        )?;
                } else {
                    self.instances.push(ActiveAura {
                        aura: aura_id.clone(),
                        source,
                        stacks: applied_stacks,
                        remaining_duration: duration_seconds,
                        periodic_effects: periodic_effect_states(definition),
                    });
                }
            }
            AuraStackingPolicy::Replace => {
                if let Some(existing) = self.find_mut(
                    aura_id,
                    source,
                    definition.stacking_scope,
                ) {
                    let refreshed_duration =
                        refresh::refreshed_duration(
                            existing.remaining_duration,
                            duration_seconds,
                            definition.refresh_policy,
                        )?;

                    existing.source = source;
                    existing.stacks = applied_stacks;
                    existing.remaining_duration = refreshed_duration;
                    existing.periodic_effects =
                        periodic_effect_states(definition);
                } else {
                    self.instances.push(ActiveAura {
                        aura: aura_id.clone(),
                        source,
                        stacks: applied_stacks,
                        remaining_duration: duration_seconds,
                        periodic_effects: periodic_effect_states(definition),
                    });
                }
            }
            AuraStackingPolicy::Independent => {
                self.instances.push(ActiveAura {
                    aura: aura_id.clone(),
                    source,
                    stacks: applied_stacks,
                    remaining_duration: duration_seconds,
                    periodic_effects: periodic_effect_states(definition),
                });
            }
        }

        Ok(())
    }

    pub fn remove(
        &mut self,
        aura_id: &ContentId,
        stacks: Option<u32>,
    ) -> u32 {
        match stacks {
            None => {
                let mut removed = 0u32;

                self.instances.retain(|instance| {
                    if instance.aura == *aura_id {
                        removed = removed.saturating_add(instance.stacks);
                        false
                    } else {
                        true
                    }
                });

                removed
            }
            Some(0) => 0,
            Some(requested) => {
                let mut remaining = requested;
                let mut removed = 0u32;

                for instance in &mut self.instances {
                    if instance.aura != *aura_id || remaining == 0 {
                        continue;
                    }

                    let amount = instance.stacks.min(remaining);

                    instance.stacks -= amount;
                    remaining -= amount;
                    removed = removed.saturating_add(amount);
                }

                self.instances.retain(|instance| instance.stacks > 0);

                removed
            }
        }
    }

    pub fn tick(
        &mut self,
        delta_seconds: f32,
    ) -> Vec<ExpiredAura> {
        self.advance(delta_seconds).expired
    }

    pub fn advance(
        &mut self,
        delta_seconds: f32,
    ) -> AuraAdvance {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return AuraAdvance::default();
        }

        let mut result = AuraAdvance::default();

        for instance in &mut self.instances {
            let active_delta = instance
                .remaining_duration
                .map(|remaining| delta_seconds.min(remaining.max(0.0)))
                .unwrap_or(delta_seconds) as f64;

            for periodic_effect in &mut instance.periodic_effects {
                periodic_effect.elapsed_seconds += active_delta;

                let tolerance =
                    periodic_effect.interval_seconds * 0.000001;

                while periodic_effect.elapsed_seconds + tolerance
                    >= periodic_effect.interval_seconds
                {
                    periodic_effect.elapsed_seconds = (
                        periodic_effect.elapsed_seconds
                            - periodic_effect.interval_seconds
                    )
                    .max(0.0);

                    result.periodic_ticks.push(PeriodicAuraTick {
                        aura: instance.aura.clone(),
                        effect: periodic_effect.effect.clone(),
                        source: instance.source,
                        stacks: instance.stacks,
                        stack_scaling: periodic_effect.stack_scaling,
                    });
                }
            }

            if let Some(remaining) = &mut instance.remaining_duration {
                *remaining -= delta_seconds;
            }
        }

        self.instances.retain(|instance| {
            if instance
                .remaining_duration
                .is_some_and(|remaining| remaining <= 0.0)
            {
                result.expired.push(ExpiredAura {
                    aura: instance.aura.clone(),
                    source: instance.source,
                    stacks: instance.stacks,
                });

                false
            } else {
                true
            }
        });

        result
    }

    pub fn get(
        &self,
        aura_id: &ContentId,
    ) -> Option<&ActiveAura> {
        self.instances
            .iter()
            .find(|instance| instance.aura == *aura_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ActiveAura> {
        self.instances.iter()
    }

    pub fn stacks(&self, aura_id: &ContentId) -> u32 {
        self.instances
            .iter()
            .filter(|instance| instance.aura == *aura_id)
            .fold(0u32, |total, instance| {
                total.saturating_add(instance.stacks)
            })
    }

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    fn find_mut(
        &mut self,
        aura_id: &ContentId,
        source: Option<Entity>,
        stacking_scope: AuraStackingScope,
    ) -> Option<&mut ActiveAura> {
        self.instances.iter_mut().find(|instance| {
            if instance.aura != *aura_id {
                return false;
            }

            match stacking_scope {
                AuraStackingScope::Shared => true,
                AuraStackingScope::PerCaster => {
                    instance.source == source
                }
            }
        })
    }
}

fn periodic_effect_states(
    definition: &AuraDefinition,
) -> Vec<ActivePeriodicEffect> {
    definition
        .periodic_effects
        .iter()
        .map(|effect| ActivePeriodicEffect {
            effect: effect.effect.clone(),
            interval_seconds: effect.interval_seconds as f64,
            elapsed_seconds: 0.0,
            stack_scaling: effect.stack_scaling,
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuraRuntimeError {
    InvalidDefinition(AuraValidationError),
    ZeroStacks,
    InvalidDuration,
}

#[cfg(test)]
#[path = "aura_runtime/tests.rs"]
mod tests;
