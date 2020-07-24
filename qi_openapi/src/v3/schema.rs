use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Spec {
    pub openapi: String,
    pub paths: BTreeMap<String, PathItem>,
    pub components: Option<Components>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct PathItem {
    pub summary: Option<String>,
    pub description: Option<String>,

    pub get: Option<Operation>,
    pub put: Option<Operation>,
    pub post: Option<Operation>,
    pub delete: Option<Operation>,
    pub options: Option<Operation>,
    pub head: Option<Operation>,
    pub patch: Option<Operation>,
    pub trace: Option<Operation>,

    pub parameters: Option<BTreeMap<String, Parameter>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Components {
    pub schemas: Option<BTreeMap<String, Schema>>,
    pub responses: Option<BTreeMap<String, Response>>,
    pub parameters: Option<BTreeMap<String, Parameter>>,
    pub request_bodies: Option<BTreeMap<String, RequestBody>>,
    pub headers: Option<BTreeMap<String, Header>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Schema {
    #[serde(rename = "$ref")]
    pub ref_: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub items: Option<Box<Schema>>,
    pub format: Option<String>,
    pub properties: Option<BTreeMap<String, Schema>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Response {
    pub description: Option<String>,
    pub headers: Option<BTreeMap<String, Header>>,
    pub content: Option<BTreeMap<String, MediaType>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub in_: String,
    pub required: Option<bool>,
    pub schema: Schema,
    pub style: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct RequestBody {
    pub content: BTreeMap<String, MediaType>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Header {
    pub description: Option<String>,
    pub schema: Schema,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Operation {
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    pub parameters: Option<Vec<Parameter>>,
    pub request_body: Option<RequestBody>,
    pub responses: Option<BTreeMap<String, Response>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct MediaType {
    pub schema: Schema,
}
