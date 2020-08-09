mod rust;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;

use clap::{crate_authors, crate_description, crate_version, App, Arg};
use serde::{Deserialize, Serialize};

use qi_algorithm;
use qi_openapi::v3;
pub use qi_openapi::v3::Method;
use qi_openapi::v3::{from_json_reader, from_yaml_reader, Schema, SchemaType, Spec};

use crate::rust::ActixWebGenerator;
use rust::Generator;

fn main() {
    let matches = App::new("qi Rust")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("input")
                .required(true)
                .index(1)
                .help("Set input"),
        )
        .get_matches();

    let input = Path::new(matches.value_of("input").unwrap());
    let file = fs::File::open(input).unwrap();

    let specs;
    match input.extension().and_then(OsStr::to_str).unwrap() {
        "json" => specs = from_json_reader(file).unwrap(),
        "yaml" | "yml" => specs = from_yaml_reader(file).unwrap(),
        extension @ _ => panic!("not supported file extension: {}", extension),
    }

    let mut models = format_models(&specs);
    let global_parameters = format_parameters(&specs);
    let mut ops = format_operations(&global_parameters, &specs);

    for (name, parameter) in global_parameters.iter() {
        if parameter.model.kind == ModelKind::Struct {
            models.insert(name.clone(), parameter.model.clone());
        }
    }

    for op in ops.iter_mut() {
        for param in op.input.parameters.iter_mut() {
            if param.model.kind == ModelKind::Struct {
                models.insert(param.name.clone(), param.model.clone());
                param.model = Model {
                    kind: ModelKind::Reference,
                    annotation: None,
                    name: Some(param.name.clone()),
                    properties: None,
                    element: None,
                }
            }
        }
    }

    let g = Generator::new(models);
    let actix_g = ActixWebGenerator::new(g);

    // actix_g.generate_structs();
    for op in ops {
        println!("{}\n", actix_g.generate_output(op))
    }

    // let x = serde_json::to_value(&ops).unwrap();
    // println!("{}", serde_json::to_string_pretty(&x).unwrap());

    ()
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Service {
    models: BTreeMap<String, Model>,
    operations: Vec<Operation>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Operation {
    id: String,

    method: Method,
    uri: String,
    expect: Vec<usize>,

    input: Message,
    output: Message,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    parameters: Vec<Parameter>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    // request only
    Path,
    // request only
    Query,

    Header,
    // Should have only one parameter.
    Body,
}

impl FromStr for ParameterLocation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "path" => Ok(ParameterLocation::Path),
            "query" => Ok(ParameterLocation::Query),
            "header" => Ok(ParameterLocation::Header),
            "body" => Ok(ParameterLocation::Body),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Parameter {
    name: String,
    model: Model,
    location: ParameterLocation,
    mandatory: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
enum ModelKind {
    // Any used for dynamic type
    Any,
    Boolean,
    // UTF-8 string
    String,
    Byte,

    Date,
    Time,
    Datetime,

    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint,
    Uint8,
    Uint16,
    Uint32,
    Uint64,

    Float32,
    Float64,

    Array,
    Map,
    Enum,
    Struct,
    Iterator,

    Reference,
}

impl ModelKind {
    fn is_simple(&self) -> bool {
        match self {
            ModelKind::Any
            | ModelKind::Array
            | ModelKind::Map
            | ModelKind::Enum
            | ModelKind::Struct
            | ModelKind::Iterator
            | ModelKind::Reference => false,
            _ => true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Model {
    #[serde(rename = "type")]
    kind: ModelKind,

    #[serde(skip_serializing_if = "Option::is_none")]
    annotation: Option<Annotation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<BTreeMap<String, Model>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    element: Option<Box<Model>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
struct Annotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    display: Option<String>,
}

fn format_operations(gp: &BTreeMap<String, Parameter>, specs: &Spec) -> Vec<Operation> {
    let mut ops: Vec<Operation> = Vec::new();

    for (path, item) in specs.paths.iter() {
        for (method, o) in item.operations.iter() {
            let mut op = Operation {
                id: o.operation_id.clone(),
                method: (*method).clone(),
                uri: path.clone(),
                expect: Vec::new(),
                description: None,
                tags: None,
                input: Message {
                    description: None,
                    parameters: Vec::new(),
                },
                output: Message {
                    description: None,
                    parameters: Vec::new(),
                },
            };

            println!("format operation {}", &op.id);

            if let Some(params) = o.parameters.as_ref() {
                for param in params.iter() {
                    op.input.parameters.push(parse_parameter(gp, param));
                }
            }

            if let Some(body) = o.request_body.as_ref() {
                let (_, media_type) = body.content.iter().next().unwrap();
                op.input.parameters.push(Parameter {
                    name: "body".to_string(),
                    model: parse_schema_type(&media_type.schema),
                    location: ParameterLocation::Body,
                    mandatory: true,
                })
            }

            o.responses.as_ref().and_then(|responses| {
                for (name, response) in responses.iter() {
                    if name == "default" {
                        println!("ignore operation {}'s default response", &op.id);
                        continue;
                    }

                    let status_code = usize::from_str(name.as_str()).unwrap();
                    if status_code < 100 || status_code > 300 {
                        println!(
                            "ignore operation {}'s {} error response",
                            &op.id, &status_code
                        );
                        continue;
                    }

                    op.expect.push(status_code);

                    let mut output = Message {
                        description: None,
                        parameters: Vec::new(),
                    };

                    if let Some(headers) = response.headers.as_ref() {
                        for (name, header) in headers.iter() {
                            output.parameters.push(Parameter {
                                name: name.clone(),
                                model: parse_schema_type(&header.schema),
                                location: ParameterLocation::Header,
                                mandatory: false,
                            });
                        }
                    }

                    if let Some(content) = response.content.as_ref() {
                        let (_, media_type) = content.iter().next().unwrap();
                        output.parameters.push(Parameter {
                            name: "body".to_string(),
                            model: parse_schema_type(&media_type.schema),
                            location: ParameterLocation::Body,
                            mandatory: false,
                        });
                    }

                    if op.output.parameters.len() != 0 {
                        println!("operation {}'s output has been set multiple times", &op.id)
                    }
                    op.output = output;
                }

                Some(responses)
            });

            ops.push(op);
        }
    }

    ops
}

fn format_parameters(specs: &Spec) -> BTreeMap<String, Parameter> {
    let mut gp = BTreeMap::new();

    let components = specs.components.as_ref().unwrap();

    if let Some(params) = &components.parameters {
        for (name, param) in params {
            // Parameters in components could not be references.
            gp.insert(
                name.to_string(),
                Parameter {
                    name: param.name.clone().unwrap(),
                    model: parse_schema_type(param.schema.as_ref().unwrap()),
                    location: ParameterLocation::from_str(param.in_.as_ref().unwrap().as_str())
                        .unwrap(),
                    mandatory: param.required.unwrap_or(false),
                },
            );
        }
    }

    gp
}

fn format_models(specs: &Spec) -> BTreeMap<String, Model> {
    let mut shapes: BTreeMap<String, Model> = BTreeMap::new();

    let components = specs.components.as_ref().unwrap();

    if let Some(schemas) = &components.schemas {
        for (name, schema) in schemas {
            // println!("parsing schema {}", name);
            let model = parse_schema_type(schema);
            if model.kind == ModelKind::Any {
                continue;
            }
            shapes.insert(name.to_string(), model);
        }
    }

    shapes
}

fn parse_parameter(gp: &BTreeMap<String, Parameter>, param: &v3::Parameter) -> Parameter {
    if let Some(r) = param.ref_.as_ref() {
        gp.get(parse_ref(r).as_str()).unwrap().clone()
    } else {
        Parameter {
            name: param.name.clone().unwrap(),
            model: parse_schema_type(param.schema.as_ref().unwrap()),
            location: ParameterLocation::from_str(param.in_.as_ref().unwrap().as_str()).unwrap(),
            mandatory: param.required.unwrap_or(false),
        }
    }
}

fn parse_ref(s: &String) -> String {
    s.split("/").last().unwrap().to_string()
}

fn parse_schema_type(schema: &Schema) -> Model {
    if let Some(r) = schema.ref_.as_ref() {
        return Model {
            kind: ModelKind::Reference,
            name: Some(parse_ref(r)),
            annotation: None,
            properties: None,
            element: None,
        };
    }

    let mut model = Model {
        kind: ModelKind::Any,
        annotation: None,
        name: None,
        properties: None,
        element: None,
    };

    if schema.type_.as_ref().is_none() {
        return model;
    }

    let schema_type = schema.type_.as_ref().unwrap();

    match schema_type {
        SchemaType::Boolean => model.kind = ModelKind::Boolean,
        SchemaType::Object => {
            model.kind = ModelKind::Struct;

            if let Some(props) = schema.properties.as_ref() {
                let mut m = BTreeMap::new();

                for (name, property) in props {
                    m.insert(name.to_string(), parse_schema_type(&property));
                }

                model.properties = Some(m);
            }
        }
        SchemaType::Array => {
            model.kind = ModelKind::Array;

            match schema.items.as_ref() {
                None => println!("schema has kind array, but items is none"),
                Some(items) => model.element = Some(Box::new(parse_schema_type(items.as_ref()))),
            }
        }
        SchemaType::Number => match schema.format.as_ref() {
            None => model.kind = ModelKind::Float32,
            Some(v) => match v.as_str() {
                "double" => model.kind = ModelKind::Float64,
                "float" | _ => model.kind = ModelKind::Float32,
            },
        },
        SchemaType::String => {
            model.kind = ModelKind::String;

            if let Some(v) = schema.format.as_ref() {
                match v.as_str() {
                    // TODO: base64 encoded characters
                    "byte" => {}
                    "binary" => {
                        model.kind = ModelKind::Array;
                        model.element = Some(Box::new(Model {
                            kind: ModelKind::Byte,
                            annotation: None,
                            name: None,
                            properties: None,
                            element: None,
                        }))
                    }
                    _ => {}
                }
            }
        }
        SchemaType::Integer => {
            match schema.format.as_ref() {
                None => model.kind = ModelKind::Int,
                Some(v) => {
                    match v.as_str() {
                        "int32" => model.kind = ModelKind::Int32,
                        "int64" => model.kind = ModelKind::Int64,
                        _ => model.kind = ModelKind::Int,
                    };
                }
            };
        }
    };

    model
}
