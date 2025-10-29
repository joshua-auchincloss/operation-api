use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    SpannedToken,
    ast::{
        AstStream,
        comment::CommentStream,
        items::{Item, Items, NamespaceDef, UseDef},
        meta::{ErrorMeta, ItemMeta, ItemMetaItem, VersionMeta},
    },
    ctx::{
        RefOrItemContext,
        paths::{NamedItemContext, RefContext},
    },
    defs::Spanned,
    tokens::LexingError,
};

use super::{
    common::{FromNamedSource, NamespaceChild, WithSource},
    registry::TypeRegistry,
};

pub struct ImportRegistry {
    imports: BTreeMap<String, FromNamedSource<Spanned<RefOrItemContext>>>,
}

impl ImportRegistry {
    pub fn new() -> Self {
        Self {
            imports: BTreeMap::new(),
        }
    }

    // pub fn insert(
    //     &mut self,
    //     use_def: FromNamedSource<UseDef>,
    // ) {
    //     let import_kind = if use_def
    //         .value
    //         .def
    //         .path
    //         .value
    //         .is_single_segment()
    //     {
    //         ImportKind::Single(
    //             use_def
    //                 .value
    //                 .path
    //                 .value
    //                 .to_named_item_context(),
    //         )
    //     } else {
    //         ImportKind::Namespace(use_def.value.def.path.value.to_ref_context())
    //     };

    //     self.imports.insert(
    //         name.borrow_string().to_string(),
    //         use_def.map_value(|_| Spanned::new(import_kind, name.span())),
    //     );
    // }
}

pub struct NamespaceCtx {
    pub ctx: super::paths::RefContext,

    pub sources: BTreeMap<PathBuf, Arc<String>>,

    pub comments: FromNamedSource<CommentStream>,

    pub error: Option<FromNamedSource<ErrorMeta>>,

    pub version: Option<FromNamedSource<VersionMeta>>,

    pub namespace: FromNamedSource<NamespaceDef>,

    pub imports: Vec<FromNamedSource<RefOrItemContext>>,

    pub children: BTreeMap<NamedItemContext, FromNamedSource<NamespaceChild>>,

    /// Reference to the global type registry.
    /// Shared across all namespaces in the compilation context.
    /// Set to None during initial parsing, populated by CompileCtx.
    pub(crate) registry: TypeRegistry,
}

impl NamespaceCtx {
    /// Get the type registry, panicking if not set.
    /// Should only be called during type resolution after CompileCtx has populated it.
    pub(crate) fn registry(&self) -> &TypeRegistry {
        &self.registry
    }

    pub(crate) fn report_error(
        err: crate::Error,
        path: &Path,
        source: &Arc<String>,
    ) -> miette::Report {
        err.with_source(path.to_path_buf(), Arc::clone(source))
            .to_report(None, None, None)
    }

    pub async fn from_file(
        ctx: RefContext,
        fs: &dyn operation_api_fs::FileSystem,
        path: impl AsRef<Path>,
        registry: TypeRegistry,
    ) -> crate::Result<Self> {
        let path = path.as_ref();
        let source = fs.read_to_string(path).await?;

        let source = Arc::new(source);
        let mut tt = crate::tokens::tokenize(&source).map_err(|e: LexingError| {
            crate::Error::from(e).with_source(path.to_path_buf(), Arc::clone(&source))
        })?;

        let ast = AstStream::from_tokens_with(path, &mut tt)?;

        Self::from_ast_stream(ctx, ast, path.to_path_buf(), source, registry)
    }

    /// Parse multiple files that belong to the same namespace.
    /// Only one file should contain the namespace declaration.
    ///
    /// Files are read in parallel for better performance, then processed sequentially
    /// to maintain deterministic ordering.
    pub async fn load_files(
        ref_ctx: RefContext,
        fs: &dyn operation_api_fs::FileSystem,
        paths: &[PathBuf],
        required_namespace: Option<&SpannedToken![ident]>,
        registry: TypeRegistry,
    ) -> crate::Result<Self> {
        if paths.is_empty() {
            return Err(crate::Error::EmptyFileList);
        }

        // Parallelize file reading using futures
        use futures_util::future::join_all;

        let read_futures: Vec<_> = paths
            .iter()
            .map(|path| {
                async move {
                    let source = fs.read_to_string(path).await?;
                    Ok::<(PathBuf, String), operation_api_fs::Error>((path.clone(), source))
                }
            })
            .collect();

        let file_contents = join_all(read_futures).await;

        // Process files sequentially to maintain deterministic order
        let mut namespace_ctx: Option<NamespaceCtx> = None;
        let mut found_namespace_decl = false;

        for result in file_contents {
            let (path, source_str) = result?;
            let source = Arc::new(source_str);

            let mut tt = crate::tokens::tokenize(&source).map_err(|e: LexingError| {
                crate::Error::from(e).with_source(path.clone(), Arc::clone(&source))
            })?;

            let ast = AstStream::from_tokens_with(&path, &mut tt)?;

            if let Some(ctx) = &mut namespace_ctx {
                // Merge into existing context
                ctx.merge_ast_stream(
                    ast,
                    path,
                    source,
                    &mut found_namespace_decl,
                    registry.clone(),
                )?;
            } else {
                let ctx = Self::from_ast_stream(
                    ref_ctx.clone(),
                    ast,
                    path.clone(),
                    Arc::clone(&source),
                    registry.clone(),
                )?;
                found_namespace_decl = ctx.namespace.source == path;

                // Validate namespace name if required
                if let Some(required) = required_namespace
                    && ctx.namespace.value.def.name.borrow_string() != required.borrow_string()
                {
                    let err = crate::Error::NamespaceMismatch {
                        expected: required.borrow_string().to_string(),
                        found: ctx
                            .namespace
                            .value
                            .def
                            .name
                            .borrow_string()
                            .to_string(),
                    }
                    .with_source(path.clone(), source);
                    return Err(err);
                }

                namespace_ctx = Some(ctx);
            }
        }

        namespace_ctx.ok_or_else(|| crate::Error::FailedToCreateNamespaceCtx)
    }

    fn handle_use(
        ctx: &RefContext,
        imports: &mut Vec<FromNamedSource<RefOrItemContext>>,
        use_def: UseDef,
        path: PathBuf,
    ) {
        imports.extend(
            use_def
                .def
                .value
                .path
                .qualified_paths(ctx.package.clone())
                .into_iter()
                .map(|p| p.with_source(path.clone())),
        );
    }

    fn from_ast_stream(
        ctx: RefContext,
        ast: AstStream,
        path: PathBuf,
        source: Arc<String>,
        registry: TypeRegistry,
    ) -> crate::Result<Self> {
        let mut sources = BTreeMap::new();
        sources.insert(path.clone(), Arc::clone(&source));

        let module_comments = ast.module_comments;

        let mut namespace: Option<FromNamedSource<NamespaceDef>> = None;
        let mut imports = Vec::new();
        let mut children = BTreeMap::new();
        let mut version: Option<FromNamedSource<VersionMeta>> = None;
        let mut error: Option<FromNamedSource<ErrorMeta>> = None;

        Self::extract_meta_items(&[&ast.module_meta], &path, &mut version, &mut error)?;

        let ast_p = path.clone();
        let ast_s = source.clone();
        // process items
        for item in ast.nodes {
            match item.value {
                Items::Namespace(ns_def) => {
                    // extract meta from namespace item first (before consuming ns_def)
                    Self::extract_meta_items(&ns_def.meta(), &path, &mut version, &mut error)
                        .map_err(|err| err.with_source(ast_p.clone(), ast_s.clone()))?;

                    if let Some(ref existing) = namespace {
                        // allow multiple declarations in the same directory with the same name
                        if ns_def.def.name.borrow_string()
                            != existing.value.def.name.borrow_string()
                        {
                            let err = crate::Error::ns_dir(
                                &existing.value.def.name,
                                &ns_def.def.name,
                                path.clone(),
                            );
                            return Err(err.with_source(path, source));
                        }
                        // same name - just continue, don't replace
                    } else {
                        namespace = Some(ns_def.with_source(path.clone()));
                    }
                },
                Items::Use(use_def) => {
                    Self::handle_use(&ctx, &mut imports, use_def, path.clone());
                },
                Items::SpannedNamespace(inner_ns) => {
                    let ctx = ctx.enter(inner_ns.def.name.borrow_string());

                    Self::process_spanned_namespace(
                        ctx,
                        &mut children,
                        inner_ns,
                        &path,
                        &source,
                        registry.clone(),
                    )?;
                },
                Items::OneOf(def) => {
                    let name = def.def.name.clone();
                    let ns_name = namespace
                        .as_ref()
                        .ok_or_else(|| {
                            crate::Error::NsNotDeclared
                                .with_source(path.clone(), Arc::clone(&source))
                        })?
                        .value
                        .def
                        .name
                        .clone();

                    Self::insert_typed_child(
                        &ctx,
                        &mut children,
                        ns_name,
                        &name,
                        || NamespaceChild::OneOf(def),
                        &path,
                        &source,
                        "oneof",
                    )?;
                },
                Items::Enum(def) => {
                    let name = def.def.name().clone();
                    let ns_name = namespace
                        .as_ref()
                        .ok_or_else(|| {
                            crate::Error::NsNotDeclared
                                .with_source(path.clone(), Arc::clone(&source))
                        })?
                        .value
                        .def
                        .name
                        .clone();

                    Self::insert_typed_child(
                        &ctx,
                        &mut children,
                        ns_name,
                        &name,
                        || NamespaceChild::Enum(def),
                        &path,
                        &source,
                        "enum",
                    )?;
                },
                Items::Struct(def) => {
                    let name = def.def.name.clone();
                    let ns_name = namespace
                        .as_ref()
                        .ok_or_else(|| {
                            crate::Error::NsNotDeclared
                                .with_source(path.clone(), Arc::clone(&source))
                        })?
                        .value
                        .def
                        .name
                        .clone();

                    Self::insert_typed_child(
                        &ctx,
                        &mut children,
                        ns_name,
                        &name,
                        || NamespaceChild::Struct(def),
                        &path,
                        &source,
                        "struct",
                    )?;
                },
                Items::Type(def) => {
                    let name = def.def.name.clone();
                    let ns_name = namespace
                        .as_ref()
                        .ok_or_else(|| {
                            crate::Error::NsNotDeclared
                                .with_source(path.clone(), Arc::clone(&source))
                        })?
                        .value
                        .def
                        .name
                        .clone();

                    Self::insert_typed_child(
                        &ctx,
                        &mut children,
                        ns_name,
                        &name,
                        || NamespaceChild::Type(def),
                        &path,
                        &source,
                        "type",
                    )?;
                },
                Items::Error(def) => {
                    let name = def.def.name.clone();
                    let ns_name = namespace
                        .as_ref()
                        .ok_or_else(|| {
                            crate::Error::NsNotDeclared
                                .with_source(path.clone(), Arc::clone(&source))
                        })?
                        .value
                        .def
                        .name
                        .clone();

                    Self::insert_typed_child(
                        &ctx,
                        &mut children,
                        ns_name,
                        &name,
                        || NamespaceChild::Error(def),
                        &path,
                        &source,
                        "error",
                    )?;
                },
                Items::Operation(def) => {
                    let name = def.def.name.clone();
                    let ns_name = namespace
                        .as_ref()
                        .ok_or_else(|| {
                            crate::Error::NsNotDeclared
                                .with_source(path.clone(), Arc::clone(&source))
                        })?
                        .value
                        .def
                        .name
                        .clone();

                    Self::insert_typed_child(
                        &ctx,
                        &mut children,
                        ns_name,
                        &name,
                        || NamespaceChild::Operation(def),
                        &path,
                        &source,
                        "operation",
                    )?;
                },
            }
        }

        let namespace = namespace.ok_or_else(|| {
            crate::Error::NsNotDeclared.with_source(path.clone(), Arc::clone(&source))
        })?;

        Ok(Self {
            ctx,
            sources,
            comments: module_comments.with_source(path.clone()),
            error,
            version,
            namespace,
            imports,
            children,
            registry,
        })
    }

    /// merge an additional AstStream into this namespace context
    fn merge_ast_stream(
        &mut self,
        ast: AstStream,
        path: PathBuf,
        source: Arc<String>,
        found_namespace_decl: &mut bool,
        registry: TypeRegistry,
    ) -> crate::Result<()> {
        self.sources
            .insert(path.clone(), Arc::clone(&source));

        // process module-level meta
        Self::extract_meta_items(
            &[&ast.module_meta],
            &path,
            &mut self.version,
            &mut self.error,
        )?;

        // process items
        for item in ast.nodes {
            match item.value {
                Items::Namespace(ns_def) => {
                    // verify namespace name matches existing
                    if ns_def.def.name.borrow_string()
                        != self.namespace.value.def.name.borrow_string()
                    {
                        let err = crate::Error::ns_dir(
                            &self.namespace.value.def.name,
                            &ns_def.def.name,
                            path.clone(),
                        );
                        return Err(err.with_source(path, source));
                    }

                    *found_namespace_decl = true;
                    Self::extract_meta_items(
                        &ns_def.meta(),
                        &path,
                        &mut self.version,
                        &mut self.error,
                    )?;
                },
                Items::Use(use_def) => {
                    Self::handle_use(&self.ctx, &mut self.imports, use_def, path.clone());
                },
                Items::SpannedNamespace(inner_ns) => {
                    let ctx = self
                        .ctx
                        .enter(inner_ns.def.name.borrow_string());

                    Self::process_spanned_namespace(
                        ctx,
                        &mut self.children,
                        inner_ns,
                        &path,
                        &source,
                        registry.clone(),
                    )?;
                },
                Items::OneOf(def) => {
                    let name = def.def.name.clone();
                    Self::insert_typed_child(
                        &self.ctx,
                        &mut self.children,
                        self.namespace.value.def.name.clone(),
                        &name,
                        || NamespaceChild::OneOf(def),
                        &path,
                        &source,
                        "oneof",
                    )?;
                },
                Items::Enum(def) => {
                    let name = def.def.value.name().clone();
                    Self::insert_typed_child(
                        &self.ctx,
                        &mut self.children,
                        self.namespace.value.def.name.clone(),
                        &name,
                        || NamespaceChild::Enum(def),
                        &path,
                        &source,
                        "enum",
                    )?;
                },
                Items::Struct(def) => {
                    let name = def.def.name.clone();
                    Self::insert_typed_child(
                        &self.ctx,
                        &mut self.children,
                        self.namespace.value.def.name.clone(),
                        &name,
                        || NamespaceChild::Struct(def),
                        &path,
                        &source,
                        "struct",
                    )?;
                },
                Items::Type(def) => {
                    let name = def.def.name.clone();
                    Self::insert_typed_child(
                        &self.ctx,
                        &mut self.children,
                        self.namespace.value.def.name.clone(),
                        &name,
                        || NamespaceChild::Type(def),
                        &path,
                        &source,
                        "type",
                    )?;
                },
                Items::Error(def) => {
                    let name = def.def.name.clone();
                    Self::insert_typed_child(
                        &self.ctx,
                        &mut self.children,
                        self.namespace.value.def.name.clone(),
                        &name,
                        || NamespaceChild::Error(def),
                        &path,
                        &source,
                        "error",
                    )?;
                },
                Items::Operation(def) => {
                    let name = def.def.name.clone();
                    Self::insert_typed_child(
                        &self.ctx,
                        &mut self.children,
                        self.namespace.value.def.name.clone(),
                        &name,
                        || NamespaceChild::Operation(def),
                        &path,
                        &source,
                        "operation",
                    )?;
                },
            }
        }

        Ok(())
    }

    fn insert_typed_child<F>(
        ctx: &RefContext,
        children: &mut BTreeMap<NamedItemContext, FromNamedSource<NamespaceChild>>,
        namespace_name: SpannedToken![ident],
        name: &SpannedToken![ident],
        constructor: F,
        path: &Path,
        source: &Arc<String>,
        type_tag: &'static str,
    ) -> crate::Result<()>
    where
        F: FnOnce() -> NamespaceChild, {
        let qual = ctx.item(name.clone());
        crate::utils::insert_unique_ident(
            namespace_name,
            children,
            qual,
            type_tag,
            constructor().with_source(path.to_path_buf()),
        )
        .map_err(|e| {
            let span = name.span();
            e.with_span(span.start, span.end)
                .with_source(path.to_path_buf(), Arc::clone(source))
        })
    }

    pub fn process_spanned_namespace(
        ctx: RefContext,
        children: &mut BTreeMap<NamedItemContext, FromNamedSource<NamespaceChild>>,
        inner_ns: Item<crate::ast::namespace::SpannedNamespace>,
        path: &Path,
        source: &Arc<String>,
        registry: TypeRegistry,
    ) -> crate::Result<()> {
        let name = inner_ns.def.name.clone();
        let ast = inner_ns.def.value.ast.value;

        let nested_ctx = Self::from_ast_stream(
            ctx.clone(),
            ast,
            path.to_path_buf(),
            Arc::clone(source),
            registry,
        )?;

        let child = NamespaceChild::Namespace(Box::new(nested_ctx));

        let qual_name = ctx.item(name.clone());

        if children.contains_key(&qual_name) {
            let span = name.span();
            let err = crate::Error::DuplicateNamespace {
                name: name.borrow_string().to_string(),
            }
            .with_span(span.start, span.end);
            return Err(err.with_source(path.to_path_buf(), Arc::clone(source)));
        }

        children.insert(qual_name, child.with_source(path.to_path_buf()));
        Ok(())
    }

    fn extract_meta_items(
        meta_vec: &[&Spanned<ItemMeta>],
        path: &Path,
        version: &mut Option<FromNamedSource<VersionMeta>>,
        error: &mut Option<FromNamedSource<ErrorMeta>>,
    ) -> crate::Result<()> {
        for meta_spanned in meta_vec {
            for meta_item in &meta_spanned.value.meta {
                match meta_item {
                    ItemMetaItem::Version(v) => {
                        if version.is_some() {
                            return Err(crate::Error::DuplicateMetaAttribute {
                                attribute: "version".to_string(),
                                path: path.to_path_buf(),
                            });
                        }
                        *version = Some(v.clone().with_source(path.to_path_buf()));
                    },
                    ItemMetaItem::Error(e) => {
                        if error.is_some() {
                            return Err(crate::Error::DuplicateMetaAttribute {
                                attribute: "error".to_string(),
                                path: path.to_path_buf(),
                            });
                        }
                        *error = Some(e.clone().with_source(path.to_path_buf()));
                    },
                }
            }
        }
        Ok(())
    }
}
