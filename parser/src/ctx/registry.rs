use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    SpannedToken,
    ast::ty::PathOrIdent,
    ctx::{Definition, NamespaceCtx, ResolvedType, paths::NamedItemContext},
    defs::{Span, Spanned},
    tokens::ToTokens,
};

#[derive(Clone)]
pub struct TypeRegistry {
    inner: Arc<Mutex<HashMap<super::paths::NamedItemContext, Spanned<ResolvedType>>>>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::default(),
        }
    }

    fn with_lock<F, R>(
        &self,
        f: F,
    ) -> crate::Result<R>
    where
        F: FnOnce(&HashMap<super::paths::NamedItemContext, Spanned<ResolvedType>>) -> R, {
        self.inner
            .lock()
            .map(|guard| f(&guard))
            .map_err(|_| {
                crate::Error::InternalError {
                    message: "TypeRegistry lock poisoned".into(),
                }
            })
    }

    fn with_lock_mut<F, R>(
        &self,
        f: F,
    ) -> crate::Result<R>
    where
        F: FnOnce(&mut HashMap<super::paths::NamedItemContext, Spanned<ResolvedType>>) -> R, {
        self.inner
            .lock()
            .map(|mut guard| f(&mut guard))
            .map_err(|_| {
                crate::Error::InternalError {
                    message: "TypeRegistry lock poisoned".into(),
                }
            })
    }

    pub fn register(
        &self,
        context: &super::paths::RefContext,
        name: &SpannedToken![ident],
        kind: Definition,
        span: Span,
    ) -> crate::Result<()> {
        self.with_lock_mut(|inner| {
            let span = span.span();

            let path = context.item(name.clone());

            if inner.contains_key(&path) {
                return Err(crate::Error::DuplicateType {
                    name: path.display(),
                }
                .with_span(span.start, span.end));
            }
            inner.insert(
                path.clone(),
                Spanned::new(
                    span.start,
                    span.end,
                    ResolvedType {
                        kind,
                        qualified_path: path,
                    },
                ),
            );
            Ok(())
        })?
    }

    #[tracing::instrument(
        level = "TRACE",
        target = "type-registry",
        skip(self, context, reference, ns),
        fields(
            context = context.display(),
            reference = %reference.display(),
            ns = ns.ctx.display(),
        )
    )]
    pub fn resolve(
        &self,
        context: &super::paths::RefContext,
        reference: &PathOrIdent,
        ns: &NamespaceCtx,
    ) -> Option<Spanned<ResolvedType>> {
        self.with_lock(|inner| {
            let uses_candidates =
                super::graph::extract::TypeExtractor::generate_candidates(reference, context, ns);
            for candidate in &uses_candidates {
                tracing::trace!(
                    candidate = candidate.display(),
                    candidate_path = candidate.path().display(),
                    "attempting candidate"
                );
                if let Some(found) = inner.get(candidate) {
                    return Some(found.clone());
                }
            }
            None
        })
        .ok()
        .flatten()
    }

    /// Resolve a type if valid, returns None if not found
    /// This is an alias for `resolve` with a more explicit name
    pub fn resolve_if_valid(
        &self,
        context: &super::paths::RefContext,
        reference: &PathOrIdent,
        ns: &NamespaceCtx,
    ) -> Option<Spanned<ResolvedType>> {
        self.resolve(context, reference, ns)
    }

    /// Resolve a type or return an error if not found
    pub fn resolve_required(
        &self,
        context: &super::paths::RefContext,
        reference: &PathOrIdent,
        ns: &NamespaceCtx,
    ) -> crate::Result<Spanned<ResolvedType>> {
        self.resolve(context, reference, ns)
            .ok_or_else(|| {
                let type_name = match reference {
                    PathOrIdent::Ident(name) => name.borrow_string().clone(),
                    PathOrIdent::Path(path) => path.display(),
                };
                crate::Error::UndefinedType { name: type_name }
            })
    }

    /// Check if a type reference is valid
    pub fn is_valid(
        &self,
        context: &super::paths::RefContext,
        reference: &PathOrIdent,
        ns: &NamespaceCtx,
    ) -> bool {
        self.resolve(context, reference, ns)
            .is_some()
    }

    /// Get all registered types (for debugging/introspection)
    pub fn all_types(&self) -> Vec<(NamedItemContext, Span)> {
        self.with_lock(|inner| {
            inner
                .iter()
                .map(|(path, spanned)| (path.clone(), spanned.span.clone()))
                .collect()
        })
        .unwrap_or_default()
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        // Simple test: verify registry can be created
        let registry = TypeRegistry::new();
        assert_eq!(
            registry.all_types().len(),
            0,
            "New registry should be empty"
        );
    }

    #[test]
    fn test_registry_thread_safety() {
        use std::thread;

        // Test that TypeRegistry can be shared across threads
        let registry = TypeRegistry::new();
        let mut handles = vec![];

        for i in 0..5 {
            let registry_clone = registry.clone();
            let handle = thread::spawn(move || {
                // Just verify we can access the registry from multiple threads
                let _types = registry_clone.all_types();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }
}
