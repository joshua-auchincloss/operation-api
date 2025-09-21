use std::{collections::BTreeSet, path::PathBuf};

use crate::generate::Source;

pub fn walk(source: &Source) -> super::Result<Vec<PathBuf>> {
    let mut found = BTreeSet::new();
    for include in &source.include {
        for p in glob::glob(include)? {
            found.insert(p?);
        }
    }

    let mut match_exp = vec![];
    for exclude in &source.exclude {
        match_exp.push(glob::Pattern::new(exclude)?);
    }

    let mut out: Vec<PathBuf> = found
        .into_iter()
        .filter(|it| {
            let remove = match_exp
                .iter()
                .any(|re| re.matches_path(it));

            if remove {
                tracing::info!(
                    "removing '{}' from targets due to exclusion rule",
                    it.display()
                )
            }

            !remove
        })
        .inspect(|it| {
            tracing::info!("including '{}' in targets", it.display());
        })
        .collect();

    out.sort();

    Ok(out)
}
