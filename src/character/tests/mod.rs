use super::build::CharacterBuild;
use super::class::CharacterClass;
use super::race::{CharacterFaction, CharacterRace};

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
    assert!(CharacterBuild::new(CharacterRace::Dwarf, CharacterClass::Shaman).is_some());

    assert!(CharacterBuild::new(CharacterRace::Tauren, CharacterClass::Paladin).is_some());

    assert!(CharacterBuild::new(CharacterRace::Gnome, CharacterClass::Shaman).is_none());

    assert!(CharacterBuild::new(CharacterRace::Orc, CharacterClass::Paladin).is_none());
}
