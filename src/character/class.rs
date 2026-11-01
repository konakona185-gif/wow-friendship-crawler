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

pub(super) const ALL_CLASSES: [CharacterClass; 9] = [
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
