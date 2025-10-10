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
