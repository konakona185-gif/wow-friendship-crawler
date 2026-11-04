use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use bevy::prelude::Resource;

use crate::content::ContentId;

use super::effect_execution::{
    EffectExecutionContext,
    EffectExecutionError,
};
use super::ResolvedEffect;

pub trait CustomEffectHandler: Send + Sync + 'static {
    fn execute(
        &self,
        effect: &ResolvedEffect,
        context: &mut EffectExecutionContext<'_>,
    ) -> Result<(), EffectExecutionError>;
}

impl<F> CustomEffectHandler for F
where
    F: for<'a> Fn(
            &ResolvedEffect,
            &mut EffectExecutionContext<'a>,
        ) -> Result<(), EffectExecutionError>
        + Send
        + Sync
        + 'static,
{
    fn execute(
        &self,
        effect: &ResolvedEffect,
        context: &mut EffectExecutionContext<'_>,
    ) -> Result<(), EffectExecutionError> {
        self(effect, context)
    }
}

#[derive(Resource, Default)]
pub struct CustomEffectHandlers {
    handlers: HashMap<ContentId, Box<dyn CustomEffectHandler>>,
}

impl CustomEffectHandlers {
    pub fn register<H>(
        &mut self,
        id: ContentId,
        handler: H,
    ) -> Result<(), CustomEffectRegistryError>
    where
        H: CustomEffectHandler,
    {
        self.register_boxed(id, Box::new(handler))
    }

    pub fn register_boxed(
        &mut self,
        id: ContentId,
        handler: Box<dyn CustomEffectHandler>,
    ) -> Result<(), CustomEffectRegistryError> {
        if self.handlers.contains_key(&id) {
            return Err(
                CustomEffectRegistryError::DuplicateHandler(id),
            );
        }

        self.handlers.insert(id, handler);

        Ok(())
    }

    pub fn unregister(&mut self, id: &ContentId) -> bool {
        self.handlers.remove(id).is_some()
    }

    pub fn contains(&self, id: &ContentId) -> bool {
        self.handlers.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    pub(crate) fn get(
        &self,
        id: &ContentId,
    ) -> Option<&dyn CustomEffectHandler> {
        self.handlers.get(id).map(Box::as_ref)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CustomEffectRegistryError {
    DuplicateHandler(ContentId),
}

impl Display for CustomEffectRegistryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateHandler(id) => {
                write!(
                    formatter,
                    "Esiste già un handler registrato con l'identificativo: {}",
                    id.as_str()
                )
            }
        }
    }
}

impl std::error::Error for CustomEffectRegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler;

    impl CustomEffectHandler for TestHandler {
        fn execute(
            &self,
            _effect: &ResolvedEffect,
            _context: &mut EffectExecutionContext<'_>,
        ) -> Result<(), EffectExecutionError> {
            Err(EffectExecutionError::UnsupportedOperation)
        }
    }

    fn handler_id() -> ContentId {
        ContentId::new(
            "classic_1_12_1:handler.test",
        )
        .unwrap()
    }

    #[test]
    fn registry_starts_empty() {
        let handlers = CustomEffectHandlers::default();

        assert!(handlers.is_empty());
        assert_eq!(handlers.len(), 0);
        assert!(!handlers.contains(&handler_id()));
    }

    #[test]
    fn registry_registers_handler() {
        let mut handlers = CustomEffectHandlers::default();
        let id = handler_id();

        assert!(
            handlers.register(id.clone(), TestHandler).is_ok()
        );

        assert!(handlers.contains(&id));
        assert_eq!(handlers.len(), 1);
        assert!(handlers.get(&id).is_some());
    }

    #[test]
    fn registry_rejects_duplicate_handler_ids() {
        let mut handlers = CustomEffectHandlers::default();
        let id = handler_id();

        handlers
            .register(id.clone(), TestHandler)
            .unwrap();

        assert_eq!(
            handlers.register(id.clone(), TestHandler),
            Err(CustomEffectRegistryError::DuplicateHandler(id.clone()))
        );

        assert_eq!(handlers.len(), 1);
        assert!(handlers.get(&id).is_some());
    }

    #[test]
    fn registry_unregisters_handler() {
        let mut handlers = CustomEffectHandlers::default();
        let id = handler_id();

        handlers
            .register(id.clone(), TestHandler)
            .unwrap();

        assert!(handlers.unregister(&id));
        assert!(!handlers.contains(&id));
        assert!(handlers.is_empty());
    }

    #[test]
    fn registry_returns_false_when_unregistering_missing_handler() {
        let mut handlers = CustomEffectHandlers::default();

        assert!(!handlers.unregister(&handler_id()));
        assert!(handlers.is_empty());
    }

    #[test]
    fn registry_accepts_closure_handlers() {
        let mut handlers = CustomEffectHandlers::default();
        let id = handler_id();

        let handler = |
            _effect: &ResolvedEffect,
            _context: &mut EffectExecutionContext<'_>,
        | {
            Err(EffectExecutionError::UnsupportedOperation)
        };

        assert!(handlers.register(id.clone(), handler).is_ok());
        assert!(handlers.contains(&id));
    }

    #[test]
    fn closure_handler_can_return_success() {
        let mut handlers = CustomEffectHandlers::default();
        let id = handler_id();

        let handler = |
            _effect: &ResolvedEffect,
            _context: &mut EffectExecutionContext<'_>,
        | {
            Ok(())
        };

        assert!(handlers.register(id.clone(), handler).is_ok());
        assert!(handlers.get(&id).is_some());
    }
}
