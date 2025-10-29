use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use convert_case::{Case, Casing};

use crate::{
    SpannedToken,
    ast::{AstStream, items::Items},
    ctx::registry::TypeRegistry,
    tokens::LexingError,
};

use super::namespace::NamespaceCtx;

pub struct SchemaCtx {
    pub root_path: PathBuf,
    pub package: operation_api_manifests::package::PackageManifest,
    pub namespaces: BTreeMap<String, NamespaceCtx>,
    pub registry: TypeRegistry,
}

impl SchemaCtx {
    pub async fn from_path(
        fs: &dyn operation_api_fs::FileSystem,
        root_path: impl AsRef<Path>,
        registry: TypeRegistry,
    ) -> crate::Result<Self> {
        use operation_api_manifests::config::NewForNamed;

        let root_path = root_path.as_ref();

        // load package manifest
        let package = operation_api_manifests::package::PackageManifest::new(fs, root_path)?;

        // find lib.pld
        let lib_path = root_path.join("schema").join("lib.pld");

        // Note: We need to check if file exists. This might require adding an exists() method
        // to the FileSystem trait if not already available. For now, we attempt to read and
        // handle the error.

        // parse lib.pld to get root namespace and use statements
        let lib_source = fs
            .read_to_string(&lib_path)
            .await
            .map_err(|e| {
                // If file doesn't exist, convert to MissingLibError
                match e {
                    operation_api_fs::Error::IoError(_) => crate::Error::MissingLibError,
                    _ => crate::Error::from(e),
                }
            })?;

        let root_ctx =
            super::paths::RefContext::new(package.package.name.to_case(Case::Snake), vec![]);

        let lib_source = Arc::new(lib_source);
        let mut tt = crate::tokens::tokenize(&lib_source).map_err(|e: LexingError| {
            crate::Error::from(e).with_source(lib_path.clone(), Arc::clone(&lib_source))
        })?;

        let lib_ast = AstStream::from_tokens_with(&lib_path, &mut tt)?;

        // extract namespace and use statements from lib.pld
        let mut lib_namespace: Option<SpannedToken![ident]> = None;
        let mut use_statements: Vec<_> = Vec::new();

        let mut namespaces = BTreeMap::new();

        for item in &lib_ast.nodes {
            match &item.value {
                Items::Namespace(ns) => {
                    if lib_namespace.is_some() {
                        let def_span = ns.def.span();
                        return Err(crate::Error::NsConflict
                            .with_span(def_span.start, def_span.end)
                            .with_source(lib_path, lib_source.clone()));
                    }
                    lib_namespace = Some(ns.def.name.clone());
                },
                Items::Use(use_def) => {
                    use_statements.push(use_def.def.root_ident().to_string());
                },
                Items::SpannedNamespace(..) => {
                    // todo: handle spanned namespaces here
                },
                _ => {
                    return Err(
                        crate::Error::LibPldInvalidItem.with_source(lib_path, lib_source.clone())
                    );
                },
            }
        }

        let schema_dir = root_path.join("schema");

        for use_name in use_statements {
            let ctx = root_ctx.enter(&use_name);
            let single_file = schema_dir.join(format!("{}.pld", use_name));
            let dir_path = schema_dir.join(&use_name);

            let files_to_load = if fs.exists_sync(&single_file) {
                vec![single_file]
            } else {
                let include = vec![format!("{}/**/*.pld", dir_path.display())];

                fs.find_glob(&include, &package.files.exclude)
                    .map_err(crate::Error::from)?
            };

            if files_to_load.is_empty() {
                return Err(crate::Error::UsePathNotFound {
                    name: use_name.clone(),
                }
                .with_source(lib_path.clone(), Arc::clone(&lib_source)));
            }

            let ns_ctx = NamespaceCtx::load_files(ctx, fs, &files_to_load, None, registry.clone())
                .await
                .map_err(|e| e.with_source(lib_path.clone(), Arc::clone(&lib_source)))?;

            // verify namespace name matches use statement
            let ns_name = ns_ctx
                .namespace
                .value
                .def
                .name
                .borrow_string()
                .to_string();
            if ns_name != use_name {
                return Err(crate::Error::NamespaceMismatch {
                    expected: use_name.clone(),
                    found: ns_name,
                }
                .with_source(lib_path.clone(), Arc::clone(&lib_source)));
            }

            namespaces.insert(use_name, ns_ctx);
        }

        Ok(Self {
            root_path: root_path.to_path_buf(),
            package,
            namespaces,
            registry,
        })
    }

    /// Get a namespace by name
    pub fn get_namespace(
        &self,
        name: &str,
    ) -> Option<&NamespaceCtx> {
        self.namespaces.get(name)
    }

    /// Get a namespace by name (mutable)
    pub fn get_namespace_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut NamespaceCtx> {
        self.namespaces.get_mut(name)
    }
}
