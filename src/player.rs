mod input;
mod movement;
mod movement_helpers;

use bevy::prelude::*;

use benilla_world::schedule::WorldStage;

#[derive(Component)]
pub struct Character;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u64);

#[derive(Component, Default)]
pub struct CharacterCommand {
    pub movement: Vec2,
    pub jump: bool,
    pub look_delta: f32,
    pub select_target: bool,
    pub ability_1: bool,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterInputSet {
    Collect,
    Process,
}

#[derive(Component)]
pub enum FellowshipRole {
    Tank,
    Healer,
    Dps,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component, Default)]
pub struct GroundingState {
    pub steep_support: bool,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (CharacterInputSet::Collect, CharacterInputSet::Process).chain(),
        )
        .add_systems(
            Update,
            input::local_player_command_system.in_set(CharacterInputSet::Collect),
        )
        .add_systems(
            Update,
            movement::player_movement_system
                .in_set(CharacterInputSet::Process)
                .in_set(WorldStage::Input),
        );
    }
}
