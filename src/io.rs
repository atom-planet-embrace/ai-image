//! Input and output of images.

#[cfg(any(feature = "tga", feature = "dds", feature = "bmp"))]
use no_std_io::io;

/// The decoder traits.
pub(crate) mod decoder;
/// The encoder traits.
pub(crate) mod encoder;

pub(crate) mod format;
pub(crate) mod free_functions;
#[cfg(feature = "std")]
pub(crate) mod image_reader_type;
pub(crate) mod limits;

#[cfg(feature = "std")]
#[deprecated(note = "this type has been moved and renamed to image::ImageReader")]
/// Deprecated re-export of `ImageReader` as `Reader`
pub type Reader<R> = ImageReader<R>;
#[deprecated(note = "this type has been moved to image::Limits")]
/// Deprecated re-export of `Limits`
pub type Limits = limits::Limits;
#[deprecated(note = "this type has been moved to image::LimitSupport")]
/// Deprecated re-export of `LimitSupport`
pub type LimitSupport = limits::LimitSupport;

#[cfg(feature = "std")]
pub(crate) use self::image_reader_type::ImageReader;

/// Adds `read_exact_vec`
#[cfg(any(feature = "tga", feature = "dds", feature = "bmp"))]
pub(crate) trait ReadExt {
    fn read_exact_vec(&mut self, vec: &mut alloc::vec::Vec<u8>, len: usize) -> io::Result<()>;
}

#[cfg(any(feature = "tga", feature = "dds", feature = "bmp"))]
impl<R: io::Read> ReadExt for R {
    fn read_exact_vec(&mut self, vec: &mut alloc::vec::Vec<u8>, len: usize) -> io::Result<()> {
        let initial_len = vec.len();
        // no_std_io::Error doesn't provide other()
        #[allow(clippy::io_other_error)]
        vec.try_reserve(len)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "allocation failed"))?;
        vec.resize(initial_len + len, 0);
        match self.read_exact(&mut vec[initial_len..]) {
            Ok(()) => Ok(()),
            Err(e) => {
                vec.truncate(initial_len);
                Err(e)
            }
        }
    }
}
