use bevy::prelude::{Component, Entity};

use crate::content::{
    AuraDefinition,
    AuraStackingPolicy,
    AuraValidationError,
    ContentId,
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
                if let Some(existing) = self.find_mut(aura_id) {
                    existing.remaining_duration = duration_seconds;
                } else {
                    self.instances.push(ActiveAura {
                        aura: aura_id.clone(),
                        source,
                        stacks: applied_stacks,
                        remaining_duration: duration_seconds,
                    });
                }
            }
            AuraStackingPolicy::AddStacks => {
                if let Some(existing) = self.find_mut(aura_id) {
                    existing.stacks = existing
                        .stacks
                        .saturating_add(applied_stacks)
                        .min(definition.max_stacks);

                    existing.remaining_duration = duration_seconds;
                } else {
                    self.instances.push(ActiveAura {
                        aura: aura_id.clone(),
                        source,
                        stacks: applied_stacks,
                        remaining_duration: duration_seconds,
                    });
                }
            }
            AuraStackingPolicy::Replace => {
                if let Some(existing) = self.find_mut(aura_id) {
                    existing.source = source;
                    existing.stacks = applied_stacks;
                    existing.remaining_duration = duration_seconds;
                } else {
                    self.instances.push(ActiveAura {
                        aura: aura_id.clone(),
                        source,
                        stacks: applied_stacks,
                        remaining_duration: duration_seconds,
                    });
                }
            }
            AuraStackingPolicy::Independent => {
                self.instances.push(ActiveAura {
                    aura: aura_id.clone(),
                    source,
                    stacks: applied_stacks,
                    remaining_duration: duration_seconds,
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
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return Vec::new();
        }

        for instance in &mut self.instances {
            if let Some(remaining) = &mut instance.remaining_duration {
                *remaining -= delta_seconds;
            }
        }

        let mut expired = Vec::new();

        self.instances.retain(|instance| {
            if instance
                .remaining_duration
                .is_some_and(|remaining| remaining <= 0.0)
            {
                expired.push(ExpiredAura {
                    aura: instance.aura.clone(),
                    source: instance.source,
                    stacks: instance.stacks,
                });

                false
            } else {
                true
            }
        });

        expired
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
    ) -> Option<&mut ActiveAura> {
        self.instances
            .iter_mut()
            .find(|instance| instance.aura == *aura_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuraRuntimeError {
    InvalidDefinition(AuraValidationError),
    ZeroStacks,
    InvalidDuration,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::content::ContentMetadata;

    fn aura_definition(
        policy: AuraStackingPolicy,
        max_stacks: u32,
    ) -> AuraDefinition {
        AuraDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic:aura.test").unwrap(),
                pack_id: ContentId::new("pack:classic").unwrap(),
                name: "Test Aura".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            max_stacks,
            stacking_policy: policy,
            periodic_effects: Vec::new(),
        }
    }

    fn aura_id() -> ContentId {
        ContentId::new("classic:aura.test").unwrap()
    }

    fn source() -> Entity {
        Entity::from_bits(1)
    }

    fn other_source() -> Entity {
        Entity::from_bits(2)
    }

    #[test]
    fn first_application_creates_an_active_aura() {
        let definition = aura_definition(
            AuraStackingPolicy::AddStacks,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 2, Some(10.0))
            .unwrap();

        let aura = active.get(&aura_id()).unwrap();

        assert_eq!(aura.source(), Some(source()));
        assert_eq!(aura.stacks(), 2);
        assert_eq!(aura.remaining_duration(), Some(10.0));
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn add_stacks_increases_stacks_and_refreshes_duration() {
        let definition = aura_definition(
            AuraStackingPolicy::AddStacks,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 2, Some(10.0))
            .unwrap();

        active
            .apply(&definition, Some(other_source()), 2, Some(20.0))
            .unwrap();

        let aura = active.get(&aura_id()).unwrap();

        assert_eq!(aura.stacks(), 4);
        assert_eq!(aura.remaining_duration(), Some(20.0));
        assert_eq!(aura.source(), Some(source()));
    }

    #[test]
    fn add_stacks_respects_maximum_stacks() {
        let definition = aura_definition(
            AuraStackingPolicy::AddStacks,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 4, Some(10.0))
            .unwrap();

        active
            .apply(&definition, Some(source()), 4, Some(10.0))
            .unwrap();

        assert_eq!(active.stacks(&aura_id()), 5);
    }

    #[test]
    fn refresh_duration_preserves_existing_stacks() {
        let definition = aura_definition(
            AuraStackingPolicy::RefreshDuration,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 3, Some(10.0))
            .unwrap();

        active
            .apply(&definition, Some(other_source()), 1, Some(25.0))
            .unwrap();

        let aura = active.get(&aura_id()).unwrap();

        assert_eq!(aura.stacks(), 3);
        assert_eq!(aura.remaining_duration(), Some(25.0));
        assert_eq!(aura.source(), Some(source()));
    }

    #[test]
    fn replace_policy_overwrites_stacks_and_duration() {
        let definition = aura_definition(
            AuraStackingPolicy::Replace,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 4, Some(10.0))
            .unwrap();

        active
            .apply(&definition, Some(other_source()), 2, Some(20.0))
            .unwrap();

        let aura = active.get(&aura_id()).unwrap();

        assert_eq!(aura.stacks(), 2);
        assert_eq!(aura.remaining_duration(), Some(20.0));
        assert_eq!(aura.source(), Some(other_source()));
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn independent_policy_creates_separate_instances() {
        let definition = aura_definition(
            AuraStackingPolicy::Independent,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 2, Some(10.0))
            .unwrap();

        active
            .apply(&definition, Some(other_source()), 3, Some(20.0))
            .unwrap();

        assert_eq!(active.len(), 2);
        assert_eq!(active.stacks(&aura_id()), 5);

        let instances: Vec<&ActiveAura> = active.iter().collect();

        assert_eq!(instances[0].source(), Some(source()));
        assert_eq!(instances[0].remaining_duration(), Some(10.0));

        assert_eq!(instances[1].source(), Some(other_source()));
        assert_eq!(instances[1].remaining_duration(), Some(20.0));
    }

    #[test]
    fn partial_removal_removes_oldest_stacks_first() {
        let definition = aura_definition(
            AuraStackingPolicy::Independent,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 2, Some(10.0))
            .unwrap();

        active
            .apply(&definition, Some(other_source()), 3, Some(20.0))
            .unwrap();

        let removed = active.remove(&aura_id(), Some(3));

        assert_eq!(removed, 3);
        assert_eq!(active.len(), 1);
        assert_eq!(active.stacks(&aura_id()), 2);
        assert_eq!(
            active.get(&aura_id()).unwrap().source(),
            Some(other_source())
        );
    }

    #[test]
    fn full_removal_removes_all_matching_instances() {
        let definition = aura_definition(
            AuraStackingPolicy::Independent,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 2, Some(10.0))
            .unwrap();

        active
            .apply(&definition, Some(other_source()), 3, Some(20.0))
            .unwrap();

        let removed = active.remove(&aura_id(), None);

        assert_eq!(removed, 5);
        assert!(active.is_empty());
    }

    #[test]
    fn ticking_expires_timed_auras() {
        let definition = aura_definition(
            AuraStackingPolicy::AddStacks,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 2, Some(5.0))
            .unwrap();

        assert!(active.tick(2.0).is_empty());
        assert_eq!(
            active.get(&aura_id()).unwrap().remaining_duration(),
            Some(3.0)
        );

        let expired = active.tick(3.0);

        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].aura, aura_id());
        assert_eq!(expired[0].source, Some(source()));
        assert_eq!(expired[0].stacks, 2);
        assert!(active.is_empty());
    }

    #[test]
    fn permanent_auras_do_not_expire() {
        let definition = aura_definition(
            AuraStackingPolicy::AddStacks,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 1, None)
            .unwrap();

        assert!(active.tick(1000.0).is_empty());
        assert_eq!(active.stacks(&aura_id()), 1);
    }

    #[test]
    fn invalid_applications_do_not_mutate_active_auras() {
        let definition = aura_definition(
            AuraStackingPolicy::AddStacks,
            5,
        );

        let mut active = ActiveAuras::default();

        assert_eq!(
            active.apply(&definition, Some(source()), 0, Some(10.0)),
            Err(AuraRuntimeError::ZeroStacks)
        );

        assert_eq!(
            active.apply(&definition, Some(source()), 1, Some(0.0)),
            Err(AuraRuntimeError::InvalidDuration)
        );

        assert_eq!(
            active.apply(
                &definition,
                Some(source()),
                1,
                Some(f32::INFINITY)
            ),
            Err(AuraRuntimeError::InvalidDuration)
        );

        assert!(active.is_empty());
    }

    #[test]
    fn invalid_tick_deltas_do_not_change_durations() {
        let definition = aura_definition(
            AuraStackingPolicy::AddStacks,
            5,
        );

        let mut active = ActiveAuras::default();

        active
            .apply(&definition, Some(source()), 1, Some(10.0))
            .unwrap();

        assert!(active.tick(0.0).is_empty());
        assert!(active.tick(-1.0).is_empty());
        assert!(active.tick(f32::NAN).is_empty());

        assert_eq!(
            active.get(&aura_id()).unwrap().remaining_duration(),
            Some(10.0)
        );
    }
}
