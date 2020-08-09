use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Spec {
    pub openapi: String,
    pub paths: BTreeMap<String, PathItem>,
    pub components: Option<Components>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Copy, Clone, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    Head,
    Patch,
    Options,
    Trace,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct PathItem {
    pub summary: Option<String>,
    pub description: Option<String>,

    pub parameters: Option<Vec<Parameter>>,

    #[serde(flatten)]
    pub operations: BTreeMap<Method, Operation>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Components {
    pub schemas: Option<BTreeMap<String, Schema>>,
    pub responses: Option<BTreeMap<String, Response>>,
    pub parameters: Option<BTreeMap<String, Parameter>>,
    pub request_bodies: Option<BTreeMap<String, RequestBody>>,
    pub headers: Option<BTreeMap<String, Header>>,
}

// ref: https://swagger.io/specification/
//   - integer as a type is also supported and is defined as a JSON number without a fraction or exponent part.
//   - null is not supported as a type (see nullable for an alternative solution).
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    Boolean,
    Object,
    Array,
    Number,
    String,
    Integer,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Schema {
    #[serde(rename = "$ref")]
    pub ref_: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<SchemaType>,
    pub items: Option<Box<Schema>>,
    pub format: Option<String>,
    pub properties: Option<BTreeMap<String, Schema>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Response {
    pub description: Option<String>,
    pub headers: Option<BTreeMap<String, Header>>,
    pub content: Option<BTreeMap<String, MediaType>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Parameter {
    #[serde(rename = "$ref")]
    pub ref_: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "in")]
    pub in_: Option<String>,
    pub required: Option<bool>,
    pub schema: Option<Schema>,
    pub style: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct RequestBody {
    pub content: BTreeMap<String, MediaType>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Header {
    pub description: Option<String>,
    pub schema: Schema,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Operation {
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    pub parameters: Option<Vec<Parameter>>,
    #[serde(rename = "requestBody")]
    pub request_body: Option<RequestBody>,
    pub responses: Option<BTreeMap<String, Response>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct MediaType {
    pub schema: Schema,
}
