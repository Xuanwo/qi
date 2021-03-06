use crate::{Model, ModelKind, Operation, Parameter, Service};
use std::collections::{BTreeMap, HashMap};

pub struct Generator {
    srv: Service,
}

impl Generator {
    pub fn new(srv: Service) -> Generator {
        Generator { srv }
    }

    fn generate_type(&self, m: &Model) -> String {
        let s = match m.kind {
            // Simple type
            ModelKind::Any => "std::any::Any",
            ModelKind::Boolean => "bool",
            ModelKind::String => "String",
            ModelKind::Byte => "u8",
            // FIXME
            ModelKind::Date => "",
            ModelKind::Time => "",
            ModelKind::Datetime => "",
            ModelKind::Int => "isize",
            ModelKind::Int8 => "i8",
            ModelKind::Int16 => "i16",
            ModelKind::Int32 => "i32",
            ModelKind::Int64 => "i64",
            ModelKind::Uint => "usize",
            ModelKind::Uint8 => "u8",
            ModelKind::Uint16 => "u16",
            ModelKind::Uint32 => "u32",
            ModelKind::Uint64 => "u64",
            ModelKind::Float32 => "f32",
            ModelKind::Float64 => "f64",
            // Compose type
            ModelKind::Array => {
                assert!(m.element.is_some());

                let element = m.element.as_ref().unwrap();
                let element_type = self.generate_type(element);

                return format!("Vec<{}>", element_type);
            }
            // FIXME
            ModelKind::Enum => "",
            ModelKind::Map => {
                assert!(m.element.is_some());

                let element = m.element.as_ref().unwrap();
                let element_type = self.generate_type(element);

                return format!("HashMap<String, {}>", element_type);
            }
            ModelKind::Struct => {
                println!("struct {:?} should be extracted as another type", m);

                ""
            }
            // FIXME
            ModelKind::Iterator => "",
            ModelKind::Reference => {
                assert!(m.name.is_some());

                let name = m.name.clone().unwrap();
                let ref_model = self.srv.models.get(&name).unwrap();

                if ref_model.kind == ModelKind::Struct {
                    return name;
                } else {
                    return self.generate_type(ref_model);
                }
            }
        };

        s.to_string()
    }

    // Language related.
    pub fn generate_struct(&self, name: &String, m: &Model) -> String {
        assert_eq!(ModelKind::Struct, m.kind);

        let mut s = String::new();

        if m.properties.as_ref().is_none() {
            s.push_str(format!("struct {} {{}}", name).as_str());
            return s;
        }

        s.push_str(format!("struct {} {{\n", name).as_str());
        for (name, prop) in m.properties.as_ref().unwrap().iter() {
            s.push_str(format!("  {}: {},\n", name, self.generate_type(prop)).as_str());
        }
        s.push_str("}\n");

        s
    }
}

pub struct ActixWebGenerator {
    g: Generator,
}

// common
//
// generate_iterator()
// generate_input(operation)
// generate_output(operation)

// server
//
// generate_dispatch(operations): Request -> operationId
// generate_parse_request(operation): Request->input
// generate_format_response(operation): output->Response
// generate_handle(operation) -> fn PutObject(input) -> output, error

// client
//
// generate_format_request(operation): input->Request
// generate_parse_response(operation): Response->output
// generate_send(operation) -> fn PutObject(Request) -> Response, error
impl ActixWebGenerator {
    pub fn new(g: Generator) -> ActixWebGenerator {
        ActixWebGenerator { g }
    }
    fn generate_iterator() {}
    pub fn generate_structs(&self) {
        for (name, model) in self.g.srv.models.iter() {
            if model.kind != ModelKind::Struct {
                continue;
            }
            println!("{}\n", self.g.generate_struct(name, model))
        }
    }
    pub fn generate_input(&self, op: Operation) -> String {
        let name = op.id + "Input";
        let mut model = Model {
            kind: ModelKind::Struct,
            annotation: None,
            name: None,
            properties: Some(BTreeMap::new()),
            element: None,
        };

        let m = model.properties.as_mut().unwrap();

        for param in op.input.path.iter() {
            m.insert(param.name.clone(), param.model.clone());
        }
        for param in op.input.query.iter() {
            m.insert(param.name.clone(), param.model.clone());
        }
        for param in op.input.header.iter() {
            m.insert(param.name.clone(), param.model.clone());
        }
        if let Some(body) = op.input.body {
            if body.kind == ModelKind::String {
                m.insert(
                    "body".to_string(),
                    Model {
                        kind: ModelKind::Iterator,
                        annotation: None,
                        name: None,
                        properties: None,
                        element: Some(Box::new(Model {
                            kind: ModelKind::Byte,
                            annotation: None,
                            name: None,
                            properties: None,
                            element: None,
                        })),
                    },
                );
            } else {
                m.extend(body.properties.clone().unwrap());
            }
        }

        self.g.generate_struct(&name, &model)
    }
    pub fn generate_output(&self, op: Operation) -> String {
        let name = op.id + "Output";
        let mut model = Model {
            kind: ModelKind::Struct,
            annotation: None,
            name: None,
            properties: Some(BTreeMap::new()),
            element: None,
        };

        let m = model.properties.as_mut().unwrap();

        for param in op.output.header.iter() {
            m.insert(param.name.clone(), param.model.clone());
        }
        if let Some(body) = op.output.body {
            if body.kind == ModelKind::String {
                m.insert(
                    "body".to_string(),
                    Model {
                        kind: ModelKind::Iterator,
                        annotation: None,
                        name: None,
                        properties: None,
                        element: Some(Box::new(Model {
                            kind: ModelKind::Byte,
                            annotation: None,
                            name: None,
                            properties: None,
                            element: None,
                        })),
                    },
                );
            } else {
                if let Some(props) = body.properties.clone() {
                    m.extend(props);
                } else {
                    m.insert("body".to_string(), body);
                }
            }
        }

        self.g.generate_struct(&name, &model)
    }

    //  App::new().service(web::resource("/index.html").route(
    //         web::route()
    //             .guard(
    //                 guard::fn_guard(
    //                     |req| req.headers()
    //                              .contains_key("content-type")))
    //             .to(|| HttpResponse::MethodNotAllowed()))
    //     );
    pub fn generate_dispatch(&self) -> String {
        let mut s = String::new();

        s.push_str("App::new()\n");
        for op in self.g.srv.operations.iter() {
            let (id, method, uri) = (&op.id, &op.method, &op.uri);

            s.push_str(
                format!(
                    "  .service(
    web::resource(\"{}\")
      .route(
        web::route().guard(guard::{:?}())
    )))
  )\n",
                    uri, method,
                )
                .as_str(),
            );
        }

        s
    }
    fn generate_parse_request(&self) {}
    fn generate_format_response(&self) {}
    fn generate_handle(&self) {}
}
