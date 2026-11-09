use bevy::prelude::Entity;

use crate::combat::{calculate_damage_mitigation, CombatAttributes, CombatLevel, DamageRequest};
use crate::content::{ContentDatabase, ContentId, ContentPackLoader, DamageSchoolMitigation};

#[test]
fn classic_pack_loads_all_composite_damage_schools() {
    let mut database = ContentDatabase::new();

    ContentPackLoader::load_ron(
        &mut database,
        include_str!("../../../../content/packs/classic_1_12_1.ron"),
    )
    .unwrap();

    let expected_schools = [
        (
            "classic_1_12_1:school.shadowfrost",
            "Shadowfrost",
            "classic_1_12_1:stat.shadow_resistance",
            "classic_1_12_1:stat.frost_resistance",
        ),
        (
            "classic_1_12_1:school.entropy",
            "Entropy",
            "classic_1_12_1:stat.fire_resistance",
            "classic_1_12_1:stat.shadow_resistance",
        ),
        (
            "classic_1_12_1:school.frostfire",
            "Frostfire",
            "classic_1_12_1:stat.frost_resistance",
            "classic_1_12_1:stat.fire_resistance",
        ),
    ];

    for (school_id, expected_name, first_stat, second_stat) in expected_schools {
        let school_id = ContentId::new(school_id).unwrap();
        let school = database.damage_school(&school_id).unwrap();

        assert_eq!(school.metadata.name, expected_name);

        assert_eq!(
            school.mitigation,
            DamageSchoolMitigation::Composite {
                components: vec![
                    DamageSchoolMitigation::Resistance {
                        stat: ContentId::new(first_stat).unwrap(),
                    },
                    DamageSchoolMitigation::Resistance {
                        stat: ContentId::new(second_stat).unwrap(),
                    },
                ],
            }
        );
    }
}

#[test]
fn classic_pack_composite_schools_use_only_the_highest_mitigation() {
    let mut database = ContentDatabase::new();

    ContentPackLoader::load_ron(
        &mut database,
        include_str!("../../../../content/packs/classic_1_12_1.ron"),
    )
    .unwrap();

    let expected_schools = [
        (
            "classic_1_12_1:school.shadowfrost",
            "classic_1_12_1:stat.shadow_resistance",
            "classic_1_12_1:stat.frost_resistance",
        ),
        (
            "classic_1_12_1:school.entropy",
            "classic_1_12_1:stat.fire_resistance",
            "classic_1_12_1:stat.shadow_resistance",
        ),
        (
            "classic_1_12_1:school.frostfire",
            "classic_1_12_1:stat.frost_resistance",
            "classic_1_12_1:stat.fire_resistance",
        ),
    ];

    let armor_id = ContentId::new("classic_1_12_1:stat.armor").unwrap();
    let attacker_level = CombatLevel::new(60).unwrap();

    for (school_id, first_stat, second_stat) in expected_schools {
        let school_id = ContentId::new(school_id).unwrap();
        let school = database.damage_school(&school_id).unwrap();

        let mut attributes = CombatAttributes::default();

        attributes
            .set(ContentId::new(first_stat).unwrap(), 100.0)
            .unwrap();

        attributes
            .set(ContentId::new(second_stat).unwrap(), 200.0)
            .unwrap();

        let request = DamageRequest {
            source: None,
            target: Entity::from_bits(1),
            base_amount: 100.0,
            school: school_id,
        };

        let result =
            calculate_damage_mitigation(&request, school, &armor_id, &attributes, &attacker_level)
                .unwrap();

        assert_eq!(result.incoming_amount, 100.0);
        assert_eq!(result.mitigated_amount, 50.0);
        assert_eq!(result.remaining_amount(), 50.0);
    }
}
