#![no_main]
#[macro_use] extern crate libfuzzer_sys;

fuzz_target!(|data: &[u8]| {
    let _ = ai_image::load_from_memory_with_format(data, ai_image::ImageFormat::Png);
});
