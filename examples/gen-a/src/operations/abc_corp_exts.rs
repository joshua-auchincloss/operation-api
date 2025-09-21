#[derive(serde :: Serialize, serde :: Deserialize, operation_api_sdk :: Struct)]
#[fields(version = 1)]
pub struct StructWithOneOf {
    #[serde(rename = "my_flag")]
    #[fields(enm)]
    pub my_flag: MaybeFlagType,
}
#[derive(serde :: Serialize, serde :: Deserialize, operation_api_sdk :: OneOf)]
#[serde(untagged)]
#[fields(version = 1)]
pub enum MaybeFlagType {
    BoolFlag,
    Int(i32),
    Str(String),
}
operation_api_core::namespace! { "abc.corp.exts" { StructWithOneOf , MaybeFlagType , } }
