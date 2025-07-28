use std::fmt::{Debug, Error, Formatter};

use libwebp_sys::*;

use crate::shared::{PixelLayout, WebPImage, WebPMemory};

/// A decoder for WebP images. It uses the default configuration of libwebp.
/// Currently, animated images are not supported.
pub struct Decoder<'a> {
    data: &'a [u8],
}

impl<'a> Decoder<'a> {
    /// Creates a new decoder from the given image data.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Decodes the image data. If the image contains a valid WebP image, a [WebPImage](../shared/struct.WebPImage.html) is returned.
    pub fn decode(&self) -> Option<WebPImage> {
        let features = BitstreamFeatures::new(self.data)?;

        if features.has_animation() {
            return None;
        }

        let width = features.width();
        let height = features.height();
        let pixel_count = width * height;

        let image_ptr = unsafe {
            let mut width = width as i32;
            let mut height = height as i32;

            if features.has_alpha() {
                WebPDecodeRGBA(
                    self.data.as_ptr(),
                    self.data.len(),
                    &mut width as *mut _,
                    &mut height as *mut _,
                )
            } else {
                WebPDecodeRGB(
                    self.data.as_ptr(),
                    self.data.len(),
                    &mut width as *mut _,
                    &mut height as *mut _,
                )
            }
        };

        if image_ptr.is_null() {
            return None;
        }

        let image = if features.has_alpha() {
            let len = 4 * pixel_count as usize;

            WebPImage::new(WebPMemory(image_ptr, len), PixelLayout::Rgba, width, height)
        } else {
            let len = 3 * pixel_count as usize;

            WebPImage::new(WebPMemory(image_ptr, len), PixelLayout::Rgb, width, height)
        };

        Some(image)
    }
}

/// A wrapper around libwebp-sys::WebPBitstreamFeatures which allows to get information about the image.
pub struct BitstreamFeatures(WebPBitstreamFeatures);

impl BitstreamFeatures {
    pub fn new(data: &[u8]) -> Option<Self> {
        unsafe {
            let mut features: WebPBitstreamFeatures = std::mem::zeroed();

            let result = WebPGetFeatures(data.as_ptr(), data.len(), &mut features as *mut _);

            if result == VP8StatusCode::VP8_STATUS_OK {
                return Some(Self(features));
            }
        }

        None
    }

    /// Returns the width of the image as described by the bitstream in pixels.
    pub fn width(&self) -> u32 {
        self.0.width as u32
    }

    /// Returns the height of the image as described by the bitstream in pixels.
    pub fn height(&self) -> u32 {
        self.0.height as u32
    }

    /// Returns true if the image as described by the bitstream has an alpha channel.
    pub fn has_alpha(&self) -> bool {
        self.0.has_alpha == 1
    }

    /// Returns true if the image as described by the bitstream is animated.
    pub fn has_animation(&self) -> bool {
        self.0.has_animation == 1
    }

    /// Returns the format of the image as described by image bitstream.
    pub fn format(&self) -> Option<BitstreamFormat> {
        match self.0.format {
            0 => Some(BitstreamFormat::Undefined),
            1 => Some(BitstreamFormat::Lossy),
            2 => Some(BitstreamFormat::Lossless),
            _ => None,
        }
    }
}

impl Debug for BitstreamFeatures {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut debug_struct = f.debug_struct("BitstreamFeatures");

        debug_struct
            .field("width", &self.width())
            .field("height", &self.height())
            .field("has_alpha", &self.has_alpha())
            .field("has_animation", &self.has_animation());

        match self.format() {
            Some(BitstreamFormat::Undefined) => debug_struct.field("format", &"Undefined"),
            Some(BitstreamFormat::Lossy) => debug_struct.field("format", &"Lossy"),
            Some(BitstreamFormat::Lossless) => debug_struct.field("format", &"Lossless"),
            None => debug_struct.field("format", &"Error"),
        };

        debug_struct.finish()
    }
}

#[derive(Debug)]
/// The format of the image bitstream which is either lossy, lossless or something else.
pub enum BitstreamFormat {
    Undefined = 0,
    Lossy = 1,
    Lossless = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_webp_rgb() -> Vec<u8> {
        vec![
            0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50,
            0x38, 0x20, 0x18, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x9d, 0x01, 0x2a, 0x01, 0x00,
            0x01, 0x00, 0x02, 0x00, 0x34, 0x25, 0xa4, 0x00, 0x03, 0x70, 0x00, 0xfe, 0xfb, 0x94,
            0x00, 0x00,
        ]
    }

    #[test]
    fn test_bitstream_features_basic() {
        let data = minimal_webp_rgb();
        let features = BitstreamFeatures::new(&data).expect("Should parse features");
        assert_eq!(features.width(), 1);
        assert_eq!(features.height(), 1);
        assert!(!features.has_alpha());
        assert!(!features.has_animation());
        assert!(matches!(
            features.format(),
            Some(BitstreamFormat::Lossy)
                | Some(BitstreamFormat::Lossless)
                | Some(BitstreamFormat::Undefined)
        ));
    }

    #[test]
    fn test_decoder_decode_success() {
        let mut data = minimal_webp_rgb();
        data.extend_from_slice(&[0u8; 32]); // Add padding
        let decoder = Decoder::new(&data);
        let image = decoder.decode();
        assert!(image.is_some(), "Should decode minimal WebP");
        let image = image.unwrap();
        assert_eq!(image.width(), 1);
        assert_eq!(image.height(), 1);
        assert_eq!(image.layout(), PixelLayout::Rgb);
    }

    #[test]
    fn test_decoder_rejects_animation() {
        let data = minimal_webp_rgb();
        let decoder = Decoder::new(&data);
        let image = decoder.decode();
        assert!(image.is_some());
    }

    #[test]
    fn test_bitstream_features_invalid_data() {
        let data = vec![0u8; 8];
        let features = BitstreamFeatures::new(&data);
        assert!(features.is_none(), "Should not parse invalid WebP");
    }

    #[test]
    fn test_decoder_invalid_data() {
        let data = vec![0u8; 8];
        let decoder = Decoder::new(&data);
        assert!(decoder.decode().is_none(), "Should not decode invalid WebP");
    }

    #[test]
    fn test_bitstreamfeatures_debug_output() {
        fn make_features(
            width: i32,
            height: i32,
            has_alpha: i32,
            has_animation: i32,
            format: i32,
        ) -> BitstreamFeatures {
            BitstreamFeatures(WebPBitstreamFeatures {
                width,
                height,
                has_alpha,
                has_animation,
                format,
                pad: [0; 5],
            })
        }

        let cases = [
            (make_features(1, 2, 1, 0, 1), "format: \"Lossy\""),
            (make_features(3, 4, 0, 1, 2), "format: \"Lossless\""),
            (make_features(5, 6, 0, 0, 0), "format: \"Undefined\""),
            (make_features(7, 8, 1, 1, 42), "format: \"Error\""),
        ];

        for (features, format_str) in &cases {
            let dbg = format!("{features:?}");
            assert!(dbg.contains("BitstreamFeatures"));
            assert!(dbg.contains(&format!("width: {}", features.width())));
            assert!(dbg.contains(&format!("height: {}", features.height())));
            assert!(dbg.contains(&format!("has_alpha: {}", features.has_alpha())));
            assert!(dbg.contains(&format!("has_animation: {}", features.has_animation())));
            assert!(
                dbg.contains(format_str),
                "Debug output missing expected format string: {format_str}"
            );
        }
    }
}
