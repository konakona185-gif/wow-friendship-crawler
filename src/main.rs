
mod player;
mod camera;
mod combat;
mod fellowship;
mod character;
mod content;
mod interaction;
mod navigation;
mod movement;

use bevy::camera::{PerspectiveProjection, Projection};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::render::view::Hdr;

use benilla_assets::coords::wow_to_bevy;
use benilla_world::ffx_glow::FfxGlow;
use benilla_world::interact::{CreaturePickPart, PickBox, WorldObject};
use benilla_world::model_render::ModelKind;
use benilla_world::schedule::WorldLive;
use benilla_world::terrain_stream::SPAWN_XY;
use benilla_world::view::{
    MsaaSetting,
    WorldCamera,
    CAM_FAR,
    CAM_FOVY,
    NEARCLIP_DEFAULT,
};
use benilla_world::world_plugins::WorldPlugins;

use player::{
    Character,
    LocalPlayer,
    PlayerId,
    CharacterCommand,
    FellowshipRole,
    MovementSpeed,
    Velocity,
    GroundingState,
    PlayerPlugin,
};

use camera::{ThirdPersonCamera, CameraPlugin};
use combat::{
    Target,
    Enemy,
    EnemyAi,
    CombatStats,
    CombatLevel,
    CombatAttributes,
    HealthPool,
    CombatPlugin,
    TargetIndicator,
};
use fellowship::{
    Fellowship,
    FellowshipPlugin,
    JoinFellowship,
    LeaveFellowship,
};
use character::{CharacterBuild, CharacterClass, CharacterRace};
use content::{ContentDatabase, ContentId, ContentPackLoader};
use interaction::InteractionPlugin;
use navigation::{EnemyNavigationPlugin, NavigationPlugin};

const STARTING_COMBAT_LEVEL: u32 = 60;

fn main() {
    let is_headless = std::env::var_os("CI").is_some();

    if is_headless {
        run_headless();
    } else {
        run_game();
    }
}

fn run_headless() {
    let mut app = App::new();

    app.insert_resource(load_content_database());

    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_systems(Startup, setup_headless_game);

    app.add_plugins((
        PlayerPlugin,
        CameraPlugin,
        CombatPlugin,
        FellowshipPlugin,
        InteractionPlugin,
    ));

    app.update();
    run_headless_tests(&mut app);
}

fn run_game() {
    let mut app = App::new();

    app.insert_resource(load_content_database());

    if let Some(data_dir) = benilla_formats::wow_data() {
        if let Err(error) =
            benilla_assets::register_mpq_source(&mut app, &data_dir)
        {
            eprintln!("Impossibile registrare gli archivi WoW: {error:#}");
        }
    } else {
        eprintln!(
            "Installazione WoW 1.12.1 non trovata. Percorsi controllati: {:?}",
            benilla_formats::candidates()
        );
    }

    app.add_plugins(benilla_world::boot::tuned_default_plugins(Window {
        title: "Fellowship WoW Crawler".into(),
        resolution: UVec2::new(1280, 720).into(),
        ..default()
    }));

    app.add_plugins(benilla_world::thread_qos::ThreadQosPlugin);
    app.add_plugins(WorldPlugins);
    app.add_plugins(NavigationPlugin);
    app.add_plugins(EnemyNavigationPlugin);

    app.insert_resource(WorldLive(true));

    benilla_assets::register_asset_loaders(&mut app);

    app.add_systems(
        Startup,
        setup_game.after(benilla_assets::AssetSet::Open),
    );

    app.add_plugins((
        PlayerPlugin,
        CameraPlugin,
        CombatPlugin,
        FellowshipPlugin,
        InteractionPlugin,
    ));

    app.run();
}

fn load_content_database() -> ContentDatabase {
    let mut database = ContentDatabase::new();

    ContentPackLoader::load_ron(
        &mut database,
        include_str!("../content/packs/classic_1_12_1.ron"),
    )
    .expect("Impossibile caricare il pacchetto Classic 1.12.1.");

    database
}

fn run_headless_tests(app: &mut App) {
    {
        let database = app.world().resource::<ContentDatabase>();

        let fundamental_schools = [
            "classic_1_12_1:school.physical",
            "classic_1_12_1:school.holy",
            "classic_1_12_1:school.shadow",
            "classic_1_12_1:school.nature",
            "classic_1_12_1:school.frost",
            "classic_1_12_1:school.arcane",
            "classic_1_12_1:school.fire",
        ];

        for school_id in fundamental_schools {
            let id = ContentId::new(school_id).unwrap();

            assert!(
                database.damage_school(&id).is_some(),
                "Manca una scuola fondamentale: {school_id}."
            );
        }

        let composite_schools = [
            "classic_1_12_1:school.shadowfrost",
            "classic_1_12_1:school.entropy",
            "classic_1_12_1:school.frostfire",
        ];

        for school_id in composite_schools {
            let id = ContentId::new(school_id).unwrap();

            assert!(
                database.damage_school(&id).is_some(),
                "Manca una scuola composita: {school_id}."
            );
        }

        assert_eq!(
            database.damage_school_count(),
            10,
            "Il database deve contenere sette scuole fondamentali e tre composite."
        );

        assert_eq!(
            database.combat_attribute_count(),
            10,
            "Il database deve contenere i dieci attributi di combattimento."
        );

        assert_eq!(
            database.resource_count(),
            3,
            "Il database deve contenere Mana, Rage ed Energy."
        );

        assert!(
            database.creature_count() == 1
                && database.effect_count() == 3
                && database.aura_count() == 1
                && database.ability_count() == 1,
            "Il pacchetto deve contenere l'Imp, tre effetti, un'aura e un'abilità."
        );

        assert!(
            database.validate().is_ok(),
            "Il database dei contenuti deve superare la validazione."
        );
    }

    println!("Test di caricamento del pacchetto Classic superato.");

    assert!(
        CharacterBuild::new(CharacterRace::Dwarf, CharacterClass::Shaman).is_some(),
        "Il Nano deve poter scegliere Sciamano."
    );

    assert!(
        CharacterBuild::new(CharacterRace::Tauren, CharacterClass::Paladin).is_some(),
        "Il Tauren deve poter scegliere Paladino."
    );

    assert!(
        CharacterBuild::new(CharacterRace::Gnome, CharacterClass::Shaman).is_none(),
        "Lo Gnomo non deve poter scegliere Sciamano."
    );

    assert!(
        CharacterBuild::new(CharacterRace::Human, CharacterClass::Druid).is_none(),
        "L'Umano non deve poter scegliere Druido."
    );

    println!("Test delle combinazioni razza-classe superato.");

    {
        let world = app.world_mut();

        let mut player_query = world.query_filtered::<Entity, With<Character>>();
        let player_count = player_query.iter(world).count();

        let mut local_player_query = world.query_filtered::<Entity, With<LocalPlayer>>();
        let local_player_count = local_player_query.iter(world).count();

        let mut command_query = world.query_filtered::<Entity, With<CharacterCommand>>();
        let command_count = command_query.iter(world).count();

        let mut player_id_query = world.query_filtered::<&PlayerId, With<Character>>();
        let player_id_count = player_id_query
            .iter(world)
            .filter(|id| **id == PlayerId(1))
            .count();

        let mut build_query = world.query_filtered::<&CharacterBuild, With<Character>>();
        let builds: Vec<CharacterBuild> = build_query.iter(world).copied().collect();

        let mut enemy_query = world.query_filtered::<Entity, With<Enemy>>();
        let enemy_count = enemy_query.iter(world).count();

        let mut enemy_ai_query = world.query_filtered::<&EnemyAi, With<Enemy>>();
        let enemy_ai_states: Vec<EnemyAi> = enemy_ai_query.iter(world).copied().collect();

        let mut combat_level_query =
            world.query_filtered::<&CombatLevel, With<CombatStats>>();
        let combatant_levels: Vec<u32> = combat_level_query
            .iter(world)
            .map(|level| level.value())
            .collect();

        let mut attributes_query =
            world.query_filtered::<&CombatAttributes, With<CombatStats>>();
        let combatant_attributes_count = attributes_query.iter(world).count();

        let mut health_pool_query =
            world.query::<(&CombatStats, &HealthPool)>();
        let combatant_health_values: Vec<(f32, f32, f32, f32)> =
            health_pool_query
                .iter(world)
                .map(|(stats, health_pool)| {
                    (
                        stats.current_health,
                        stats.max_health,
                        health_pool.current(),
                        health_pool.maximum(),
                    )
                })
                .collect();

        assert_eq!(
            player_count,
            1,
            "Il test headless deve creare esattamente un giocatore."
        );

        assert_eq!(
            local_player_count,
            1,
            "Il test headless deve assegnare un LocalPlayer."
        );

        assert_eq!(
            command_count,
            1,
            "Il test headless deve assegnare i comandi al giocatore."
        );

        assert_eq!(
            player_id_count,
            1,
            "Il test headless deve assegnare PlayerId(1)."
        );

        assert_eq!(
            builds.len(),
            1,
            "Il giocatore deve avere una configurazione razza-classe."
        );

        assert!(
            builds[0].is_valid(),
            "La configurazione del personaggio deve essere valida."
        );

        assert_eq!(builds[0].race, CharacterRace::Human);
        assert_eq!(builds[0].class, CharacterClass::Warrior);

        assert_eq!(
            enemy_count,
            3,
            "Il test headless deve creare tre nemici."
        );

        assert_eq!(
            enemy_ai_states.len(),
            enemy_count,
            "Tutti i nemici devono avere il componente EnemyAi."
        );

        assert!(
            enemy_ai_states.iter().all(|ai| ai.target.is_some()),
            "Tutti i nemici vicini devono aver acquisito un bersaglio."
        );

        assert_eq!(
            combatant_levels.len(),
            player_count + enemy_count,
            "Tutti i combattenti devono avere CombatLevel."
        );

        assert!(
            combatant_levels
                .iter()
                .all(|level| *level == STARTING_COMBAT_LEVEL),
            "Tutti i combattenti iniziali devono essere al livello 60."
        );

        assert_eq!(
            combatant_attributes_count,
            player_count + enemy_count,
            "Tutti i combattenti devono avere CombatAttributes."
        );

        assert_eq!(
            combatant_health_values.len(),
            player_count + enemy_count,
            "Tutti i combattenti devono avere HealthPool."
        );

        assert!(
            combatant_health_values.iter().all(
                |(stats_current, stats_max, pool_current, pool_max)| {
                    stats_current == pool_current && stats_max == pool_max
                }
            ),
            "CombatStats e HealthPool devono iniziare con valori sincronizzati."
        );

        let fellowship = world.resource::<Fellowship>();

        assert_eq!(
            fellowship.member_count(),
            1,
            "La Fellowship deve contenere un membro iniziale."
        );

        assert!(
            fellowship.get_member(PlayerId(1)).is_some(),
            "PlayerId(1) deve essere registrato nella Fellowship."
        );

        assert!(
            !fellowship.has_duplicate_ids(),
            "La Fellowship non deve contenere ID duplicati."
        );
    }

    println!("Test iniziale della Fellowship superato.");
    println!("Test di acquisizione dei bersagli nemici superato.");
    println!("Test dei livelli iniziali dei combattenti superato.");
    println!("Test degli attributi di combattimento superato.");
    println!("Test delle HealthPool iniziali superato.");

    app.world_mut().write_message(LeaveFellowship {
        player_id: PlayerId(1),
    });

    app.update();

    {
        let world = app.world_mut();
        let fellowship = world.resource::<Fellowship>();

        assert_eq!(fellowship.member_count(), 0);

        assert!(
            fellowship.get_member(PlayerId(1)).is_none(),
            "Il giocatore deve essere uscito dalla Fellowship."
        );
    }

    {
        let world = app.world_mut();
        let mut player_query = world.query_filtered::<Entity, With<Character>>();

        assert_eq!(
            player_query.iter(world).count(),
            1,
            "Uscire dalla Fellowship non deve eliminare il personaggio."
        );
    }

    println!("Test di uscita dalla Fellowship superato.");

    app.world_mut().write_message(JoinFellowship {
        player_id: PlayerId(1),
    });

    app.update();

    {
        let world = app.world_mut();
        let fellowship = world.resource::<Fellowship>();

        assert_eq!(fellowship.member_count(), 1);

        assert!(
            fellowship.get_member(PlayerId(1)).is_some(),
            "Il giocatore deve poter rientrare nella Fellowship."
        );

        assert!(!fellowship.has_duplicate_ids());
    }

    println!("Test di rientro nella Fellowship superato.");

    println!(
        "Test headless completato: personaggio, razza, classe, livelli, attributi, HealthPool, Fellowship, tre nemici e acquisizione dei bersagli verificati."
    );
}

fn setup_headless_game(mut commands: Commands) {
    commands.spawn((
        Character,
        LocalPlayer,
        PlayerId(1),
        CharacterBuild::new(
            CharacterRace::Human,
            CharacterClass::Warrior,
        )
        .expect("Configurazione iniziale razza-classe non valida."),
        CharacterCommand::default(),
        FellowshipRole::Tank,
        MovementSpeed(8.0),
        Velocity(Vec3::ZERO),
        GroundingState::default(),
        Target { entity: None },
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
        CombatLevel::new(STARTING_COMBAT_LEVEL)
            .expect("Livello iniziale del giocatore non valido."),
        CombatAttributes::default(),
        Transform::from_xyz(0.0, 1.0, 0.0),
        GlobalTransform::default(),
    ))
    .insert(
        HealthPool::new(100.0, 100.0)
            .expect("HealthPool iniziale del giocatore non valida."),
    );

    let enemy_positions = [
        Vec3::new(5.0, 0.5, -5.0),
        Vec3::new(-6.0, 0.5, -8.0),
        Vec3::new(2.0, 0.5, -12.0),
    ];

    for (i, pos) in enemy_positions.iter().enumerate() {
        let health = 50.0 + (i as f32 * 20.0);

        commands.spawn((
            Enemy,
            EnemyAi::default(),
            CombatStats {
                current_health: health,
                max_health: health,
                resource: 0.0,
            },
            HealthPool::new(health, health)
                .expect("HealthPool iniziale del nemico non valida."),
            CombatLevel::new(STARTING_COMBAT_LEVEL)
                .expect("Livello iniziale del nemico non valido."),
            CombatAttributes::default(),
            Transform::from_translation(*pos),
            GlobalTransform::default(),
        ));
    }
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    msaa: Res<MsaaSetting>,
) {
    let spawn_position = wow_to_bevy([
        SPAWN_XY.0,
        SPAWN_XY.1,
        84.53,
    ]);

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 4000.0,
            ..default()
        },
        Transform::from_translation(
            spawn_position + Vec3::new(10.0, 20.0, 10.0),
        )
        .looking_at(spawn_position, Vec3::Y),
    ));

    let player_mesh = meshes.add(Capsule3d::new(0.5, 1.0));
    let player_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.8, 0.2),
        ..default()
    });

    commands
        .spawn((
            Character,
            LocalPlayer,
            PlayerId(1),
            CharacterBuild::new(
                CharacterRace::Human,
                CharacterClass::Warrior,
            )
            .expect("Configurazione iniziale razza-classe non valida."),
            CharacterCommand::default(),
            FellowshipRole::Tank,
            MovementSpeed(8.0),
            Velocity(Vec3::ZERO),
            GroundingState::default(),
            Target { entity: None },
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            CombatLevel::new(STARTING_COMBAT_LEVEL)
                .expect("Livello iniziale del giocatore non valido."),
            CombatAttributes::default(),
            Mesh3d(player_mesh),
            MeshMaterial3d(player_material),
        ))
        .insert(
            HealthPool::new(100.0, 100.0)
                .expect("HealthPool iniziale del giocatore non valida."),
        )
        .insert((
            PickBox,
            CreaturePickPart,
            WorldObject {
                kind: ModelKind::Creature,
                label: "Giocatore".into(),
                id: 0,
                detail: "Warrior".into(),
            },
            Transform::from_translation(spawn_position),
        ));

    let enemy_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let enemy_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.1, 0.1),
        ..default()
    });

    let target_indicator_mesh = meshes.add(Torus::new(0.64, 0.68));
    let target_indicator_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.12, 0.04),
        unlit: true,
        ..default()
    });

    let enemy_positions = [
        spawn_position + Vec3::new(5.0, -0.5, -5.0),
        spawn_position + Vec3::new(-6.0, -0.5, -8.0),
        spawn_position + Vec3::new(2.0, -0.5, -12.0),
    ];

    for (i, pos) in enemy_positions.iter().enumerate() {
        let health = 50.0 + (i as f32 * 20.0);

        let enemy_entity = commands.spawn((
            Enemy,
            EnemyAi::default(),
            CombatStats {
                current_health: health,
                max_health: health,
                resource: 0.0,
            },
            HealthPool::new(health, health)
                .expect("HealthPool iniziale del nemico non valida."),
            CombatLevel::new(STARTING_COMBAT_LEVEL)
                .expect("Livello iniziale del nemico non valido."),
            CombatAttributes::default(),
            Mesh3d(enemy_mesh.clone()),
            MeshMaterial3d(enemy_material.clone()),
            PickBox,
            CreaturePickPart,
            WorldObject {
                kind: ModelKind::Creature,
                label: format!("Nemico {}", i + 1),
                id: 0,
                detail: String::new(),
            },
            Transform::from_translation(*pos),
        )).id();

        commands.spawn((
            Mesh3d(target_indicator_mesh.clone()),
            MeshMaterial3d(target_indicator_material.clone()),
            TargetIndicator {
                enemy: enemy_entity,
            },
            Transform::from_translation(
                *pos + Vec3::new(0.0, -0.49, 0.0),
            ),
            Visibility::Hidden,
        ));
    }

    commands.spawn((
        ThirdPersonCamera {
            distance: 8.0,
            height: 4.5,
        },
        Camera3d::default(),
        WorldCamera,
        msaa.level(),
        Hdr,
        Tonemapping::None,
        FfxGlow::WORLD,
        Projection::from(PerspectiveProjection {
            far: CAM_FAR,
            near: NEARCLIP_DEFAULT,
            fov: CAM_FOVY,
            ..default()
        }),
        Transform::from_translation(
            spawn_position + Vec3::new(0.0, 4.5, 8.0),
        )
        .looking_at(spawn_position, Vec3::Y),
    ));
}
