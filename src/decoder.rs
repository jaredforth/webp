use libwebp_sys::*;

use std::ops::Deref;

#[cfg(feature = "image-conversion")]
use image::*;
use std::fmt::{Debug, Formatter, Error};


pub struct WebPImage<'a> {
    features: BitstreamFeatures,
    data: &'a mut [u8],
}

impl<'a> WebPImage<'a> {
    #[cfg(feature = "image-conversion")]
    pub fn as_image(&self) -> DynamicImage {
        if self.features.has_alpha() {
            let image = ImageBuffer::from_raw(
                self.features.width(),
                self.features.height(),
                self.data.to_owned(),
            ).expect("ImageBuffer couldn't be created");

            DynamicImage::ImageRgba8(image)
        } else {
            let image = ImageBuffer::from_raw(
                self.features.width(),
                self.features.height(),
                self.data.to_owned(),
            ).expect("ImageBuffer couldn't be created");

            DynamicImage::ImageRgb8(image)
        }
    }

    pub fn from_data(data: &'a [u8]) -> Option<Self> {
        let features = BitstreamFeatures::new(data)?;

        if features.has_animation() {
            unimplemented!()
        }

        let data = unsafe { Self::decode(data, &features)? };

        let image = Self {
            features,
            data,
        };

        Some(image)
    }

    unsafe fn decode(data: &'a [u8], features: &BitstreamFeatures) -> Option<&'a mut [u8]> {
        let width = features.width();
        let height = features.height();
        let pixel_count = width * height;

        let mut width = width as i32;
        let mut height = height as i32;

        let image_ptr = if features.has_alpha() {
            WebPDecodeRGBA(
                data.as_ptr(),
                data.len(),
                &mut width as *mut _,
                &mut height as *mut _,
            )
        } else {
            WebPDecodeRGB(
                data.as_ptr(),
                data.len(),
                &mut width as *mut _,
                &mut height as *mut _,
            )
        };

        if image_ptr.is_null() {
            return None;
        }

        if features.has_alpha() {
            Some(std::slice::from_raw_parts_mut(image_ptr, 4 * pixel_count as usize))
        } else {
            Some(std::slice::from_raw_parts_mut(image_ptr, 3 * pixel_count as usize))
        }
    }
}

impl<'a> Deref for WebPImage<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a> Drop for WebPImage<'a> {
    fn drop(&mut self) {
        unsafe {
            WebPFree(self.data.as_mut_ptr() as _)
        }
    }
}

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

    pub fn width(&self) -> u32 {
        self.0.width as u32
    }

    pub fn height(&self) -> u32 {
        self.0.height as u32
    }

    pub fn has_alpha(&self) -> bool {
        self.0.has_alpha == 1
    }

    pub fn has_animation(&self) -> bool {
        self.0.has_animation == 1
    }

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
pub enum BitstreamFormat {
    Undefined = 0,
    Lossy = 1,
    Lossless = 2,
}