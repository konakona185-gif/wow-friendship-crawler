use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharacterFaction {
    Alliance,
    Horde,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharacterRace {
    Human,
    Dwarf,
    NightElf,
    Gnome,
    Orc,
    Undead,
    Tauren,
    Troll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharacterClass {
    Warrior,
    Paladin,
    Hunter,
    Rogue,
    Priest,
    Shaman,
    Mage,
    Warlock,
    Druid,
}

const ALL_CLASSES: [CharacterClass; 9] = [
    CharacterClass::Warrior,
    CharacterClass::Paladin,
    CharacterClass::Hunter,
    CharacterClass::Rogue,
    CharacterClass::Priest,
    CharacterClass::Shaman,
    CharacterClass::Mage,
    CharacterClass::Warlock,
    CharacterClass::Druid,
];

impl CharacterRace {
    pub fn faction(self) -> CharacterFaction {
        match self {
            Self::Human
            | Self::Dwarf
            | Self::NightElf
            | Self::Gnome => CharacterFaction::Alliance,

            Self::Orc
            | Self::Undead
            | Self::Tauren
            | Self::Troll => CharacterFaction::Horde,
        }
    }

    pub fn can_play_class(self, class: CharacterClass) -> bool {
        match class {
            CharacterClass::Warrior => true,

            CharacterClass::Paladin => matches!(
                self,
                Self::Human | Self::Dwarf | Self::Tauren
            ),

            CharacterClass::Hunter => matches!(
                self,
                Self::Dwarf
                    | Self::NightElf
                    | Self::Orc
                    | Self::Tauren
                    | Self::Troll
            ),

            CharacterClass::Rogue => matches!(
                self,
                Self::Human
                    | Self::Dwarf
                    | Self::NightElf
                    | Self::Gnome
                    | Self::Orc
                    | Self::Undead
                    | Self::Troll
            ),

            CharacterClass::Priest => matches!(
                self,
                Self::Human
                    | Self::Dwarf
                    | Self::NightElf
                    | Self::Undead
                    | Self::Troll
            ),

            CharacterClass::Shaman => matches!(
                self,
                Self::Dwarf
                    | Self::Orc
                    | Self::Tauren
                    | Self::Troll
            ),

            CharacterClass::Mage => matches!(
                self,
                Self::Human
                    | Self::Gnome
                    | Self::Undead
                    | Self::Troll
            ),

            CharacterClass::Warlock => matches!(
                self,
                Self::Human
                    | Self::Gnome
                    | Self::Orc
                    | Self::Undead
            ),

            CharacterClass::Druid => matches!(
                self,
                Self::NightElf | Self::Tauren
            ),
        }
    }

    pub fn available_classes(self) -> Vec<CharacterClass> {
        ALL_CLASSES
            .iter()
            .copied()
            .filter(|class| self.can_play_class(*class))
            .collect()
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterBuild {
    pub race: CharacterRace,
    pub class: CharacterClass,
}

impl CharacterBuild {
    pub fn new(
        race: CharacterRace,
        class: CharacterClass,
    ) -> Option<Self> {
        if race.can_play_class(class) {
            Some(Self { race, class })
        } else {
            None
        }
    }

    pub fn is_valid(self) -> bool {
        self.race.can_play_class(self.class)
    }

    pub fn faction(self) -> CharacterFaction {
        self.race.faction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RACES: [CharacterRace; 8] = [
        CharacterRace::Human,
        CharacterRace::Dwarf,
        CharacterRace::NightElf,
        CharacterRace::Gnome,
        CharacterRace::Orc,
        CharacterRace::Undead,
        CharacterRace::Tauren,
        CharacterRace::Troll,
    ];

    const CLASSES: [CharacterClass; 9] = [
        CharacterClass::Warrior,
        CharacterClass::Paladin,
        CharacterClass::Hunter,
        CharacterClass::Rogue,
        CharacterClass::Priest,
        CharacterClass::Shaman,
        CharacterClass::Mage,
        CharacterClass::Warlock,
        CharacterClass::Druid,
    ];

    fn expected_classes(race: CharacterRace) -> &'static [CharacterClass] {
        match race {
            CharacterRace::Human => &[
                CharacterClass::Warrior,
                CharacterClass::Paladin,
                CharacterClass::Rogue,
                CharacterClass::Priest,
                CharacterClass::Mage,
                CharacterClass::Warlock,
            ],

            CharacterRace::Dwarf => &[
                CharacterClass::Warrior,
                CharacterClass::Paladin,
                CharacterClass::Hunter,
                CharacterClass::Rogue,
                CharacterClass::Priest,
                CharacterClass::Shaman,
            ],

            CharacterRace::NightElf => &[
                CharacterClass::Warrior,
                CharacterClass::Hunter,
                CharacterClass::Rogue,
                CharacterClass::Priest,
                CharacterClass::Druid,
            ],

            CharacterRace::Gnome => &[
                CharacterClass::Warrior,
                CharacterClass::Rogue,
                CharacterClass::Mage,
                CharacterClass::Warlock,
            ],

            CharacterRace::Orc => &[
                CharacterClass::Warrior,
                CharacterClass::Hunter,
                CharacterClass::Rogue,
                CharacterClass::Shaman,
                CharacterClass::Warlock,
            ],

            CharacterRace::Undead => &[
                CharacterClass::Warrior,
                CharacterClass::Rogue,
                CharacterClass::Priest,
                CharacterClass::Mage,
                CharacterClass::Warlock,
            ],

            CharacterRace::Tauren => &[
                CharacterClass::Warrior,
                CharacterClass::Paladin,
                CharacterClass::Hunter,
                CharacterClass::Shaman,
                CharacterClass::Druid,
            ],

            CharacterRace::Troll => &[
                CharacterClass::Warrior,
                CharacterClass::Hunter,
                CharacterClass::Rogue,
                CharacterClass::Priest,
                CharacterClass::Shaman,
                CharacterClass::Mage,
            ],
        }
    }

    #[test]
    fn character_counts_are_correct() {
        assert_eq!(RACES.len(), 8);
        assert_eq!(CLASSES.len(), 9);
    }

    #[test]
    fn race_class_matrix_is_exact() {
        for race in RACES {
            let expected = expected_classes(race);
            let available = race.available_classes();

            assert_eq!(
                available.as_slice(),
                expected,
                "Matrice delle classi errata per la razza {:?}.",
                race
            );

            for class in CLASSES {
                let should_be_available = expected.contains(&class);

                assert_eq!(
                    race.can_play_class(class),
                    should_be_available,
                    "Combinazione razza-classe errata: {:?} - {:?}.",
                    race,
                    class
                );

                assert_eq!(
                    CharacterBuild::new(race, class).is_some(),
                    should_be_available,
                    "Validazione della configurazione errata: {:?} - {:?}.",
                    race,
                    class
                );
            }
        }
    }

    #[test]
    fn faction_assignments_are_correct() {
        for race in [
            CharacterRace::Human,
            CharacterRace::Dwarf,
            CharacterRace::NightElf,
            CharacterRace::Gnome,
        ] {
            assert_eq!(race.faction(), CharacterFaction::Alliance);
        }

        for race in [
            CharacterRace::Orc,
            CharacterRace::Undead,
            CharacterRace::Tauren,
            CharacterRace::Troll,
        ] {
            assert_eq!(race.faction(), CharacterFaction::Horde);
        }
    }

    #[test]
    fn custom_class_exceptions_are_valid() {
        assert!(
            CharacterBuild::new(
                CharacterRace::Dwarf,
                CharacterClass::Shaman
            ).is_some()
        );

        assert!(
            CharacterBuild::new(
                CharacterRace::Tauren,
                CharacterClass::Paladin
            ).is_some()
        );

        assert!(
            CharacterBuild::new(
                CharacterRace::Gnome,
                CharacterClass::Shaman
            ).is_none()
        );

        assert!(
            CharacterBuild::new(
                CharacterRace::Orc,
                CharacterClass::Paladin
            ).is_none()
        );
    }
}
