use super::{ContentId, ContentPackManifest, ContentPackValidationError};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default)]
pub struct ContentPackRegistry {
    packs: HashMap<ContentId, ContentPackManifest>,
}

impl ContentPackRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        manifest: ContentPackManifest,
    ) -> Result<(), ContentPackRegistryError> {
        manifest
            .validate()
            .map_err(ContentPackRegistryError::InvalidManifest)?;

        if self.packs.contains_key(&manifest.id) {
            return Err(ContentPackRegistryError::DuplicatePack(
                manifest.id.clone(),
            ));
        }

        self.packs.insert(manifest.id.clone(), manifest);

        Ok(())
    }

    pub fn get(&self, id: &ContentId) -> Option<&ContentPackManifest> {
        self.packs.get(id)
    }

    pub fn len(&self) -> usize {
        self.packs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packs.is_empty()
    }

    pub fn load_order(
        &self,
    ) -> Result<Vec<&ContentPackManifest>, ContentPackRegistryError> {
        let mut pack_ids: Vec<ContentId> = self.packs.keys().cloned().collect();

        pack_ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut ordered_packs = Vec::new();

        for pack_id in &pack_ids {
            visit_pack(
                self,
                pack_id,
                &mut visiting,
                &mut visited,
                &mut ordered_packs,
            )?;
        }

        Ok(ordered_packs)
    }
}

fn visit_pack<'a>(
    registry: &'a ContentPackRegistry,
    pack_id: &ContentId,
    visiting: &mut HashSet<ContentId>,
    visited: &mut HashSet<ContentId>,
    ordered_packs: &mut Vec<&'a ContentPackManifest>,
) -> Result<(), ContentPackRegistryError> {
    if visited.contains(pack_id) {
        return Ok(());
    }

    if !visiting.insert(pack_id.clone()) {
        return Err(ContentPackRegistryError::DependencyCycle(
            pack_id.clone(),
        ));
    }

    let manifest = registry
        .packs
        .get(pack_id)
        .ok_or_else(|| ContentPackRegistryError::UnknownPack(pack_id.clone()))?;

    let mut dependencies: Vec<&ContentId> =
        manifest.dependencies.iter().collect();

    dependencies.sort_by(|a, b| a.as_str().cmp(b.as_str()));

    for dependency in dependencies {
        if !registry.packs.contains_key(dependency) {
            return Err(ContentPackRegistryError::MissingDependency {
                pack: pack_id.clone(),
                dependency: dependency.clone(),
            });
        }

        visit_pack(
            registry,
            dependency,
            visiting,
            visited,
            ordered_packs,
        )?;
    }

    visiting.remove(pack_id);
    visited.insert(pack_id.clone());
    ordered_packs.push(manifest);

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentPackRegistryError {
    InvalidManifest(ContentPackValidationError),
    DuplicatePack(ContentId),
    MissingDependency {
        pack: ContentId,
        dependency: ContentId,
    },
    DependencyCycle(ContentId),
    UnknownPack(ContentId),
}

impl Display for ContentPackRegistryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidManifest(error) => {
                write!(formatter, "Manifest non valido: {error}")
            }
            Self::DuplicatePack(id) => {
                write!(formatter, "Pacchetto duplicato: {}", id.as_str())
            }
            Self::MissingDependency { pack, dependency } => {
                write!(
                    formatter,
                    "Il pacchetto {} richiede la dipendenza mancante {}",
                    pack.as_str(),
                    dependency.as_str()
                )
            }
            Self::DependencyCycle(id) => {
                write!(
                    formatter,
                    "Rilevato un ciclo di dipendenze nel pacchetto {}",
                    id.as_str()
                )
            }
            Self::UnknownPack(id) => {
                write!(formatter, "Pacchetto sconosciuto: {}", id.as_str())
            }
        }
    }
}

impl std::error::Error for ContentPackRegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_manifest(
        id: &str,
        dependencies: &[&str],
    ) -> ContentPackManifest {
        ContentPackManifest {
            id: ContentId::new(id).unwrap(),
            name: id.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: dependencies
                .iter()
                .map(|dependency| ContentId::new(*dependency).unwrap())
                .collect(),
        }
    }

    #[test]
    fn registry_registers_valid_manifests() {
        let mut registry = ContentPackRegistry::new();

        registry
            .register(create_manifest("pack:classic", &[]))
            .unwrap();

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        let id = ContentId::new("pack:classic").unwrap();

        assert!(registry.get(&id).is_some());
    }

    #[test]
    fn registry_rejects_duplicate_packs() {
        let mut registry = ContentPackRegistry::new();
        let manifest = create_manifest("pack:classic", &[]);

        registry.register(manifest.clone()).unwrap();

        assert_eq!(
            registry.register(manifest),
            Err(ContentPackRegistryError::DuplicatePack(
                ContentId::new("pack:classic").unwrap()
            ))
        );
    }

    #[test]
    fn registry_resolves_dependencies_in_load_order() {
        let mut registry = ContentPackRegistry::new();

        registry
            .register(create_manifest(
                "pack:expansion",
                &["pack:classic"],
            ))
            .unwrap();

        registry
            .register(create_manifest("pack:classic", &[]))
            .unwrap();

        let order = registry.load_order().unwrap();

        let ordered_ids: Vec<&str> = order
            .iter()
            .map(|manifest| manifest.id.as_str())
            .collect();

        assert_eq!(
            ordered_ids,
            vec!["pack:classic", "pack:expansion"]
        );
    }

    #[test]
    fn registry_rejects_missing_dependencies() {
        let mut registry = ContentPackRegistry::new();

        registry
            .register(create_manifest(
                "pack:expansion",
                &["pack:classic"],
            ))
            .unwrap();

        assert_eq!(
            registry.load_order(),
            Err(ContentPackRegistryError::MissingDependency {
                pack: ContentId::new("pack:expansion").unwrap(),
                dependency: ContentId::new("pack:classic").unwrap(),
            })
        );
    }

    #[test]
    fn registry_rejects_dependency_cycles() {
        let mut registry = ContentPackRegistry::new();

        registry
            .register(create_manifest(
                "pack:classic",
                &["pack:expansion"],
            ))
            .unwrap();

        registry
            .register(create_manifest(
                "pack:expansion",
                &["pack:classic"],
            ))
            .unwrap();

        assert!(matches!(
            registry.load_order(),
            Err(ContentPackRegistryError::DependencyCycle(_))
        ));
    }

    #[test]
    fn registry_rejects_invalid_manifests() {
        let mut registry = ContentPackRegistry::new();

        let manifest = create_manifest("classic:content", &[]);

        assert_eq!(
            registry.register(manifest),
            Err(ContentPackRegistryError::InvalidManifest(
                ContentPackValidationError::InvalidPackIdNamespace
            ))
        );

        assert!(registry.is_empty());
    }
}
