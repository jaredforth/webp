#[cfg(feature = "img")]
use image::DynamicImage;
use libwebp_sys::*;

use crate::shared::*;

/// An encoder for WebP images. It uses the default configuration of libwebp.
pub struct Encoder<'a> {
    image: &'a [u8],
    layout: PixelLayout,
    width: u32,
    height: u32,
}

impl<'a> Encoder<'a> {
    /// Creates a new encoder from the given image data.
    /// The image data must be in the pixel layout of the color parameter.
    pub fn new(image: &'a [u8], layout: PixelLayout, width: u32, height: u32) -> Self {
        Self {
            image,
            layout,
            width,
            height,
        }
    }

    #[cfg(feature = "img")]
    /// Creates a new encoder from the given image.
    pub fn from_image(image: &'a DynamicImage) -> Result<Self, &'a str> {
        match image {
            DynamicImage::ImageLuma8(_) => Err("Unimplemented"),
            DynamicImage::ImageLumaA8(_) => Err("Unimplemented"),
            DynamicImage::ImageRgb8(image) => Ok(Self::from_rgb(
                image.as_ref(),
                image.width(),
                image.height(),
            )),
            DynamicImage::ImageRgba8(image) => Ok(Self::from_rgba(
                image.as_ref(),
                image.width(),
                image.height(),
            )),
            _ => Err("Unimplemented"),
        }
    }

    /// Creates a new encoder from the given image data in the RGB pixel layout.
    pub fn from_rgb(image: &'a [u8], width: u32, height: u32) -> Self {
        Self {
            image,
            layout: PixelLayout::Rgb,
            width,
            height,
        }
    }

    /// Creates a new encoder from the given image data in the RGBA pixel layout.
    pub fn from_rgba(image: &'a [u8], width: u32, height: u32) -> Self {
        Self {
            image,
            layout: PixelLayout::Rgba,
            width,
            height,
        }
    }

    /// Encode the image with the given quality.
    /// The image quality must be between 0.0 and 100.0 inclusive for minimal and maximal quality respectively.
    pub fn encode(&self, quality: f32) -> WebPMemory {
        self.encode_simple(false, quality).unwrap()
    }

    /// Encode the image losslessly.
    pub fn encode_lossless(&self) -> WebPMemory {
        self.encode_simple(true, 75.0).unwrap()
    }

    pub fn encode_simple(
        &self,
        lossless: bool,
        quality: f32,
    ) -> Result<WebPMemory, WebPEncodingError> {
        let mut config = WebPConfig::new().unwrap();
        config.lossless = if lossless { 1 } else { 0 };
        config.alpha_compression = if lossless { 0 } else { 1 };
        config.quality = quality;
        self.encode_advanced(&config)
    }

    pub fn encode_advanced(&self, config: &WebPConfig) -> Result<WebPMemory, WebPEncodingError> {
        unsafe {
            let mut picture = new_picture(self.image, self.layout, self.width, self.height);
            encode(&mut picture, config)
        }
    }
}

pub(crate) unsafe fn new_picture(
    image: &[u8],
    layout: PixelLayout,
    width: u32,
    height: u32,
) -> ManageedPicture {
    let mut picture = WebPPicture::new().unwrap();
    picture.use_argb = 1;
    picture.width = width as i32;
    picture.height = height as i32;
    match layout {
        PixelLayout::Rgba => unsafe {
            WebPPictureImportRGBA(&mut picture, image.as_ptr(), width as i32 * 4);
        },
        PixelLayout::Rgb => unsafe {
            WebPPictureImportRGB(&mut picture, image.as_ptr(), width as i32 * 3);
        },
    }
    ManageedPicture(picture)
}
unsafe fn encode(
    picture: &mut WebPPicture,
    config: &WebPConfig,
) -> Result<WebPMemory, WebPEncodingError> {
    unsafe {
        if WebPValidateConfig(config) == 0 {
            return Err(WebPEncodingError::VP8_ENC_ERROR_INVALID_CONFIGURATION);
        }
        let mut ww = std::mem::MaybeUninit::uninit();
        WebPMemoryWriterInit(ww.as_mut_ptr());
        picture.writer = Some(WebPMemoryWrite);
        picture.custom_ptr = ww.as_mut_ptr() as *mut std::ffi::c_void;
        let status = libwebp_sys::WebPEncode(config, picture);
        let ww = ww.assume_init();
        let mem = WebPMemory(ww.mem, ww.size);

        if status != VP8StatusCode::VP8_STATUS_OK as i32 {
            Ok(mem)
        } else {
            Err(picture.error_code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared;

    #[test]
    fn test_encoder_new_assigns_fields() {
        let data = [1, 2, 3, 4, 5, 6];
        let enc = Encoder::new(&data, shared::PixelLayout::Rgb, 2, 3);
        assert_eq!(enc.image, &data);
        assert_eq!(enc.layout, shared::PixelLayout::Rgb);
        assert_eq!(enc.width, 2);
        assert_eq!(enc.height, 3);
    }

    #[test]
    fn test_encoder_from_rgb_and_rgba() {
        let rgb = [10, 20, 30, 40, 50, 60];
        let rgba = [1, 2, 3, 4, 5, 6, 7, 8];
        let enc_rgb = Encoder::from_rgb(&rgb, 2, 1);
        let enc_rgba = Encoder::from_rgba(&rgba, 2, 1);
        assert_eq!(enc_rgb.layout, shared::PixelLayout::Rgb);
        assert_eq!(enc_rgba.layout, shared::PixelLayout::Rgba);
        assert_eq!(enc_rgb.image, &rgb);
        assert_eq!(enc_rgba.image, &rgba);
        assert_eq!(enc_rgb.width, 2);
        assert_eq!(enc_rgba.height, 1);
    }

    #[cfg(feature = "img")]
    #[test]
    fn test_encoder_from_image_error_branches() {
        use image::{DynamicImage, ImageBuffer};

        let luma = DynamicImage::ImageLuma8(ImageBuffer::from_pixel(1, 1, image::Luma([0])));
        let luma_a = DynamicImage::ImageLumaA8(ImageBuffer::from_pixel(1, 1, image::LumaA([0, 0])));
        assert!(Encoder::from_image(&luma).is_err());
        assert!(Encoder::from_image(&luma_a).is_err());

        let rgb = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(2, 2, image::Rgb([1, 2, 3])));
        let rgba =
            DynamicImage::ImageRgba8(ImageBuffer::from_pixel(2, 2, image::Rgba([1, 2, 3, 4])));
        assert!(Encoder::from_image(&rgb).is_ok());
        assert!(Encoder::from_image(&rgba).is_ok());
    }

    #[test]
    fn test_encode_runs_without_panic() {
        let width = 2;
        let height = 2;
        let image = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0];
        let encoder = Encoder::new(&image, PixelLayout::Rgb, width, height);

        let mem = encoder.encode(75.0);
        assert!(!mem.is_empty());
    }
}
