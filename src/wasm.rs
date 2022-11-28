use wasm_bindgen::prelude::*;

use crate::{error_message::ErrorMessage, transpile};

#[wasm_bindgen]
pub fn compile(code: &str) -> String {
    let mut buf: Vec<u8> = Vec::new();

    if let Err(err) = transpile(&mut buf, code) {
        err.format_err(&mut buf, ()).unwrap();
    }

    String::from_utf8(buf).unwrap()
}
