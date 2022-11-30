#![no_main]

use libfuzzer_sys::fuzz_target;

use taro::error_message::ErrorMessage;
use taro::transpile;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut buf: Vec<u8> = Vec::new();

        if let Err(err) = transpile(&mut buf, s) {
            err.format_err(&mut buf, ()).unwrap();
        }
    }
});
