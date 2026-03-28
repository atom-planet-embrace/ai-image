//! Types describing image metadata
pub(crate) mod cicp;

use core::num::NonZeroU32;

pub use self::cicp::{
    Cicp, CicpColorPrimaries, CicpMatrixCoefficients, CicpTransferCharacteristics, CicpTransform,
    CicpVideoFullRangeFlag,
};

/// Describes the transformations to be applied to the image.
/// Compatible with [Exif orientation](https://web.archive.org/web/20200412005226/https://www.impulseadventure.com/photo/exif-orientation.html).
///
/// Orientation is specified in the file's metadata, and is often written by cameras.
///
/// You can apply it to an image via [`DynamicImage::apply_orientation`](crate::DynamicImage::apply_orientation).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Orientation {
    /// Do not perform any transformations.
    NoTransforms,
    /// Rotate by 90 degrees clockwise.
    Rotate90,
    /// Rotate by 180 degrees. Can be performed in-place.
    Rotate180,
    /// Rotate by 270 degrees clockwise. Equivalent to rotating by 90 degrees counter-clockwise.
    Rotate270,
    /// Flip horizontally. Can be performed in-place.
    FlipHorizontal,
    /// Flip vertically. Can be performed in-place.
    FlipVertical,
    /// Rotate by 90 degrees clockwise and flip horizontally.
    Rotate90FlipH,
    /// Rotate by 270 degrees clockwise and flip horizontally.
    Rotate270FlipH,
}

impl Orientation {
    /// Converts from [Exif orientation](https://web.archive.org/web/20200412005226/https://www.impulseadventure.com/photo/exif-orientation.html)
    #[must_use]
    pub fn from_exif(exif_orientation: u8) -> Option<Self> {
        match exif_orientation {
            1 => Some(Self::NoTransforms),
            2 => Some(Self::FlipHorizontal),
            3 => Some(Self::Rotate180),
            4 => Some(Self::FlipVertical),
            5 => Some(Self::Rotate90FlipH),
            6 => Some(Self::Rotate90),
            7 => Some(Self::Rotate270FlipH),
            8 => Some(Self::Rotate270),
            0 | 9.. => None,
        }
    }

    /// Converts into [Exif orientation](https://web.archive.org/web/20200412005226/https://www.impulseadventure.com/photo/exif-orientation.html)
    #[must_use]
    pub fn to_exif(self) -> u8 {
        match self {
            Self::NoTransforms => 1,
            Self::FlipHorizontal => 2,
            Self::Rotate180 => 3,
            Self::FlipVertical => 4,
            Self::Rotate90FlipH => 5,
            Self::Rotate90 => 6,
            Self::Rotate270FlipH => 7,
            Self::Rotate270 => 8,
        }
    }

    /// Extracts the image orientation from a raw Exif chunk.
    ///
    /// You can obtain the Exif chunk using
    /// [ImageDecoder::exif_metadata](crate::ImageDecoder::exif_metadata).
    ///
    /// It is more convenient to use [ImageDecoder::orientation](crate::ImageDecoder::orientation)
    /// than to invoke this function.
    /// Only use this function if you extract and process the Exif chunk separately.
    #[must_use]
    pub fn from_exif_chunk(chunk: &[u8]) -> Option<Self> {
        Self::from_exif_chunk_inner(chunk).map(|res| res.0)
    }

    /// Extracts the image orientation from a raw Exif chunk and sets the orientation in the Exif chunk to `Orientation::NoTransforms`.
    /// This is useful if you want to apply the orientation yourself, and then encode the image with the rest of the Exif chunk intact.
    ///
    /// If the orientation data is not cleared from the Exif chunk after you apply the orientation data yourself,
    /// the image will end up being rotated once again by any software that correctly handles Exif, leading to an incorrect result.
    ///
    /// If the Exif value is present but invalid, `None` is returned and the Exif chunk is not modified.
    #[must_use]
    pub fn remove_from_exif_chunk(chunk: &mut [u8]) -> Option<Self> {
        if let Some((orientation, offset, endian)) = Self::from_exif_chunk_inner(chunk) {
            let off = offset as usize;
            let no_orientation: u16 = Self::NoTransforms.to_exif().into();
            match endian {
                ExifEndian::Big => {
                    let bytes = no_orientation.to_be_bytes();
                    chunk[off..off + 2].copy_from_slice(&bytes);
                }
                ExifEndian::Little => {
                    let bytes = no_orientation.to_le_bytes();
                    chunk[off..off + 2].copy_from_slice(&bytes);
                }
            }
            Some(orientation)
        } else {
            None
        }
    }

    /// Returns the orientation, the offset in the Exif chunk where it was found, and Exif chunk endianness
    #[must_use]
    fn from_exif_chunk_inner(chunk: &[u8]) -> Option<(Self, u64, ExifEndian)> {
        if chunk.len() < 4 {
            return None;
        }
        let magic = &chunk[..4];

        match magic {
            [0x49, 0x49, 42, 0] => {
                return Self::locate_orientation_entry(chunk, ExifEndian::Little)
                    .map(|(orient, offset)| (orient, offset, ExifEndian::Little));
            }
            [0x4d, 0x4d, 0, 42] => {
                return Self::locate_orientation_entry(chunk, ExifEndian::Big)
                    .map(|(orient, offset)| (orient, offset, ExifEndian::Big));
            }
            _ => {}
        }
        None
    }

    fn read_u16_at(chunk: &[u8], offset: usize, endian: ExifEndian) -> Option<u16> {
        let bytes = chunk.get(offset..offset + 2)?;
        Some(match endian {
            ExifEndian::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
            ExifEndian::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
        })
    }

    fn read_u32_at(chunk: &[u8], offset: usize, endian: ExifEndian) -> Option<u32> {
        let bytes = chunk.get(offset..offset + 4)?;
        Some(match endian {
            ExifEndian::Big => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            ExifEndian::Little => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        })
    }

    /// Locate the orientation entry in the Exif IFD
    fn locate_orientation_entry(chunk: &[u8], endian: ExifEndian) -> Option<(Self, u64)> {
        let ifd_offset = Self::read_u32_at(chunk, 4, endian)? as usize;
        let entries = Self::read_u16_at(chunk, ifd_offset, endian)?;
        let mut pos = ifd_offset + 2;
        for _ in 0..entries {
            let tag = Self::read_u16_at(chunk, pos, endian)?;
            let format = Self::read_u16_at(chunk, pos + 2, endian)?;
            let count = Self::read_u32_at(chunk, pos + 4, endian)?;
            let value = Self::read_u16_at(chunk, pos + 8, endian)?;
            pos += 12; // Each IFD entry is 12 bytes
            if tag == 0x112 && format == 3 && count == 1 {
                let offset = (pos - 4) as u64; // points back to the value field
                let orientation = Self::from_exif(value.min(255) as u8);
                return orientation.map(|orient| (orient, offset));
            }
        }
        // If we reached this point without returning early, there was no orientation
        None
    }
}

#[derive(Debug, Copy, Clone)]
enum ExifEndian {
    Big,
    Little,
}

/// The number of times animated image should loop over.
#[derive(Clone, Copy)]
pub enum LoopCount {
    /// Loop the image Infinitely
    Infinite,
    /// Loop the image within Finite times.
    Finite(NonZeroU32),
}

#[cfg(all(test, feature = "jpeg"))]
mod tests {
    use crate::{codecs::jpeg::JpegDecoder, ImageDecoder as _};
    use std::io::Cursor;

    // This brings all the items from the parent module into scope,
    // so you can directly use `add` instead of `super::add`.
    use super::*;

    const TEST_IMAGE: &[u8] = include_bytes!("../tests/images/jpg/portrait_2.jpg");

    #[test] // This attribute marks the function as a test function.
    fn test_extraction_and_clearing() {
        let reader = Cursor::new(TEST_IMAGE);
        let mut decoder = JpegDecoder::new(reader).expect("Failed to decode test image");
        let mut exif_chunk = decoder
            .exif_metadata()
            .expect("Failed to extract Exif chunk")
            .expect("No Exif chunk found in test image");

        let orientation = Orientation::from_exif_chunk(&exif_chunk)
            .expect("Failed to extract orientation from Exif chunk");
        assert_eq!(orientation, Orientation::FlipHorizontal);

        let orientation = Orientation::remove_from_exif_chunk(&mut exif_chunk)
            .expect("Failed to remove orientation from Exif chunk");
        assert_eq!(orientation, Orientation::FlipHorizontal);
        // Now that the orientation has been cleared, any subsequent extractions should return NoTransforms
        let orientation = Orientation::from_exif_chunk(&exif_chunk)
            .expect("Failed to extract orientation from Exif chunk after clearing it");
        assert_eq!(orientation, Orientation::NoTransforms);
    }
}
