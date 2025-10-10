use std::path::PathBuf;

use crate::generate::Source;

pub fn walk(source: &Source) -> super::Result<Vec<PathBuf>> {
    Ok(operation_api_manifests::files::match_paths(
        &source.include,
        &source.exclude,
    )?)
}
