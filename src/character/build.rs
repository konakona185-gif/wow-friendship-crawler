use bevy::prelude::Component;

use super::class::CharacterClass;
use super::race::{CharacterFaction, CharacterRace};

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
