#[derive(serde :: Serialize, serde :: Deserialize, operation_api_sdk :: Struct)]
#[fields(version = 1)]
#[doc = "Some struct"]
#[fields(describe(text = "Some struct"))]
pub struct BasicStruct {
    #[serde(rename = "a")]
    #[fields(describe(text = "field a"))]
    #[doc = "field a"]
    a: i32,
    #[serde(rename = "b")]
    b: Option<i32>,
    #[serde(rename = "c")]
    c: Vec<f32>,
    #[serde(rename = "d")]
    d: [[f32; 4]; 4],
}
#[derive(serde :: Serialize, serde :: Deserialize, operation_api_sdk :: Struct)]
#[fields(version = 1)]
#[doc = "# test\nthis is a struct description\n"]
#[fields(describe(text = "# test\nthis is a struct description\n"))]
pub struct BasicStructWithReadme {
    #[serde(rename = "a")]
    #[fields(describe(text = "field a"))]
    #[doc = "field a"]
    a: i32,
}
operation_api_core::namespace! { "abc.corp.test" { BasicStruct , BasicStructWithReadme , } }
