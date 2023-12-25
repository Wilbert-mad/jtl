// https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-rust-exports/constructor.html

use jtl_service::{diagnostic, document, SGlobal, SchemaService, Service, StructuresMidd};
use lsp_types::Position;
use serde_wasm_bindgen;
use serde_wasm_bindgen::Error;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[deprecated]
#[wasm_bindgen(js_name = doDiagnostic)]
pub fn do_diagnostic(source: String) -> Result<JsValue, Error> {
    serde_wasm_bindgen::to_value(&diagnostic(source))
}

#[wasm_bindgen]
pub struct WASMLspSchema {
    v: String,
    global: Vec<SGlobal>,
    structures: HashMap<String, Vec<StructuresMidd>>,
}

#[wasm_bindgen]
impl WASMLspSchema {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WASMLspSchema {
            v: "1.0.0".to_string(),
            global: Vec::new(),
            structures: HashMap::new(),
        }
    }

    pub fn get_version(&self) -> String {
        self.v.clone()
    }

    pub fn insert_global(&mut self, key: String, struct_type: String) {
        self.global.push(SGlobal(key, struct_type))
    }

    // pub fn insert_struct(&mut self, key: String, key_struct: Vec) {
    //     self.structures.insert(key, key_struct)
    // }

    fn into_schema(&self) -> SchemaService {
        SchemaService {
            v: self.get_version(),
            global: self.global.clone(),
            structures: self.structures.clone(),
        }
    }
}

#[wasm_bindgen(js_name = serviceDoAutocomplete)]
pub fn service_do_autocomplete(
    source: String,
    position: Box<[u32]>,
    schema_service: &WASMLspSchema,
) -> Result<JsValue, Error> {
    // let schema: Option<SchemaService> = {
    //     if schema_service.is_some() {
    //         Some(schema_service.unwrap().into_schema())
    //     } else {
    //         None
    //     }
    // };
    let schema = schema_service.into_schema();

    let completion_results = Service::do_autocomplete(
        document::Document::new("//master".to_string(), "jtl".to_string(), 1, source),
        Position {
            character: position[0],
            line: position[1],
        },
        Some(schema),
    );

    serde_wasm_bindgen::to_value(&completion_results)
}

#[wasm_bindgen(js_name = serviceDoDiagnostic)]
pub fn service_do_diagnostic() {}
