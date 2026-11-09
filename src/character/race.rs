use super::class::{CharacterClass, ALL_CLASSES};

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

impl CharacterRace {
    pub fn faction(self) -> CharacterFaction {
        match self {
            Self::Human | Self::Dwarf | Self::NightElf | Self::Gnome => CharacterFaction::Alliance,

            Self::Orc | Self::Undead | Self::Tauren | Self::Troll => CharacterFaction::Horde,
        }
    }

    pub fn can_play_class(self, class: CharacterClass) -> bool {
        match class {
            CharacterClass::Warrior => true,

            CharacterClass::Paladin => matches!(self, Self::Human | Self::Dwarf | Self::Tauren),

            CharacterClass::Hunter => matches!(
                self,
                Self::Dwarf | Self::NightElf | Self::Orc | Self::Tauren | Self::Troll
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
                Self::Human | Self::Dwarf | Self::NightElf | Self::Undead | Self::Troll
            ),

            CharacterClass::Shaman => {
                matches!(self, Self::Dwarf | Self::Orc | Self::Tauren | Self::Troll)
            }

            CharacterClass::Mage => {
                matches!(self, Self::Human | Self::Gnome | Self::Undead | Self::Troll)
            }

            CharacterClass::Warlock => {
                matches!(self, Self::Human | Self::Gnome | Self::Orc | Self::Undead)
            }

            CharacterClass::Druid => matches!(self, Self::NightElf | Self::Tauren),
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
