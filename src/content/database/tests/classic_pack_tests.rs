use crate::content::{
    ContentDatabase,
    ContentId,
    ContentPackLoader,
    DamageSchoolMitigation,
};

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
        let school = database
            .damage_school(&school_id)
            .unwrap();

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
