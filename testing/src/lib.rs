pub use insta::{assert_yaml_snapshot, with_settings};

#[macro_export]
macro_rules! insta_test {
    ($f: expr) => {
        $crate::with_settings!({filters => vec![]}, {
            ($f)()
        })
    };
}
