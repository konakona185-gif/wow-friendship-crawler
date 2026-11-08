use super::*;

#[test]
fn database_validates_complete_talent_definitions() {
    let mut database = database_with_classic_pack();

    database
        .register_damage_school(physical_damage_school())
        .unwrap();
    database.register_effect(damage_effect()).unwrap();
    database.register_ability(ability()).unwrap();

    let talent_id = ContentId::new("classic:talent.test").unwrap();

    database
        .register_talent_tree(talent_tree(vec![talent_id.clone()]))
        .unwrap();

    database
        .register_talent(talent(
            "classic:talent.test",
            Vec::new(),
        ))
        .unwrap();

    assert!(database.validate().is_ok());
    assert_eq!(database.talent_tree_count(), 1);
    assert_eq!(database.talent_count(), 1);
    assert_eq!(database.ability_count(), 1);
}

#[test]
fn database_rejects_talent_tree_mismatches() {
    let mut database = database_with_classic_pack();

    let talent_id = ContentId::new("classic:talent.test").unwrap();

    database
        .register_talent_tree(talent_tree(vec![talent_id.clone()]))
        .unwrap();

    let mut mismatched_talent = talent(
        "classic:talent.test",
        Vec::new(),
    );
    mismatched_talent.tree =
        ContentId::new("classic:tree.fury").unwrap();

    database
        .register_talent(mismatched_talent)
        .unwrap();

    assert_eq!(
        database.validate(),
        Err(ContentDatabaseError::TalentTreeMismatch {
            talent: talent_id,
            tree: ContentId::new("classic:tree.arms").unwrap(),
        })
    );
}

#[test]
fn database_rejects_talent_prerequisite_cycles() {
    let mut database = database_with_classic_pack();

    database
        .register_damage_school(physical_damage_school())
        .unwrap();
    database.register_effect(damage_effect()).unwrap();
    database.register_ability(ability()).unwrap();

    let first_id = ContentId::new("classic:talent.first").unwrap();
    let second_id = ContentId::new("classic:talent.second").unwrap();

    database
        .register_talent_tree(talent_tree(vec![
            first_id.clone(),
            second_id.clone(),
        ]))
        .unwrap();

    database
        .register_talent(talent(
            "classic:talent.first",
            vec![TalentPrerequisite {
                talent: second_id.clone(),
                required_rank: 1,
            }],
        ))
        .unwrap();

    database
        .register_talent(talent(
            "classic:talent.second",
            vec![TalentPrerequisite {
                talent: first_id,
                required_rank: 1,
            }],
        ))
        .unwrap();

    assert!(matches!(
        database.validate(),
        Err(ContentDatabaseError::TalentPrerequisiteCycle(_))
    ));
}
