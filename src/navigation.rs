
use avian3d::prelude::{Collider, CollisionLayers};
use avian_rerecast::prelude::AvianBackendPlugin;
use bevy::prelude::*;
use bevy_landmass::prelude::{
    Archipelago3d,
    ArchipelagoOptions,
    ArchipelagoRef3d,
    Island,
    Landmass3dPlugin,
};
use bevy_rerecast::prelude::{
    Navmesh,
    NavmeshGenerator,
    NavmeshPlugins,
    NavmeshReady,
    NavmeshSettings,
};
use benilla_world::collision::ColliderEpoch;
use benilla_world::terrain_stream::WorldLoadProgress;
use landmass_rerecast::{
    Island3dBundle,
    LandmassRerecastPlugin,
    NavMeshHandle3d,
};

const NAVIGATION_AGENT_RADIUS: f32 = 1.0 / 3.0;
const NAVIGATION_AGENT_HEIGHT: f32 = 2.027_777_7;
const NAVMESH_WALKABLE_COLLISION_LAYERS: u32 = 0b0011;

#[derive(Resource, Default)]
struct NavigationState {
    navmesh: Option<Handle<Navmesh>>,
    requested_epoch: Option<u64>,
}

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NavmeshPlugins::default(),
            AvianBackendPlugin::default(),
            Landmass3dPlugin::default(),
            LandmassRerecastPlugin::default(),
        ))
        .init_resource::<NavigationState>()
        .add_systems(
            Update,
            (
                exclude_non_walkable_colliders,
                generate_navmesh_when_ready,
            )
                .chain(),
        )
        .add_observer(create_navigation_island);
    }
}

fn exclude_non_walkable_colliders(
    mut commands: Commands,
    query: Query<
        (Entity, &CollisionLayers),
        (With<Collider>, Added<CollisionLayers>),
    >,
) {
    for (entity, layers) in &query {
        let memberships = layers.memberships.0;

        if memberships & NAVMESH_WALKABLE_COLLISION_LAYERS == 0 {
            commands
                .entity(entity)
                .insert(avian_rerecast::prelude::ExcludeColliderFromNavmesh);
        }
    }
}

fn generate_navmesh_when_ready(
    mut generator: NavmeshGenerator,
    progress: Option<Res<WorldLoadProgress>>,
    collider_epoch: Option<Res<ColliderEpoch>>,
    mut state: ResMut<NavigationState>,
) {
    let (Some(progress), Some(collider_epoch)) =
        (progress, collider_epoch)
    else {
        return;
    };

    if !progress.is_ready() || !progress.presentable() {
        return;
    }

    let current_epoch = collider_epoch.get();

    if state.requested_epoch == Some(current_epoch) {
        return;
    }

    let settings = NavmeshSettings::from_agent_3d(
        NAVIGATION_AGENT_RADIUS,
        NAVIGATION_AGENT_HEIGHT,
    );

    if let Some(handle) = &state.navmesh {
        if generator.regenerate(handle, settings) {
            state.requested_epoch = Some(current_epoch);
        }
    } else {
        let handle = generator.generate(settings);
        state.navmesh = Some(handle);
        state.requested_epoch = Some(current_epoch);

        info!("Generazione iniziale della NavMesh avviata.");
    }
}

fn create_navigation_island(
    trigger: On<NavmeshReady>,
    mut commands: Commands,
    state: Res<NavigationState>,
    archipelago_query: Query<Entity, With<Archipelago3d>>,
) {
    let Some(handle) = &state.navmesh else {
        return;
    };

    if trigger.event().0 != handle.id() {
        return;
    }

    if !archipelago_query.is_empty() {
        return;
    }

    let archipelago = commands
        .spawn(Archipelago3d::new(
            ArchipelagoOptions::from_agent_radius(
                NAVIGATION_AGENT_RADIUS,
            ),
        ))
        .id();

    commands.spawn(Island3dBundle {
        island: Island,
        archipelago_ref: ArchipelagoRef3d::new(archipelago),
        nav_mesh: NavMeshHandle3d(handle.clone()),
    });

    info!("NavMesh pronta e collegata al sistema di navigazione.");
}
