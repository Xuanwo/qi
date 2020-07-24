use std::collections::BTreeMap;
use std::fs;

use clap::{crate_authors, crate_description, crate_version, App, Arg};
use serde::{Deserialize, Serialize};

use qi_openapi::v3::{from_json_reader, Schema, Spec};

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

    let input = matches.value_of("input").unwrap();
    let file = fs::File::open(input).unwrap();
    let specs = from_json_reader(file).unwrap();

    let ops = format_operations(&specs);
    println!("{:?}", ops);

    let shapes = format_shapes(&specs);
    println!("{:?}", shapes);

    ()
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Operation {
    id: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Shape {
    type_: String,
    items: Option<Box<Shape>>,
    properties: Option<BTreeMap<String, Shape>>,
}

fn format_operations(specs: &Spec) -> Vec<Operation> {
    let mut ops: Vec<Operation> = Vec::new();

    for (_, item) in specs.paths.iter() {
        for op in vec![
            item.get.as_ref(),
            item.put.as_ref(),
            item.post.as_ref(),
            item.delete.as_ref(),
            item.options.as_ref(),
            item.head.as_ref(),
            item.patch.as_ref(),
            item.trace.as_ref(),
        ] {
            if let Some(op) = op {
                ops.push(Operation {
                    id: op.operation_id.to_string(),
                })
            }
        }
    }

    ops
}

fn format_shapes(specs: &Spec) -> BTreeMap<String, Shape> {
    let mut shapes: BTreeMap<String, Shape> = BTreeMap::new();

    let components = specs.components.as_ref().unwrap();

    if let Some(schemas) = &components.schemas {
        for (name, schema) in schemas {
            shapes.insert(name.to_string(), parse_schema(schema));
        }
    }

    shapes
}

fn parse_schema(schema: &Schema) -> Shape {
    if schema.ref_.is_some() {
        println!(
            "schema has ref {}, ignore for now.",
            schema.ref_.as_ref().unwrap()
        );
        return Shape {
            type_: String::from(""),
            items: None,
            properties: None,
        };
    }

    let mut shape = Shape {
        type_: schema.type_.as_ref().unwrap().to_string(),
        items: None,
        properties: None,
    };

    if shape.type_ == "array" {
        shape.items = schema
            .items
            .as_ref()
            .map_or(None, |v| Some(Box::new(parse_schema(v.as_ref()))));
    }

    if let Some(props) = schema.properties.as_ref() {
        let mut shape_props: BTreeMap<String, Shape> = BTreeMap::new();

        for (name, schema) in props {
            shape_props.insert(name.to_string(), parse_schema(schema));
        }

        shape.properties = Some(shape_props);
    }

    shape
}
