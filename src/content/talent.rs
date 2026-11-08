use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use super::{ContentId, ContentMetadata, ContentMetadataError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TalentTreeDefinition {
    pub metadata: ContentMetadata,
    pub class: ContentId,
    pub display_order: u16,
    pub talents: Vec<ContentId>,
}

impl TalentTreeDefinition {
    pub fn validate(&self) -> Result<(), TalentValidationError> {
        self.metadata
            .validate()
            .map_err(TalentValidationError::InvalidMetadata)?;

        if self.talents.is_empty() {
            return Err(TalentValidationError::EmptyTree);
        }

        let mut talents = HashSet::new();

        for talent in &self.talents {
            if !talents.insert(talent) {
                return Err(TalentValidationError::DuplicateTalentInTree);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TalentDefinition {
    pub metadata: ContentMetadata,
    pub tree: ContentId,
    pub position: TalentPosition,
    pub required_tree_points: u16,
    pub points_per_rank: u8,
    #[serde(default)]
    pub prerequisites: Vec<TalentPrerequisite>,
    pub ranks: Vec<TalentRankDefinition>,
}

impl TalentDefinition {
    pub fn validate(&self) -> Result<(), TalentValidationError> {
        self.metadata
            .validate()
            .map_err(TalentValidationError::InvalidMetadata)?;

        if self.points_per_rank == 0 {
            return Err(TalentValidationError::ZeroPointCost);
        }

        if self.ranks.is_empty() {
            return Err(TalentValidationError::MissingRanks);
        }

        if self.ranks.len() > u8::MAX as usize {
            return Err(TalentValidationError::InvalidRankSequence);
        }

        for (index, rank) in self.ranks.iter().enumerate() {
            if rank.rank as usize != index + 1 {
                return Err(TalentValidationError::InvalidRankSequence);
            }

            rank.validate()?;
        }

        let mut prerequisites = HashSet::new();

        for prerequisite in &self.prerequisites {
            if prerequisite.talent == self.metadata.id {
                return Err(TalentValidationError::SelfPrerequisite);
            }

            if prerequisite.required_rank == 0 {
                return Err(TalentValidationError::ZeroPrerequisiteRank);
            }

            if !prerequisites.insert(&prerequisite.talent) {
                return Err(TalentValidationError::DuplicatePrerequisite);
            }
        }

        Ok(())
    }

    pub fn max_rank(&self) -> u8 {
        self.ranks.len() as u8
    }

    pub fn rank(&self, rank: u8) -> Option<&TalentRankDefinition> {
        rank.checked_sub(1)
            .and_then(|index| self.ranks.get(index as usize))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TalentPosition {
    pub row: u8,
    pub column: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TalentPrerequisite {
    pub talent: ContentId,
    pub required_rank: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TalentRankDefinition {
    pub rank: u8,
    #[serde(default)]
    pub effects: Vec<ContentId>,
    #[serde(default)]
    pub unlocks: Vec<ContentId>,
}

impl TalentRankDefinition {
    fn validate(&self) -> Result<(), TalentValidationError> {
        if self.effects.is_empty() && self.unlocks.is_empty() {
            return Err(TalentValidationError::EmptyRankDefinition);
        }

        let mut effects = HashSet::new();

        for effect in &self.effects {
            if !effects.insert(effect) {
                return Err(TalentValidationError::DuplicateRankEffect);
            }
        }

        let mut unlocks = HashSet::new();

        for unlock in &self.unlocks {
            if !unlocks.insert(unlock) {
                return Err(TalentValidationError::DuplicateRankUnlock);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TalentValidationError {
    InvalidMetadata(ContentMetadataError),
    EmptyTree,
    DuplicateTalentInTree,
    ZeroPointCost,
    MissingRanks,
    InvalidRankSequence,
    EmptyRankDefinition,
    DuplicateRankEffect,
    DuplicateRankUnlock,
    SelfPrerequisite,
    ZeroPrerequisiteRank,
    DuplicatePrerequisite,
}

impl Display for TalentValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidMetadata(error) => {
                return write!(formatter, "Metadati del talento non validi: {error}");
            }
            Self::EmptyTree => "L'albero deve contenere almeno un talento",
            Self::DuplicateTalentInTree => "L'albero contiene talenti duplicati",
            Self::ZeroPointCost => "Il costo in punti per grado deve essere maggiore di zero",
            Self::MissingRanks => "Il talento deve contenere almeno un grado",
            Self::InvalidRankSequence => {
                "I gradi del talento devono essere consecutivi e partire da uno"
            }
            Self::EmptyRankDefinition => {
                "Ogni grado deve contenere almeno un effetto o un'abilità sbloccata"
            }
            Self::DuplicateRankEffect => "Un grado contiene effetti duplicati",
            Self::DuplicateRankUnlock => "Un grado contiene abilità sbloccate duplicate",
            Self::SelfPrerequisite => "Un talento non può essere prerequisito di sé stesso",
            Self::ZeroPrerequisiteRank => {
                "Il grado richiesto di un prerequisito deve essere maggiore di zero"
            }
            Self::DuplicatePrerequisite => "Il talento contiene prerequisiti duplicati",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for TalentValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_metadata(id: &str, name: &str) -> ContentMetadata {
        ContentMetadata {
            id: ContentId::new(id).unwrap(),
            pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
            name: name.to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        }
    }

    fn valid_tree() -> TalentTreeDefinition {
        TalentTreeDefinition {
            metadata: valid_metadata("classic_1_12_1:tree.warrior_arms", "Arms"),
            class: ContentId::new("classic_1_12_1:class.warrior").unwrap(),
            display_order: 0,
            talents: vec![ContentId::new("classic_1_12_1:talent.improved_heroic_strike").unwrap()],
        }
    }

    fn valid_talent() -> TalentDefinition {
        TalentDefinition {
            metadata: valid_metadata(
                "classic_1_12_1:talent.improved_heroic_strike",
                "Improved Heroic Strike",
            ),
            tree: ContentId::new("classic_1_12_1:tree.warrior_arms").unwrap(),
            position: TalentPosition { row: 0, column: 0 },
            required_tree_points: 0,
            points_per_rank: 1,
            prerequisites: Vec::new(),
            ranks: vec![
                TalentRankDefinition {
                    rank: 1,
                    effects: vec![ContentId::new(
                        "classic_1_12_1:effect.improved_heroic_strike_rank_1",
                    )
                    .unwrap()],
                    unlocks: Vec::new(),
                },
                TalentRankDefinition {
                    rank: 2,
                    effects: vec![ContentId::new(
                        "classic_1_12_1:effect.improved_heroic_strike_rank_2",
                    )
                    .unwrap()],
                    unlocks: Vec::new(),
                },
            ],
        }
    }

    #[test]
    fn valid_tree_passes_validation() {
        assert!(valid_tree().validate().is_ok());
    }

    #[test]
    fn tree_rejects_empty_and_duplicate_talents() {
        let mut tree = valid_tree();
        tree.talents.clear();

        assert_eq!(tree.validate(), Err(TalentValidationError::EmptyTree));

        let mut tree = valid_tree();
        tree.talents.push(tree.talents[0].clone());

        assert_eq!(
            tree.validate(),
            Err(TalentValidationError::DuplicateTalentInTree)
        );
    }

    #[test]
    fn valid_talent_passes_validation() {
        let talent = valid_talent();

        assert!(talent.validate().is_ok());
        assert_eq!(talent.max_rank(), 2);
        assert_eq!(talent.rank(1).unwrap().rank, 1);
        assert_eq!(talent.rank(2).unwrap().rank, 2);
        assert!(talent.rank(3).is_none());
    }

    #[test]
    fn talent_rejects_zero_point_cost_and_missing_ranks() {
        let mut talent = valid_talent();
        talent.points_per_rank = 0;

        assert_eq!(talent.validate(), Err(TalentValidationError::ZeroPointCost));

        let mut talent = valid_talent();
        talent.ranks.clear();

        assert_eq!(talent.validate(), Err(TalentValidationError::MissingRanks));
    }

    #[test]
    fn talent_rejects_invalid_rank_sequences() {
        let mut talent = valid_talent();
        talent.ranks[1].rank = 3;

        assert_eq!(
            talent.validate(),
            Err(TalentValidationError::InvalidRankSequence)
        );
    }

    #[test]
    fn talent_rejects_empty_and_duplicate_rank_effects() {
        let mut talent = valid_talent();
        talent.ranks[0].effects.clear();

        assert_eq!(
            talent.validate(),
            Err(TalentValidationError::EmptyRankDefinition)
        );

        let mut talent = valid_talent();
        let duplicated_effect = talent.ranks[0].effects[0].clone();
        talent.ranks[0].effects.push(duplicated_effect);

        assert_eq!(
            talent.validate(),
            Err(TalentValidationError::DuplicateRankEffect)
        );
    }

    #[test]
    fn talent_rejects_duplicate_unlocked_abilities() {
        let mut talent = valid_talent();
        let ability = ContentId::new("classic_1_12_1:warrior.mortal_strike").unwrap();

        talent.ranks[0].unlocks = vec![ability.clone(), ability];

        assert_eq!(
            talent.validate(),
            Err(TalentValidationError::DuplicateRankUnlock)
        );
    }

    #[test]
    fn talent_rejects_invalid_prerequisites() {
        let mut talent = valid_talent();
        talent.prerequisites.push(TalentPrerequisite {
            talent: talent.metadata.id.clone(),
            required_rank: 1,
        });

        assert_eq!(
            talent.validate(),
            Err(TalentValidationError::SelfPrerequisite)
        );

        let mut talent = valid_talent();
        let prerequisite = ContentId::new("classic_1_12_1:talent.tactical_mastery").unwrap();

        talent.prerequisites = vec![TalentPrerequisite {
            talent: prerequisite.clone(),
            required_rank: 0,
        }];

        assert_eq!(
            talent.validate(),
            Err(TalentValidationError::ZeroPrerequisiteRank)
        );

        let mut talent = valid_talent();
        talent.prerequisites = vec![
            TalentPrerequisite {
                talent: prerequisite.clone(),
                required_rank: 1,
            },
            TalentPrerequisite {
                talent: prerequisite,
                required_rank: 2,
            },
        ];

        assert_eq!(
            talent.validate(),
            Err(TalentValidationError::DuplicatePrerequisite)
        );
    }

    #[test]
    fn talent_supports_ron_serialization() {
        let original = valid_talent();

        let serialized = ron::to_string(&original).unwrap();
        let restored: TalentDefinition = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }

    #[test]
    fn talent_tree_supports_ron_serialization() {
        let original = valid_tree();

        let serialized = ron::to_string(&original).unwrap();
        let restored: TalentTreeDefinition = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }
}
