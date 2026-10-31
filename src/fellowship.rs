use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::character::CharacterBuild;
use crate::player::{Character, PlayerId};

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

    pub fn contains(&self, player_id: PlayerId) -> bool {
        self.members.contains_key(&player_id)
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

#[derive(Event, Clone, Copy)]
pub struct JoinFellowship {
    pub player_id: PlayerId,
}

#[derive(Event, Clone, Copy)]
pub struct LeaveFellowship {
    pub player_id: PlayerId,
}

pub struct FellowshipPlugin;

impl Plugin for FellowshipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fellowship>()
            .add_event::<JoinFellowship>()
            .add_event::<LeaveFellowship>()
            .add_systems(
                Update,
                (
                    register_new_members,
                    process_membership_requests,
                    cleanup_fellowship,
                )
                    .chain(),
            );
    }
}

fn register_new_members(
    new_players: Query<
        (Entity, &PlayerId, &CharacterBuild),
        (
            With<Character>,
            Or<(
                Added<Character>,
                Added<PlayerId>,
                Added<CharacterBuild>,
            )>,
        ),
    >,
    all_players: Query<
        (Entity, &PlayerId, &CharacterBuild),
        With<Character>,
    >,
    mut fellowship: ResMut<Fellowship>,
) {
    for (entity, player_id, build) in &new_players {
        if !build.is_valid() {
            fellowship.members.remove(player_id);

            warn!(
                "PlayerId {:?} non registrato: configurazione razza-classe non valida.",
                player_id
            );

            continue;
        }

        let matching_entities: Vec<Entity> = all_players
            .iter()
            .filter(|(_, active_id, _)| **active_id == *player_id)
            .map(|(active_entity, _, _)| active_entity)
            .take(2)
            .collect();

        if matching_entities.len() == 1 {
            if !fellowship.duplicate_ids.contains(player_id) {
                fellowship.members.insert(*player_id, entity);
            }
        } else {
            fellowship.members.remove(player_id);
            fellowship.duplicate_ids.insert(*player_id);

            warn!(
                "PlayerId duplicato rilevato: {:?}. Registrazione rifiutata.",
                player_id
            );
        }
    }
}

fn process_membership_requests(
    mut join_events: EventReader<JoinFellowship>,
    mut leave_events: EventReader<LeaveFellowship>,
    player_query: Query<
        (Entity, &PlayerId, &CharacterBuild),
        With<Character>,
    >,
    mut fellowship: ResMut<Fellowship>,
) {
    for event in join_events.read() {
        let matching_players: Vec<(Entity, CharacterBuild)> = player_query
            .iter()
            .filter(|(_, player_id, _)| **player_id == event.player_id)
            .map(|(entity, _, build)| (entity, *build))
            .take(2)
            .collect();

        if matching_players.len() != 1 {
            if matching_players.len() > 1 {
                fellowship.members.remove(&event.player_id);
                fellowship.duplicate_ids.insert(event.player_id);
            }

            warn!(
                "Impossibile aggiungere PlayerId {:?}: personaggio inesistente o identificativo duplicato.",
                event.player_id
            );

            continue;
        }

        if fellowship.duplicate_ids.contains(&event.player_id) {
            warn!(
                "Impossibile aggiungere PlayerId {:?}: identificativo in conflitto.",
                event.player_id
            );

            continue;
        }

        let (entity, build) = matching_players[0];

        if !build.is_valid() {
            warn!(
                "Impossibile aggiungere PlayerId {:?}: configurazione razza-classe non valida.",
                event.player_id
            );

            continue;
        }

        fellowship.members.insert(event.player_id, entity);

        info!(
            "PlayerId {:?} aggiunto alla Fellowship.",
            event.player_id
        );
    }

    for event in leave_events.read() {
        if fellowship.members.remove(&event.player_id).is_some() {
            info!(
                "PlayerId {:?} uscito dalla Fellowship.",
                event.player_id
            );
        }
    }
}

fn cleanup_fellowship(
    player_query: Query<
        (Entity, &PlayerId, &CharacterBuild),
        With<Character>,
    >,
    mut fellowship: ResMut<Fellowship>,
) {
    let mut active_players: HashMap<PlayerId, Vec<(Entity, bool)>> = HashMap::new();

    for (entity, player_id, build) in &player_query {
        active_players
            .entry(*player_id)
            .or_default()
            .push((entity, build.is_valid()));
    }

    let mut duplicate_ids = HashSet::new();

    for (player_id, entities) in &active_players {
        if entities.len() > 1 {
            duplicate_ids.insert(*player_id);
        }
    }

    for player_id in duplicate_ids.difference(&fellowship.duplicate_ids) {
        warn!(
            "PlayerId duplicato rilevato: {:?}. I personaggi con questo identificativo non saranno registrati nella Fellowship.",
            player_id
        );
    }

    fellowship.members.retain(|player_id, entity| {
        active_players
            .get(player_id)
            .map(|entities| {
                entities.len() == 1
                    && entities[0].0 == *entity
                    && entities[0].1
            })
            .unwrap_or(false)
    });

    for player_id in &duplicate_ids {
        fellowship.members.remove(player_id);
    }

    fellowship.duplicate_ids = duplicate_ids;
}
