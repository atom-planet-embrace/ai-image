#![no_main]
#[macro_use] extern crate libfuzzer_sys;

fuzz_target!(|data: &[u8]| {
    let _ = decode(data);
});

fn decode(data: &[u8]) -> Result<(), ai_image::ImageError> {
    use ai_image::ImageDecoder;
    let decoder = ai_image::codecs::tga::TgaDecoder::new(std::io::Cursor::new(data))?;
    if decoder.total_bytes() > 4_000_000 {
        return Ok(());
    }
    let mut buffer = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut buffer)?;
    Ok(())
}
