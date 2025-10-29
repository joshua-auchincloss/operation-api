//! Helper types and utilities shared across resolution phases

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::Arc,
};

use crate::{
    Token,
    ast::{
        anonymous::AnonymousStruct,
        items::StructDef,
        strct::{Arg, Struct},
        ty::Type,
        union::Union,
    },
    ctx::{
        SourceSpanned,
        common::{FromNamedSource, WithSource},
    },
    defs::Spanned,
    tokens::{Brace, IdentToken, Repeated},
};

pub fn build_struct_def_from_anonymous(
    generated_name: String,
    anonymous: AnonymousStruct,
    source: PathBuf,
) -> FromNamedSource<StructDef> {
    let args = Repeated {
        values: anonymous.fields.value.values,
    };

    StructDef {
        meta: Vec::new(), // No meta or comments for generated structs
        def: Spanned::call_site(Struct {
            kw: Spanned::call_site(<Token![struct]>::new()),
            name: Spanned::call_site(IdentToken::new(generated_name)),
            brace: anonymous.brace.clone(), // reuse the brace tokens from anonymous struct
            args,
        }),
        end: Spanned::call_site(<Token![;]>::new()),
    }
    .with_source(source)
}

pub fn create_ident_type(name: String) -> Type {
    use crate::ast::ty::PathOrIdent;

    Type::Ident {
        to: PathOrIdent::Ident(Spanned::call_site(IdentToken::new(name))),
    }
}

#[derive(Clone)]
pub struct UnionRecord {
    pub union_ref: Arc<Spanned<Union>>,

    /// Name context stack at point of identification.
    /// Clone of NameContext.stack when union was encountered.
    pub context_stack: Vec<String>,

    pub in_oneof: bool,
    /// if in_oneof, the variant index for numeric suffix
    pub variant_index: Option<usize>,
}

impl UnionRecord {
    pub fn generate_name(&self) -> String {
        use convert_case::{Case, Casing};

        let base_name = self
            .context_stack
            .join("_")
            .to_case(Case::Pascal);

        if self.in_oneof {
            if let Some(idx) = self.variant_index {
                format!("{}{}", base_name, idx)
            } else {
                base_name
            }
        } else {
            base_name
        }
    }
}

#[derive(Clone)]
pub struct AliasEntry {
    pub target_type: Spanned<Type>,
    pub source: PathBuf,
    pub dependencies: Vec<String>,
}

pub struct UnionWorkingSet {
    fields: BTreeMap<String, (SourceSpanned<Arg>, Option<Spanned<Token![,]>>)>,
}

impl UnionWorkingSet {
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    pub fn merge_struct(
        &mut self,
        source: PathBuf,
        fields: Vec<crate::tokens::RepeatedItem<Arg, Token![,]>>,
    ) {
        for field in fields {
            let field_name = field
                .value
                .value
                .name
                .borrow_string()
                .clone();

            self.fields
                .entry(field_name)
                .or_insert_with(|| (field.value.with_source(source.clone()), field.sep.clone()));
        }
    }

    pub fn into_struct_def(
        self,
        generated_name: String,
        source: PathBuf,
        brace: Brace,
    ) -> FromNamedSource<StructDef> {
        let args: Vec<crate::tokens::RepeatedItem<_, Token![,]>> = self
            .fields
            .into_values()
            .map(|(from_source, sep)| {
                crate::tokens::RepeatedItem {
                    value: from_source.into_spanned(),
                    sep,
                }
            })
            .collect();

        StructDef {
            meta: Vec::new(),
            def: Spanned::call_site(Struct {
                kw: Spanned::call_site(<Token![struct]>::new()),
                name: Spanned::call_site(IdentToken::new(generated_name)),
                brace,
                args: Repeated { values: args },
            }),
            end: Spanned::call_site(<Token![;]>::new()),
        }
        .with_source(source)
    }
}

#[derive(Default)]
pub struct NameContext {
    pub stack: Vec<String>,
}

impl NameContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(
        &mut self,
        name: impl Into<String>,
    ) {
        self.stack.push(name.into());
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn generate_name(&self) -> String {
        use convert_case::{Case, Casing};
        self.stack.join("_").to_case(Case::Pascal)
    }
}

pub struct ResolvedTypeRef {
    pub original: Arc<Spanned<Type>>,
    pub resolved: Arc<Spanned<Type>>,
    pub path: super::super::paths::NamedItemContext,
}

impl ResolvedTypeRef {
    pub fn new(
        path: super::super::paths::NamedItemContext,
        typ: Arc<Spanned<Type>>,
    ) -> Self {
        Self {
            original: typ.clone(),
            resolved: typ.clone(),
            path,
        }
    }

    pub fn with_resolved(
        path: super::super::paths::NamedItemContext,
        original: Arc<Spanned<Type>>,
        resolved: Arc<Spanned<Type>>,
    ) -> Self {
        Self {
            path,
            original,
            resolved,
        }
    }
}

#[derive(Default)]
pub struct NamespaceResolution {
    pub anonymous_structs: Vec<SourceSpanned<StructDef>>,
    pub identified_unions: Vec<Spanned<UnionRecord>>,
    pub union_structs: Vec<SourceSpanned<StructDef>>,
    pub resolved_aliases: HashMap<String, Spanned<Type>>,
    pub versions: HashMap<String, Spanned<u32>>,
    pub errors: HashMap<String, Spanned<String>>,
    pub oneof_variant_counters: HashMap<String, usize>,
}

impl NamespaceResolution {
    pub fn new() -> Self {
        Self::default()
    }
}
