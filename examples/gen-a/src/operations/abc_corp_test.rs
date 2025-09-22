#[derive(serde :: Serialize, serde :: Deserialize, operation_api_sdk :: Struct)]
#[fields(version = 1)]
#[doc = "Some struct"]
#[fields(describe(text = "Some struct"))]
pub(crate) struct BasicStruct {
    #[serde(rename = "a")]
    #[fields(describe(text = "field a"))]
    #[doc = "field a"]
    pub(crate) a: i32,
    #[serde(rename = "b")]
    pub(crate) b: Option<i32>,
    #[serde(rename = "c")]
    pub(crate) c: Vec<f32>,
    #[serde(rename = "d")]
    pub(crate) d: [[f32; 4]; 4],
}
#[derive(serde :: Serialize, serde :: Deserialize, operation_api_sdk :: Struct)]
#[fields(version = 1)]
#[doc = "# test\nthis is a struct description\n"]
#[fields(describe(text = "# test\nthis is a struct description\n"))]
pub struct BasicStructWithReadme {
    #[serde(rename = "a")]
    #[fields(describe(text = "field a"))]
    #[doc = "field a"]
    pub a: i32,
}
#[derive(serde :: Serialize, serde :: Deserialize, operation_api_sdk :: Struct)]
#[fields(version = 1)]
pub struct SomeStructWithEnum {
    #[serde(rename = "enum_value")]
    #[fields(enm)]
    pub enum_value: SomeEnum,
}
#[derive(operation_api_sdk :: Enum)]
#[fields(version = 1)]
#[derive(operation_api_sdk :: IntDeserialize, operation_api_sdk :: IntSerialize)]
#[repr(u64)]
#[doc = "some int based enum"]
#[fields(describe(text = "some int based enum"))]
pub enum BasicIntEnum {
    A = 0,
    #[doc = "B is 99"]
    #[fields(describe(text = "B is 99"))]
    B = 99,
}
#[derive(operation_api_sdk :: Enum)]
#[fields(version = 1)]
#[derive(serde :: Serialize, serde :: Deserialize)]
pub enum BasicStrEnum {
    #[fields(str_value = "a")]
    #[serde(rename = "a")]
    A,
    #[fields(str_value = "b")]
    #[serde(rename = "b")]
    B,
}
#[derive(operation_api_sdk :: Enum)]
#[fields(version = 1)]
#[derive(operation_api_sdk :: IntDeserialize, operation_api_sdk :: IntSerialize)]
#[repr(u64)]
pub enum SomeEnum {
    A = 0,
}
operation_api_sdk::namespace! { "abc.corp.test" { BasicStruct , BasicStructWithReadme , SomeStructWithEnum , BasicIntEnum , BasicStrEnum , SomeEnum , } }
