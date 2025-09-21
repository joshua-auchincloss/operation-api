#[derive(serde :: Serialize, serde :: Deserialize, operation_api_sdk :: Struct)]
#[fields(version = 2)]
pub struct SomeStruct {
    #[serde(rename = "a")]
    pub a: i32,
    #[serde(rename = "b")]
    pub b: f32,
    #[serde(rename = "c")]
    pub c: [[f32; 4]; 4],
    #[serde(rename = "d")]
    pub d: [[f32; 4]; 4],
}
operation_api_core::namespace! { "abc.corp.namespace" { SomeStruct , } }
