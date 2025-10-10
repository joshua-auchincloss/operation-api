use std::path::PathBuf;

use config::File;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::Error;

pub trait NewForConfig
where
    Self: Sized + DeserializeOwned + Validate, {
    const NAME: &'static str;
    fn new<S: AsRef<str>>(dir: Option<S>) -> crate::Result<Self> {
        let file_name = format!(
            "{}",
            PathBuf::from(
                dir.map(|s| String::from(s.as_ref()))
                    .unwrap_or("./".into())
            )
            .join(Self::NAME)
            .display()
        );

        let this: Self = config::ConfigBuilder::<config::builder::DefaultState>::default()
            .add_source(File::with_name(&file_name).required(false))
            .add_source(config::Environment::default().prefix("OP"))
            .build()
            .map_err(Error::from_with_source_init(file_name.clone()))?
            .try_deserialize()
            .map_err(Error::from_with_source_init(file_name.clone()))?;

        this.validate()
            .map_err(Error::from_with_source_init(file_name.clone()))?;

        Ok(this)
    }
}
