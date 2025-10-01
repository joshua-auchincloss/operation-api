use std::collections::HashMap;

use crate::defs::{
    Ident,
    meta::{Meta, Version},
};

#[allow(dead_code)]
pub fn insert_unique_ident_or_err_spanned<V>(
    ns: Vec<Ident>,
    tbl: &mut HashMap<Ident, V>,
    def: Ident,
    value: V,
    start: usize,
    end: usize,
) -> crate::Result<()> {
    match tbl.insert(def.clone(), value) {
        Some(..) => Err(crate::Error::conflict(ns, def).with_span(start, end)),
        None => Ok(()),
    }
}

pub fn extract_versions(metas: &[Meta]) -> Vec<&Version> {
    metas
        .iter()
        .filter_map(|m| {
            match m {
                Meta::Version(v) => Some(v),
                #[allow(unreachable_patterns)]
                _ => None,
            }
        })
        .collect()
}

pub fn detect_version_conflict(scope: &[&Version]) -> crate::Result<Option<usize>> {
    if scope.is_empty() {
        return Ok(None);
    }
    let first = scope[0].value;
    if scope.iter().any(|v| v.value != first) {
        let values = scope
            .iter()
            .map(|v| v.value)
            .collect::<Vec<_>>();
        let spans = scope
            .iter()
            .map(|v| (v.meta.span.start, v.meta.span.end))
            .collect::<Vec<_>>();
        let first_span = spans[0];
        return Err(
            crate::Error::VersionConflict { values, spans }.with_span(first_span.0, first_span.1)
        );
    }
    Ok(Some(first))
}
