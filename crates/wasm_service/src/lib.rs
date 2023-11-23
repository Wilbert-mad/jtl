use jtl_service::diagnostic;
use serde_wasm_bindgen;
use serde_wasm_bindgen::Error;
use wasm_bindgen::prelude::*;

#[deprecated]
#[wasm_bindgen(js_name = doDiagnostic)]
pub fn do_diagnostic(source: String) -> Result<JsValue, Error> {
    serde_wasm_bindgen::to_value(&diagnostic(source))
}

#[wasm_bindgen(js_name = createSchema)]
pub fn create_schema() {}

#[wasm_bindgen(js_name = serviceDoDiagnostic)]
pub fn service_do_diagnostic() {}

#[wasm_bindgen(js_name = serviceDoAutocomplete)]
pub fn service_do_autocomplete() {}
