use super::{
    AbilityDefinition,
    AbilityValidationError,
    ContentId,
    ContentMetadata,
    ContentPackManifest,
    ContentPackRegistry,
    ContentPackRegistryError,
    EffectDefinition,
    EffectValidationError,
    TalentDefinition,
    TalentTreeDefinition,
    TalentValidationError,
};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default)]
pub struct ContentDatabase {
    packs: ContentPackRegistry,
    effects: HashMap<ContentId, EffectDefinition>,
    abilities: HashMap<ContentId, AbilityDefinition>,
    talent_trees: HashMap<ContentId, TalentTreeDefinition>,
    talents: HashMap<ContentId, TalentDefinition>,
}

impl ContentDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_pack(
        &mut self,
        manifest: ContentPackManifest,
    ) -> Result<(), ContentDatabaseError> {
        self.packs
            .register(manifest)
            .map_err(ContentDatabaseError::InvalidPackRegistry)
    }

    pub fn register_effect(
        &mut self,
        effect: EffectDefinition,
    ) -> Result<(), ContentDatabaseError> {
        effect
            .validate()
            .map_err(ContentDatabaseError::InvalidEffect)?;

        self.validate_new_content(&effect.metadata)?;

        let id = effect.metadata.id.clone();
        self.effects.insert(id, effect);

        Ok(())
    }

    pub fn register_ability(
        &mut self,
        ability: AbilityDefinition,
    ) -> Result<(), ContentDatabaseError> {
        ability
            .validate()
            .map_err(ContentDatabaseError::InvalidAbility)?;

        self.validate_new_content(&ability.metadata)?;

        let id = ability.metadata.id.clone();
        self.abilities.insert(id, ability);

        Ok(())
    }

    pub fn register_talent_tree(
        &mut self,
        tree: TalentTreeDefinition,
    ) -> Result<(), ContentDatabaseError> {
        tree.validate()
            .map_err(ContentDatabaseError::InvalidTalentTree)?;

        self.validate_new_content(&tree.metadata)?;

        let id = tree.metadata.id.clone();
        self.talent_trees.insert(id, tree);

        Ok(())
    }

    pub fn register_talent(
        &mut self,
        talent: TalentDefinition,
    ) -> Result<(), ContentDatabaseError> {
        talent
            .validate()
            .map_err(ContentDatabaseError::InvalidTalent)?;

        self.validate_new_content(&talent.metadata)?;

        let id = talent.metadata.id.clone();
        self.talents.insert(id, talent);

        Ok(())
    }

    fn validate_new_content(
        &self,
        metadata: &ContentMetadata,
    ) -> Result<(), ContentDatabaseError> {
        if self.packs.get(&metadata.pack_id).is_none() {
            return Err(ContentDatabaseError::UnknownPack(
                metadata.pack_id.clone(),
            ));
        }

        let id = &metadata.id;

        if self.effects.contains_key(id)
            || self.abilities.contains_key(id)
            || self.talent_trees.contains_key(id)
            || self.talents.contains_key(id)
        {
            return Err(ContentDatabaseError::DuplicateContent(id.clone()));
        }

        Ok(())
    }

    pub fn effect(&self, id: &ContentId) -> Option<&EffectDefinition> {
        self.effects.get(id)
    }

    pub fn ability(&self, id: &ContentId) -> Option<&AbilityDefinition> {
        self.abilities.get(id)
    }

    pub fn talent_tree(
        &self,
        id: &ContentId,
    ) -> Option<&TalentTreeDefinition> {
        self.talent_trees.get(id)
    }

    pub fn talent(&self, id: &ContentId) -> Option<&TalentDefinition> {
        self.talents.get(id)
    }

    pub fn packs(&self) -> &ContentPackRegistry {
        &self.packs
    }

    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }

    pub fn ability_count(&self) -> usize {
        self.abilities.len()
    }

    pub fn talent_tree_count(&self) -> usize {
        self.talent_trees.len()
    }

    pub fn talent_count(&self) -> usize {
        self.talents.len()
    }

    pub fn content_count(&self) -> usize {
        self.effect_count()
            + self.ability_count()
            + self.talent_tree_count()
            + self.talent_count()
    }

    pub fn pack_load_order(
        &self,
    ) -> Result<Vec<&ContentPackManifest>, ContentDatabaseError> {
        self.packs
            .load_order()
            .map_err(ContentDatabaseError::InvalidPackRegistry)
    }

    pub fn validate(&self) -> Result<(), ContentDatabaseError> {
        self.pack_load_order()?;

        self.validate_ability_references()?;
        self.validate_talent_references()?;
        self.validate_talent_prerequisite_cycles()?;

        Ok(())
    }

    fn validate_ability_references(
        &self,
    ) -> Result<(), ContentDatabaseError> {
        for (id, ability) in &self.abilities {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{
        AbilityActivation,
        AbilityRange,
        AbilityTargeting,
        EffectKind,
        EffectMagnitude,
        EffectTarget,
        EffectTrigger,
        TalentPosition,
        TalentRankDefinition,
    };

    fn manifest(id: &str, dependencies: &[&str]) -> ContentPackManifest {
        ContentPackManifest {
            id: ContentId::new(id).unwrap(),
            name: id.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: dependencies
                .iter()
                .map(|dependency| ContentId::new(*dependency).unwrap())
                .collect(),
        }
    }

    fn metadata(id: &str, name: &str) -> ContentMetadata {
        ContentMetadata {
            id: ContentId::new(id).unwrap(),
            pack_id: ContentId::new("pack:classic").unwrap(),
            name: name.to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        }
    }

    fn damage_effect() -> EffectDefinition {
        EffectDefinition {
            metadata: metadata("classic:effect.strike", "Strike"),
            trigger: EffectTrigger::OnCast,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::Damage {
                amount: EffectMagnitude {
                    base: 100.0,
                    scaling: Vec::new(),
                },
                school: ContentId::new("classic:school.physical").unwrap(),
            },
        }
    }

    fn ability() -> AbilityDefinition {
        AbilityDefinition {
            metadata: metadata("classic:ability.strike", "Strike"),
            activation: AbilityActivation::Instant,
            targeting: AbilityTargeting::HostileUnit,
            range: AbilityRange {
                min_range: 0.0,
                max_range: 5.0,
            },
            cooldown_seconds: 3.0,
            costs: Vec::new(),
            effects: vec![
                ContentId::new("classic:effect.strike").unwrap(),
            ],
            class_restrictions: vec![
                ContentId::new("classic:class.warrior").unwrap(),
            ],
            prerequisites: Vec::new(),
        }
    }

    fn talent_tree(talent_ids: Vec<ContentId>) -> TalentTreeDefinition {
        TalentTreeDefinition {
            metadata: metadata("classic:tree.arms", "Arms"),
            class: ContentId::new("classic:class.warrior").unwrap(),
            display_order: 0,
            talents: talent_ids,
        }
    }

    fn talent(
        id: &str,
        prerequisites: Vec<super::super::TalentPrerequisite>,
    ) -> TalentDefinition {
        TalentDefinition {
            metadata: metadata(id, id),
            tree: ContentId::new("classic:tree.arms").unwrap(),
            position: TalentPosition {
                row: 0,
                column: 0,
            },
            required_tree_points: 0,
            points_per_rank: 1,
            prerequisites,
            ranks: vec![
                TalentRankDefinition {
                    rank: 1,
                    effects: vec![
                        ContentId::new("classic:effect.strike").unwrap(),
                    ],
                    unlocks: vec![
                        ContentId::new("classic:ability.strike").unwrap(),
                    ],
                },
            ],
        }
    }

    fn database_with_classic_pack() -> ContentDatabase {
        let mut database = ContentDatabase::new();
        database.register_pack(manifest("pack:classic", &[])).unwrap();
        database
    }

    #[test]
    fn database_registers_and_retrieves_content() {
        let mut database = database_with_classic_pack();
        let effect = damage_effect();
        let id = effect.metadata.id.clone();

        database.register_effect(effect).unwrap();

        assert_eq!(database.effect_count(), 1);
        assert_eq!(database.content_count(), 1);
        assert!(database.effect(&id).is_some());
    }

    #[test]
    fn database_rejects_content_from_unknown_packs() {
        let mut database = ContentDatabase::new();

        assert_eq!(
            database.register_effect(damage_effect()),
            Err(ContentDatabaseError::UnknownPack(
                ContentId::new("pack:classic").unwrap()
            ))
        );
    }

    #[test]
    fn database_rejects_duplicate_ids_across_content_types() {
        let mut database = database_with_classic_pack();
        let mut ability = ability();
        ability.metadata.id =
            ContentId::new("classic:effect.strike").unwrap();

        database.register_effect(damage_effect()).unwrap();

        assert_eq!(
            database.register_ability(ability),
            Err(ContentDatabaseError::DuplicateContent(
                ContentId::new("classic:effect.strike").unwrap()
            ))
        );
    }

    #[test]
    fn database_validates_ability_effect_references() {
        let mut database = database_with_classic_pack();
        database.register_ability(ability()).unwrap();

        assert_eq!(
            database.validate(),
            Err(ContentDatabaseError::MissingReference {
                owner: ContentId::new("classic:ability.strike").unwrap(),
                reference: ContentId::new("classic:effect.strike").unwrap(),
            })
        );

        database.register_effect(damage_effect()).unwrap();

        assert!(database.validate().is_ok());
    }

    #[test]
    fn database_validates_complete_talent_definitions() {
        let mut database = database_with_classic_pack();

        database.register_effect(damage_effect()).unwrap();
        database.register_ability(ability()).unwrap();

        let talent_id = ContentId::new("classic:talent.test").unwrap();

        database
            .register_talent_tree(talent_tree(vec![talent_id.clone()]))
            .unwrap();

        database
            .register_talent(talent(
                "classic:talent.test",
                Vec::new(),
            ))
            .unwrap();

        assert!(database.validate().is_ok());
        assert_eq!(database.talent_tree_count(), 1);
        assert_eq!(database.talent_count(), 1);
        assert_eq!(database.ability_count(), 1);
    }

    #[test]
    fn database_rejects_talent_tree_mismatches() {
        let mut database = database_with_classic_pack();

        let talent_id = ContentId::new("classic:talent.test").unwrap();

        database
            .register_talent_tree(talent_tree(Vec::new()))
            .unwrap();

        database
            .register_talent(talent(
                "classic:talent.test",
                Vec::new(),
            ))
            .unwrap();

        assert_eq!(
            database.validate(),
            Err(ContentDatabaseError::TalentTreeMismatch {
                talent: talent_id,
                tree: ContentId::new("classic:tree.arms").unwrap(),
            })
        );
    }

    #[test]
    fn database_rejects_talent_prerequisite_cycles() {
        let mut database = database_with_classic_pack();

        database.register_effect(damage_effect()).unwrap();
        database.register_ability(ability()).unwrap();

        let first_id = ContentId::new("classic:talent.first").unwrap();
        let second_id = ContentId::new("classic:talent.second").unwrap();

        database
            .register_talent_tree(talent_tree(vec![
                first_id.clone(),
                second_id.clone(),
            ]))
            .unwrap();

        database
            .register_talent(talent(
                "classic:talent.first",
                vec![super::super::TalentPrerequisite {
                    talent: second_id.clone(),
                    required_rank: 1,
                }],
            ))
            .unwrap();

        database
            .register_talent(talent(
                "classic:talent.second",
                vec![super::super::TalentPrerequisite {
                    talent: first_id,
                    required_rank: 1,
                }],
            ))
            .unwrap();

        assert!(matches!(
            database.validate(),
            Err(ContentDatabaseError::TalentPrerequisiteCycle(_))
        ));
    }

    #[test]
    fn database_resolves_pack_dependencies() {
        let mut database = ContentDatabase::new();

        database
            .register_pack(manifest(
                "pack:expansion",
                &["pack:classic"],
            ))
            .unwrap();

        database
            .register_pack(manifest("pack:classic", &[]))
            .unwrap();

        let order = database.pack_load_order().unwrap();

        let ids: Vec<&str> = order
            .iter()
            .map(|pack| pack.id.as_str())
            .collect();

        assert_eq!(ids, vec!["pack:classic", "pack:expansion"]);
    }
}
