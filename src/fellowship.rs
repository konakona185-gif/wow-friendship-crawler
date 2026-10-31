use std::collections::{hash_map::Entry, HashMap, HashSet};

use bevy::prelude::*;

use crate::player::{Character, FellowshipRole, PlayerId};

#[derive(Resource, Default)]
pub struct Fellowship {
    members: HashMap<PlayerId, Entity>,
    duplicate_ids: HashSet<PlayerId>,
}

impl Fellowship {
    pub fn get_member(&self, player_id: PlayerId) -> Option<Entity> {
        self.members.get(&player_id).copied()
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn members(&self) -> impl Iterator<Item = (PlayerId, Entity)> + '_ {
        self.members
            .iter()
            .map(|(player_id, entity)| (*player_id, *entity))
    }

    pub fn has_duplicate_ids(&self) -> bool {
        !self.duplicate_ids.is_empty()
    }
}

pub struct FellowshipPlugin;

impl Plugin for FellowshipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fellowship>()
            .add_systems(Update, update_fellowship);
    }
}

fn update_fellowship(
    player_query: Query<
        (Entity, &PlayerId),
        (With<Character>, With<FellowshipRole>),
    >,
    mut fellowship: ResMut<Fellowship>,
) {
    let mut members = HashMap::new();
    let mut duplicate_ids = HashSet::new();

    for (entity, player_id) in &player_query {
        if duplicate_ids.contains(player_id) {
            continue;
        }

        match members.entry(*player_id) {
            Entry::Vacant(entry) => {
                entry.insert(entity);
            }
            Entry::Occupied(entry) => {
                entry.remove();
                duplicate_ids.insert(*player_id);
            }
        }
    }

    for player_id in duplicate_ids.difference(&fellowship.duplicate_ids) {
        warn!(
            "PlayerId duplicato rilevato: {:?}. I personaggi con questo identificativo non saranno registrati nella Fellowship.",
            player_id
        );
    }

    fellowship.members = members;
    fellowship.duplicate_ids = duplicate_ids;
}
