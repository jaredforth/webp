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

            WebPImage::new(
                WebPMemory(image_ptr, len),
                PixelLayout::Rgba,
                width,
                height,
            )
        } else {
            let len = 3 * pixel_count as usize;

            WebPImage::new(
                WebPMemory(image_ptr, len),
                PixelLayout::Rgb,
                width,
                height,
            )
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

            // WebPGetFeatures is not available in libwebp-sys
            #[allow(non_snake_case)]
            unsafe fn WebPGetFeatures(
                data: *const u8,
                data_size: usize,
                features: *mut WebPBitstreamFeatures) -> VP8StatusCode {
                WebPGetFeaturesInternal(
                    data,
                    data_size,
                    features,
                    WEBP_DECODER_ABI_VERSION as i32,
                )
            }

            let result = WebPGetFeatures(
                data.as_ptr(),
                data.len(),
                &mut features as *mut _,
            );

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