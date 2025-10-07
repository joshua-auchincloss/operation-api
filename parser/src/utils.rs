use std::collections::BTreeMap;

use crate::SpannedToken;

#[allow(unused)]
pub fn insert_unique_ident<V>(
    ns: SpannedToken![ident],
    tbl: &mut BTreeMap<SpannedToken![ident], V>,
    def: SpannedToken![ident],
    tag: &'static str,
    value: V,
) -> crate::Result<()> {
    match tbl.insert(def.clone(), value) {
        Some(..) => Err(crate::Error::conflict(ns, def, tag)),
        None => Ok(()),
    }
}

// pub fn detect_version_conflict(scope: &[&Version]) -> crate::Result<Option<usize>> {
//     if scope.is_empty() {
//         return Ok(None);
//     }
//     let first = scope[0].value;
//     if scope.iter().any(|v| v.value != first) {
//         let values = scope
//             .iter()
//             .map(|v| v.value)
//             .collect::<Vec<_>>();
//         let spans = scope
//             .iter()
//             .map(|v| (v.meta.span.start, v.meta.span.end))
//             .collect::<Vec<_>>();
//         let first_span = spans[0];
//         return Err(
//             crate::Error::VersionConflict { values, spans }.with_span(first_span.0, first_span.1)
//         );
//     }
//     Ok(Some(first))
// }
