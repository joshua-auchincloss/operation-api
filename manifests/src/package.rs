use std::{collections::BTreeMap, path::PathBuf, sync::LazyLock};
use validator::{Validate, ValidationError};

#[allow(clippy::declare_interior_mutable_const)]
const PACKAGE_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new("[a-z]([a-z0-9\\-]*)[a-z0-9]").expect("package re"));

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum PathOrText {
    Path { path: PathBuf },
    Text(String),
}

impl PathOrText {
    pub fn text(&self) -> crate::Result<String> {
        Ok(match self {
            Self::Path { path } => std::fs::read_to_string(path)?,
            Self::Text(text) => text.clone(),
        })
    }
}

#[allow(clippy::borrow_interior_mutable_const)]
fn validate_name(name: &str) -> Result<(), ValidationError> {
    const ERR_SPEC: &str = "package name must be provided without spaces or special characters";
    if let Some(capt) = PACKAGE_RE.find(name) {
        if capt.as_str().len() != name.len() {
            return Err(ValidationError::new("package name").with_message(ERR_SPEC.into()));
        }
    } else {
        return Err(ValidationError::new("package name").with_message(ERR_SPEC.into()));
    }

    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize, validator::Validate)]
pub struct Author {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(email)]
    pub email: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, validator::Validate)]
pub struct PackageMeta {
    #[validate(length(min = 2, max = 128), custom(function = validate_name))]
    pub name: String,
    pub description: PathOrText,
    #[validate(custom(function = super::version::Version::valid_for_package))]
    pub version: super::version::Version,
    pub authors: Vec<Author>,
    #[validate(url)]
    pub homepage: Option<String>,
}

#[derive(serde::Deserialize, validator::Validate, serde::Serialize)]
pub struct PackageManifest {
    #[validate(nested)]
    pub package: PackageMeta,

    pub dependencies: BTreeMap<String, Dependency>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Dependency {
    Git {
        git: String,
        #[serde(rename = "tag")]
        branch: String,
    },
    Path {
        path: PathBuf,
    },
    Remote {
        name: String,
        registry: String,
    },
}

#[cfg(test)]
mod test {
    use validator::Validate;

    use crate::version::Version;

    #[test_case::test_case("abc_types", "0.1.0", "https://github.com/abc/foo.git"; "valid name with underscore")]
    #[test_case::test_case("abc-types", "0.1.0", "https://github.com/abc/foo.git"; "valid name with dash")]
    #[test_case::test_case("abc", "0.1.0", "https://github.com/abc/foo.git"; "simple name")]
    #[test_case::test_case("abc", "0.1.0.rc0", "https://github.com/abc/foo.git"; "version with rc")]
    fn test_pkg_validate_ok(
        name: &str,
        version: &str,
        homepage: &str,
    ) {
        let p = super::PackageMeta {
            name: name.into(),
            description: super::PathOrText::Text("".into()),
            version: Version::parse(version).unwrap(),
            authors: vec![],
            homepage: Some(homepage.into()),
        };
        p.validate().unwrap();
    }

    #[test_case::test_case("a", "0.1.0", "https://github.com/abc/foo.git", "name: Validation error: length"; "name too short")]
    #[test_case::test_case("a".repeat(129).as_str(), 
        "0.1.0", "https://github.com/abc/foo.git", 
        "name: Validation error: length"; "name too long")]
    #[test_case::test_case("abc_types!", "0.1.0", "https://github.com/abc/foo.git", "name: package name must be provided without spaces or special characters"; "invalid character in name")]
    #[test_case::test_case("abc_types", "0.1.0", "not-a-url", "homepage: Validation error: url [{\"value\": String(\"not-a-url\")}]"; "invalid homepage url")]
    #[test_case::test_case("abc", "0.1", "https://github.com/abc/foo.git", "version: Validation error:"; "version without patch")]

    fn test_pkg_validate_err(
        name: &str,
        version: &str,
        homepage: &str,
        expect: &str,
    ) {
        let p = super::PackageMeta {
            name: name.into(),
            description: super::PathOrText::Text("".into()),
            version: Version::parse(version).unwrap(),
            authors: vec![],
            homepage: Some(homepage.into()),
        };
        let err = p.validate().unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains(expect),
            "expected error to contain '{expect}', found '{msg}'"
        );
    }
}
