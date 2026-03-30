extern crate afl;

use std::io::Cursor;

use ai_image::{DynamicImage, ImageDecoder};
use ai_image::error::{ImageError, ImageResult, LimitError, LimitErrorKind};

#[inline(always)]
fn webp_decode(data: &[u8]) -> ImageResult<DynamicImage> {
    let decoder = ai_image::codecs::webp::WebPDecoder::new(Cursor::new(data))?;
    let (width, height) = decoder.dimensions();

    if width.saturating_mul(height) > 4_000_000 {
        return Err(ImageError::Limits(LimitError::from_kind(LimitErrorKind::DimensionError)));
    }

    DynamicImage::from_decoder(decoder)
}

fn main() {
    afl::fuzz(true, |data| {
        let _ = webp_decode(data);
    });
}
