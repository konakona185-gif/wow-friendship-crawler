mod ability;
mod database;
mod damage_school;
mod effect;
mod id;
mod loader;
mod metadata;
mod pack;
mod registry;
mod talent;

pub use ability::{
    AbilityActivation,
    AbilityDefinition,
    AbilityRange,
    AbilityTargeting,
    AbilityValidationError,
    ResourceCost,
};

pub use database::{
    ContentDatabase,
    ContentDatabaseError,
};

pub use damage_school::{
    DamageSchoolDefinition,
    DamageSchoolMitigation,
    DamageSchoolValidationError,
};

pub use effect::{
    EffectDefinition,
    EffectKind,
    EffectMagnitude,
    EffectScaling,
    EffectTarget,
    EffectTrigger,
    EffectValidationError,
};

pub use id::{
    ContentId,
    ContentIdError,
};

pub use loader::{
    ContentPackData,
    ContentPackLoader,
    ContentPackLoaderError,
};

pub use metadata::{
    ContentMetadata,
    ContentMetadataError,
};

pub use pack::{
    ContentPackManifest,
    ContentPackValidationError,
};

pub use registry::{
    ContentPackRegistry,
    ContentPackRegistryError,
};

pub use talent::{
    TalentDefinition,
    TalentPosition,
    TalentPrerequisite,
    TalentRankDefinition,
    TalentTreeDefinition,
    TalentValidationError,
};
