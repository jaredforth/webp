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
    pub fn from_image(image: &'a DynamicImage) -> Result<Self, &str> {
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
            let res = encode(&mut *picture, config);
            res
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
        PixelLayout::Rgba => {
            WebPPictureImportRGBA(&mut picture, image.as_ptr(), width as i32 * 4);
        }
        PixelLayout::Rgb => {
            WebPPictureImportRGB(&mut picture, image.as_ptr(), width as i32 * 3);
        }
    }
    ManageedPicture(picture)
}
unsafe fn encode(
    picture: &mut WebPPicture,
    config: &WebPConfig,
) -> Result<WebPMemory, WebPEncodingError> {
    if WebPValidateConfig(config) == 0 {
        return Err(WebPEncodingError::VP8_ENC_ERROR_INVALID_CONFIGURATION);
    }
    let mut ww = std::mem::MaybeUninit::uninit();
    WebPMemoryWriterInit(ww.as_mut_ptr());
    picture.writer = Some(WebPMemoryWrite);
    picture.custom_ptr = ww.as_mut_ptr() as *mut std::ffi::c_void;
    let status = libwebp_sys::WebPEncode(config, picture);
    let ww = ww.assume_init();
    let mem = WebPMemory(ww.mem, ww.size as usize);
    if status != VP8StatusCode::VP8_STATUS_OK as i32 {
        Ok(mem)
    } else {
        Err(picture.error_code)
    }
}
