use std::{collections::BTreeSet, path::PathBuf};

use crate::generate::Source;

pub fn walk(source: &Source) -> super::Result<Vec<PathBuf>> {
    let mut found = BTreeSet::new();
    for include in &source.include {
        for p in glob::glob(&include).unwrap() {
            found.insert(p?);
        }
    }

    let mut match_exp = vec![];
    for exclude in &source.exclude {
        let exp = glob::Pattern::new(&exclude).unwrap();
        match_exp.push(exp);
    }

    Ok(found
        .into_iter()
        .filter(|it| {
            let remove = match_exp
                .iter()
                .any(|re| re.matches_path(&it));

            if remove {
                tracing::info!(
                    "removing '{}' from targets due to exclusion rule",
                    it.display()
                )
            }

            !remove
        })
        .map(|it| {
            tracing::info!("including '{}' in targets", it.display());
            it
        })
        .collect())
}
