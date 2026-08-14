#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let markdown = String::from_utf8_lossy(data);
    std::hint::black_box(ps_render::render(&markdown));
});
