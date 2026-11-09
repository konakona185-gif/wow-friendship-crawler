use super::*;

#[test]
fn loader_rejects_content_from_another_pack_transactionally() {
    let mut database = ContentDatabase::new();
    let mut pack = empty_pack();

    pack.effects.push(EffectDefinition {
        metadata: ContentMetadata {
            id: ContentId::new("other:effect.test").unwrap(),
            pack_id: ContentId::new("pack:other").unwrap(),
            name: "Test".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        },
        trigger: super::super::super::EffectTrigger::OnCast,
        target: super::super::super::EffectTarget::SelectedTarget,
        kind: super::super::super::EffectKind::Damage {
            amount: super::super::super::EffectMagnitude {
                base: 10.0,
                scaling: Vec::new(),
            },
            school: ContentId::new("classic:school.physical").unwrap(),
        },
    });

    let result = ContentPackLoader::load(&mut database, pack);

    assert!(matches!(
        result,
        Err(ContentPackLoaderError::ContentFromDifferentPack { .. })
    ));

    assert!(database.packs().is_empty());
}

#[test]
fn loader_rejects_invalid_ron() {
    assert!(ContentPackLoader::parse_ron("invalid ron").is_err());
}
