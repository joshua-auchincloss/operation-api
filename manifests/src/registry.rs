#[derive(serde::Deserialize, serde::Serialize)]
pub struct PackageUpload {
    pub package: super::lock::Lockfile,
    pub source: operation_api_fs::memory::MemoryFileSystem,
}
