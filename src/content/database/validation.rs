use super::{ContentDatabase, ContentId, TalentDefinition};
use super::super::{
    AbilityValidationError,
    AuraValidationError,
    CombatAttributeValidationError,
    ContentPackRegistryError,
    DamageSchoolMitigation,
    DamageSchoolValidationError,
    EffectKind,
    EffectMagnitude,
    EffectValidationError,
    ResourceValidationError,
    TalentValidationError,
};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

impl ContentDatabase {
    pub fn validate(&self) -> Result<(), ContentDatabaseError> {
        self.pack_load_order()?;

        self.validate_damage_school_references()?;
        self.validate_aura_definitions()?;
        self.validate_effect_references()?;
        self.validate_ability_references()?;
        self.validate_talent_references()?;
        self.validate_talent_prerequisite_cycles()?;

        Ok(())
    }

    fn validate_damage_school_references(
        &self,
    ) -> Result<(), ContentDatabaseError> {
        for (id, school) in &self.damage_schools {
            let components = match &school.mitigation {
                DamageSchoolMitigation::Composite { components } => {
                    components.as_slice()
                }
                mitigation => std::slice::from_ref(mitigation),
            };

            for component in components {
                if let DamageSchoolMitigation::Resistance { stat } = component {
                    if !self.combat_attributes.contains_key(stat) {
                        return Err(ContentDatabaseError::MissingReference {
                            owner: id.clone(),
                            reference: stat.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_aura_definitions(
        &self,
    ) -> Result<(), ContentDatabaseError> {
        for aura in self.auras.values() {
            aura.validate()
                .map_err(ContentDatabaseError::InvalidAura)?;
        }

        Ok(())
    }

    fn validate_effect_references(
        &self,
    ) -> Result<(), ContentDatabaseError> {
        for (id, effect) in &self.effects {
            match &effect.kind {
                EffectKind::Damage { amount, school } => {
                    if !self.damage_schools.contains_key(school) {
                        return Err(ContentDatabaseError::MissingReference {
                            owner: id.clone(),
                            reference: school.clone(),
                        });
                    }

                    self.validate_effect_magnitude_references(id, amount)?;
                }
                EffectKind::GainResource { resource, amount }
                | EffectKind::SpendResource { resource, amount } => {
                    if !self.resources.contains_key(resource) {
                        return Err(ContentDatabaseError::MissingReference {
                            owner: id.clone(),
                            reference: resource.clone(),
                        });
                    }

                    self.validate_effect_magnitude_references(id, amount)?;
                }
                EffectKind::Heal { amount }
                | EffectKind::ModifyThreat { amount } => {
                    self.validate_effect_magnitude_references(id, amount)?;
                }
                EffectKind::ApplyAura { aura, .. }
                | EffectKind::RemoveAura { aura, .. } => {
                    if !self.auras.contains_key(aura) {
                        return Err(ContentDatabaseError::MissingReference {
                            owner: id.clone(),
                            reference: aura.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn validate_effect_magnitude_references(
        &self,
        owner: &ContentId,
        magnitude: &EffectMagnitude,
    ) -> Result<(), ContentDatabaseError> {
        for scaling in &magnitude.scaling {
            if !self.combat_attributes.contains_key(&scaling.stat) {
                return Err(ContentDatabaseError::MissingReference {
                    owner: owner.clone(),
                    reference: scaling.stat.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_ability_references(
        &self,
    ) -> Result<(), ContentDatabaseError> {
        for (id, ability) in &self.abilities {
            for cost in &ability.costs {
                if !self.resources.contains_key(&cost.resource) {
                    return Err(ContentDatabaseError::MissingReference {
                        owner: id.clone(),
                        reference: cost.resource.clone(),
                    });
                }
            }

            for effect in &ability.effects {
                if !self.effects.contains_key(effect) {
                    return Err(ContentDatabaseError::MissingReference {
                        owner: id.clone(),
                        reference: effect.clone(),
                    });
                }
            }

            for prerequisite in &ability.prerequisites {
                if !self.abilities.contains_key(prerequisite) {
                    return Err(ContentDatabaseError::MissingReference {
                        owner: id.clone(),
                        reference: prerequisite.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_talent_references(
        &self,
    ) -> Result<(), ContentDatabaseError> {
        for (tree_id, tree) in &self.talent_trees {
            for talent_id in &tree.talents {
                let talent = self.talents.get(talent_id).ok_or_else(|| {
                    ContentDatabaseError::MissingReference {
                        owner: tree_id.clone(),
                        reference: talent_id.clone(),
                    }
                })?;

                if talent.tree != *tree_id {
                    return Err(ContentDatabaseError::TalentTreeMismatch {
                        talent: talent_id.clone(),
                        tree: tree_id.clone(),
                    });
                }
            }
        }

        for (talent_id, talent) in &self.talents {
            let tree = self.talent_trees.get(&talent.tree).ok_or_else(|| {
                ContentDatabaseError::MissingReference {
                    owner: talent_id.clone(),
                    reference: talent.tree.clone(),
                }
            })?;

            if !tree.talents.contains(talent_id) {
                return Err(ContentDatabaseError::TalentTreeMismatch {
                    talent: talent_id.clone(),
                    tree: talent.tree.clone(),
                });
            }

            for prerequisite in &talent.prerequisites {
                let required_talent = self
                    .talents
                    .get(&prerequisite.talent)
                    .ok_or_else(|| {
                        ContentDatabaseError::MissingReference {
                            owner: talent_id.clone(),
                            reference: prerequisite.talent.clone(),
                        }
                    })?;

                if prerequisite.required_rank > required_talent.max_rank() {
                    return Err(
                        ContentDatabaseError::PrerequisiteRankOutOfRange {
                            talent: talent_id.clone(),
                            prerequisite: prerequisite.talent.clone(),
                            required_rank: prerequisite.required_rank,
                            max_rank: required_talent.max_rank(),
                        },
                    );
                }
            }

            for rank in &talent.ranks {
                for effect in &rank.effects {
                    if !self.effects.contains_key(effect) {
                        return Err(ContentDatabaseError::MissingReference {
                            owner: talent_id.clone(),
                            reference: effect.clone(),
                        });
                    }
                }

                for ability in &rank.unlocks {
                    if !self.abilities.contains_key(ability) {
                        return Err(ContentDatabaseError::MissingReference {
                            owner: talent_id.clone(),
                            reference: ability.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_talent_prerequisite_cycles(
        &self,
    ) -> Result<(), ContentDatabaseError> {
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();

        for id in self.talents.keys() {
            visit_talent(
                &self.talents,
                id,
                &mut visiting,
                &mut visited,
            )?;
        }

        Ok(())
    }
}

fn visit_talent(
    talents: &HashMap<ContentId, TalentDefinition>,
    id: &ContentId,
    visiting: &mut HashSet<ContentId>,
    visited: &mut HashSet<ContentId>,
) -> Result<(), ContentDatabaseError> {
    if visited.contains(id) {
        return Ok(());
    }

    if !visiting.insert(id.clone()) {
        return Err(ContentDatabaseError::TalentPrerequisiteCycle(
            id.clone(),
        ));
    }

    let talent = talents
        .get(id)
        .ok_or_else(|| ContentDatabaseError::MissingReference {
            owner: id.clone(),
            reference: id.clone(),
        })?;

    for prerequisite in &talent.prerequisites {
        visit_talent(
            talents,
            &prerequisite.talent,
            visiting,
            visited,
        )?;
    }

    visiting.remove(id);
    visited.insert(id.clone());

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentDatabaseError {
    InvalidPackRegistry(ContentPackRegistryError),
    InvalidEffect(EffectValidationError),
    InvalidAura(AuraValidationError),
    InvalidDamageSchool(DamageSchoolValidationError),
    InvalidCombatAttribute(CombatAttributeValidationError),
    InvalidResource(ResourceValidationError),
    InvalidAbility(AbilityValidationError),
    InvalidTalentTree(TalentValidationError),
    InvalidTalent(TalentValidationError),
    UnknownPack(ContentId),
    DuplicateContent(ContentId),
    MissingReference {
        owner: ContentId,
        reference: ContentId,
    },
    TalentTreeMismatch {
        talent: ContentId,
        tree: ContentId,
    },
    PrerequisiteRankOutOfRange {
        talent: ContentId,
        prerequisite: ContentId,
        required_rank: u8,
        max_rank: u8,
    },
    TalentPrerequisiteCycle(ContentId),
}

impl Display for ContentDatabaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPackRegistry(error) => {
                write!(formatter, "Registro dei pacchetti non valido: {error}")
            }
            Self::InvalidEffect(error) => {
                write!(formatter, "Effetto non valido: {error}")
            }
            Self::InvalidAura(error) => {
                write!(formatter, "Aura non valida: {error}")
            }
            Self::InvalidDamageSchool(error) => {
                write!(formatter, "Scuola di danno non valida: {error}")
            }
            Self::InvalidCombatAttribute(error) => {
                write!(formatter, "Attributo di combattimento non valido: {error}")
            }
            Self::InvalidResource(error) => {
                write!(formatter, "Risorsa non valida: {error}")
            }
            Self::InvalidAbility(error) => {
                write!(formatter, "Abilità non valida: {error}")
            }
            Self::InvalidTalentTree(error) => {
                write!(formatter, "Albero dei talenti non valido: {error}")
            }
            Self::InvalidTalent(error) => {
                write!(formatter, "Talento non valido: {error}")
            }
            Self::UnknownPack(id) => {
                write!(formatter, "Pacchetto non registrato: {}", id.as_str())
            }
            Self::DuplicateContent(id) => {
                write!(formatter, "Contenuto duplicato: {}", id.as_str())
            }
            Self::MissingReference { owner, reference } => {
                write!(
                    formatter,
                    "Il contenuto {} fa riferimento a {} che non esiste",
                    owner.as_str(),
                    reference.as_str()
                )
            }
            Self::TalentTreeMismatch { talent, tree } => {
                write!(
                    formatter,
                    "Il talento {} non appartiene all'albero {}",
                    talent.as_str(),
                    tree.as_str()
                )
            }
            Self::PrerequisiteRankOutOfRange {
                talent,
                prerequisite,
                required_rank,
                max_rank,
            } => {
                write!(
                    formatter,
                    "Il talento {} richiede il grado {} di {}, ma il massimo è {}",
                    talent.as_str(),
                    required_rank,
                    prerequisite.as_str(),
                    max_rank
                )
            }
            Self::TalentPrerequisiteCycle(id) => {
                write!(
                    formatter,
                    "Rilevato un ciclo di prerequisiti che coinvolge {}",
                    id.as_str()
                )
            }
        }
    }
}

impl std::error::Error for ContentDatabaseError {}
