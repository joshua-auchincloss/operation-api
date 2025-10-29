use operation_api_fs::{FileSystem, memory::MemoryFileSystem};
use operation_api_parser::ctx::CompileCtx;
use std::{path::Path, sync::Arc};

#[derive(Clone, Debug)]
pub enum Tag {
    Smoke,
    Soundness,
    Validations,
    Lockfile,
    Operation,
    Imports,
    Namespace,
    Dependencies,
    VersionResolution,
}

#[derive(Clone, Debug)]
pub struct TestMetadata {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub expect_pass: bool,
    pub tags: Vec<Tag>,
}

/// Test result tracking
#[derive(Clone, Debug)]
pub struct TestResult {
    pub metadata: TestMetadata,
    pub actual_pass: bool,
    pub matches_expectation: bool,
    pub error_message: Option<String>,
}

pub struct TestHarness {
    pub fs: MemoryFileSystem,
    pub root: String,
    pub metadata: Option<TestMetadata>,
    pub result: Option<TestResult>,
}

impl TestHarness {
    pub fn new() -> Self {
        Self {
            fs: MemoryFileSystem::new(),
            root: "pkg".to_string(),
            metadata: None,
            result: None,
        }
    }

    pub fn with_root(root: impl Into<String>) -> Self {
        Self {
            fs: MemoryFileSystem::new(),
            root: root.into(),
            metadata: None,
            result: None,
        }
    }

    pub fn from_fs(
        fs: MemoryFileSystem,
        root: impl Into<String>,
    ) -> Self {
        Self {
            fs,
            root: root.into(),
            metadata: None,
            result: None,
        }
    }

    pub fn with_metadata(
        fs: MemoryFileSystem,
        id: impl Into<String>,
        name: impl Into<String>,
        purpose: impl Into<String>,
        expect_pass: bool,
        tags: Vec<Tag>,
    ) -> Self {
        Self {
            fs,
            root: "pkg".to_string(),
            metadata: Some(TestMetadata {
                id: id.into(),
                name: name.into(),
                purpose: purpose.into(),
                expect_pass,
                tags,
            }),
            result: None,
        }
    }

    pub fn set_root(
        &mut self,
        root: impl Into<String>,
    ) {
        self.root = root.into();
    }
    pub fn add_file(
        &mut self,
        path: impl AsRef<Path>,
        content: impl AsRef<[u8]>,
    ) {
        self.fs
            .add_file(path.as_ref(), content.as_ref());
    }

    pub fn add_text_file(
        &mut self,
        path: impl AsRef<Path>,
        content: impl AsRef<str>,
    ) {
        self.add_file(path, content.as_ref().as_bytes());
    }

    /// Automatically records the test result if metadata is set.
    pub async fn compile_pass(&mut self) -> CompileCtx {
        let result = compile_pass(Arc::new(self.fs.clone()), &self.root).await;

        if let Some(ref metadata) = self.metadata {
            let (actual_pass, error_message) = match &result {
                Ok(_) => (true, None),
                Err(e) => (false, Some(format!("{:#?}", e))),
            };

            self.result = Some(TestResult {
                metadata: metadata.clone(),
                actual_pass,
                matches_expectation: actual_pass == metadata.expect_pass,
                error_message,
            });
        }

        match result {
            Ok(ctx) => ctx,
            Err(e) => {
                panic!(
                    "Expected compilation to succeed, but got error: {:?}",
                    e.to_report(None, None, None)
                )
            },
        }
    }

    pub async fn compile_fail(&mut self) -> operation_api_parser::Error {
        let result = compile_fail(Arc::new(self.fs.clone()), &self.root).await;

        if let Some(ref metadata) = self.metadata {
            let (actual_pass, error_message) = match &result {
                Ok(err) => (false, Some(format!("{:#?}", err))),
                Err(()) => (true, None),
            };

            self.result = Some(TestResult {
                metadata: metadata.clone(),
                actual_pass,
                matches_expectation: actual_pass == metadata.expect_pass,
                error_message,
            });
        }

        match result {
            Ok(err) => err,
            Err(()) => panic!("Expected compilation to fail, but it succeeded"),
        }
    }

    pub fn result(&self) -> Option<&TestResult> {
        self.result.as_ref()
    }

    pub fn operations(&self) -> Vec<operation_api_fs::memory::FsOperation> {
        self.fs.operations()
    }

    pub fn clear_operations(&mut self) {
        self.fs.clear_operations();
    }

    pub fn lockfile_exists(&self) -> bool {
        let lockfile_path = format!("{}/schema.lock", self.root);
        self.fs
            .exists_sync(Path::new(&lockfile_path))
    }

    pub fn read_lockfile(&self) -> Option<String> {
        let lockfile_path = format!("{}/schema.lock", self.root);
        self.fs
            .get_file_content(Path::new(&lockfile_path))
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }

    pub fn assert_lockfile_written(&self) {
        assert!(
            self.lockfile_exists(),
            "Expected lockfile to be written, but it doesn't exist"
        );
    }

    pub fn assert_no_lockfile(&self) {
        assert!(
            !self.lockfile_exists(),
            "Expected no lockfile, but one exists"
        );
    }

    pub fn assert_lockfile_contains(
        &self,
        package_name: &str,
    ) {
        let content = self
            .read_lockfile()
            .expect("Lockfile should exist to check contents");
        // ok this is a pretty shaky test method tbh but as-is the lockfile api needs stability b/f static tests
        assert!(
            content.contains(package_name),
            "Expected lockfile to contain '{}', but it doesn't.\nLockfile:\n{}",
            package_name,
            content
        );
    }

    #[allow(unused)]
    pub fn assert_operation_occurred(
        &self,
        op_type: &str,
    ) {
        // todo!("Implement assert_operation_occurred");
    }

    pub fn count_operations(
        &self,
        op_type: &str,
    ) -> usize {
        self.operations()
            .iter()
            .filter(|op| format!("{:?}", op).contains(op_type))
            .count()
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl TestMetadata {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        tags: Vec<Tag>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            purpose: String::new(),
            expect_pass: true,
            tags,
        }
    }

    pub fn purpose(
        mut self,
        purpose: impl Into<String>,
    ) -> Self {
        self.purpose = purpose.into();
        self
    }

    pub fn expect_fail(mut self) -> Self {
        self.expect_pass = false;
        self
    }

    pub fn tag(
        mut self,
        tags: Vec<Tag>,
    ) -> Self {
        self.tags = tags;
        self
    }
}

pub async fn compile_pass(
    fs: Arc<dyn FileSystem>,
    root: &str,
) -> Result<CompileCtx, operation_api_parser::Error> {
    CompileCtx::with_fs(fs, root).await
}

pub async fn compile_fail(
    fs: Arc<dyn FileSystem>,
    root: &str,
) -> Result<operation_api_parser::Error, ()> {
    match CompileCtx::with_fs(fs, root).await {
        Ok(_) => Err(()),
        Err(e) => Ok(e),
    }
}

pub use operation_api_fs::memory::{FsOperation, MemoryFileSystem as MemFs};
pub use operation_api_parser::Error;
