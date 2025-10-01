use std::collections::HashMap;

use crate::defs::Ident;

pub fn insert_unique_ident_or_err<V>(
    ns: Vec<Ident>,
    tbl: &mut HashMap<Ident, V>,
    def: Ident,
    value: V,
) -> crate::Result<()> {
    match tbl.insert(def.clone(), value) {
        Some(..) => Err(crate::Error::conflict(ns, def)),
        None => Ok(()),
    }
}
