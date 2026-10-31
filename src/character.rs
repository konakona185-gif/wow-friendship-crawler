use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharacterRace {
    Human,
    Dwarf,
    NightElf,
    Gnome,
    Draenei,
    Worgen,
    Orc,
    Undead,
    Tauren,
    Troll,
    BloodElf,
    Goblin,
    Pandaren,
    Dracthyr,
    VoidElf,
    LightforgedDraenei,
    DarkIronDwarf,
    KulTiran,
    Mechagnome,
    Nightborne,
    HighmountainTauren,
    MagharOrc,
    ZandalariTroll,
    Vulpera,
    Earthen,
    Haranir,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharacterClass {
    Druid,
    Hunter,
    Mage,
    Paladin,
    Priest,
    Rogue,
    Shaman,
    Warlock,
    Warrior,
    DeathKnight,
    Monk,
    DemonHunter,
    Evoker,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharacterSpecialization {
    DruidBalance,
    DruidFeral,
    DruidGuardian,
    DruidRestoration,

    HunterBeastMastery,
    HunterMarksmanship,
    HunterSurvival,

    MageArcane,
    MageFire,
    MageFrost,

    PaladinHoly,
    PaladinProtection,
    PaladinRetribution,

    PriestDiscipline,
    PriestHoly,
    PriestShadow,

    RogueAssassination,
    RogueOutlaw,
    RogueSubtlety,

    ShamanElemental,
    ShamanEnhancement,
    ShamanRestoration,

    WarlockAffliction,
    WarlockDemonology,
    WarlockDestruction,

    WarriorArms,
    WarriorFury,
    WarriorProtection,

    DeathKnightBlood,
    DeathKnightFrost,
    DeathKnightUnholy,

    MonkBrewmaster,
    MonkMistweaver,
    MonkWindwalker,

    DemonHunterHavoc,
    DemonHunterVengeance,
    DemonHunterDevourer,

    EvokerDevastation,
    EvokerPreservation,
    EvokerAugmentation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharacterRole {
    Tank,
    Healer,
    Dps,
}

impl CharacterSpecialization {
    pub fn class(self) -> CharacterClass {
        match self {
            Self::DruidBalance
            | Self::DruidFeral
            | Self::DruidGuardian
            | Self::DruidRestoration => CharacterClass::Druid,

            Self::HunterBeastMastery
            | Self::HunterMarksmanship
            | Self::HunterSurvival => CharacterClass::Hunter,

            Self::MageArcane
            | Self::MageFire
            | Self::MageFrost => CharacterClass::Mage,

            Self::PaladinHoly
            | Self::PaladinProtection
            | Self::PaladinRetribution => CharacterClass::Paladin,

            Self::PriestDiscipline
            | Self::PriestHoly
            | Self::PriestShadow => CharacterClass::Priest,

            Self::RogueAssassination
            | Self::RogueOutlaw
            | Self::RogueSubtlety => CharacterClass::Rogue,

            Self::ShamanElemental
            | Self::ShamanEnhancement
            | Self::ShamanRestoration => CharacterClass::Shaman,

            Self::WarlockAffliction
            | Self::WarlockDemonology
            | Self::WarlockDestruction => CharacterClass::Warlock,

            Self::WarriorArms
            | Self::WarriorFury
            | Self::WarriorProtection => CharacterClass::Warrior,

            Self::DeathKnightBlood
            | Self::DeathKnightFrost
            | Self::DeathKnightUnholy => CharacterClass::DeathKnight,

            Self::MonkBrewmaster
            | Self::MonkMistweaver
            | Self::MonkWindwalker => CharacterClass::Monk,

            Self::DemonHunterHavoc
            | Self::DemonHunterVengeance
            | Self::DemonHunterDevourer => CharacterClass::DemonHunter,

            Self::EvokerDevastation
            | Self::EvokerPreservation
            | Self::EvokerAugmentation => CharacterClass::Evoker,
        }
    }

    pub fn role(self) -> CharacterRole {
        match self {
            Self::DruidGuardian
            | Self::PaladinProtection
            | Self::WarriorProtection
            | Self::DeathKnightBlood
            | Self::MonkBrewmaster
            | Self::DemonHunterVengeance => CharacterRole::Tank,

            Self::DruidRestoration
            | Self::PaladinHoly
            | Self::PriestDiscipline
            | Self::PriestHoly
            | Self::ShamanRestoration
            | Self::MonkMistweaver
            | Self::EvokerPreservation => CharacterRole::Healer,

            _ => CharacterRole::Dps,
        }
    }
}

impl CharacterRace {
    pub fn can_play_class(self, class: CharacterClass) -> bool {
        match class {
            CharacterClass::Druid => matches!(
                self,
                Self::NightElf
                    | Self::Worgen
                    | Self::KulTiran
                    | Self::Haranir
                    | Self::Tauren
                    | Self::Troll
                    | Self::HighmountainTauren
                    | Self::ZandalariTroll
            ),
            _ => false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterBuild {
    pub race: CharacterRace,
    pub class: CharacterClass,
    pub specialization: CharacterSpecialization,
}

impl CharacterBuild {
    pub fn is_valid(self) -> bool {
        self.class == self.specialization.class()
            && self.race.can_play_class(self.class)
    }

    pub fn role(self) -> CharacterRole {
        self.specialization.role()
    }
}
