#[cfg(feature = "img")]
use image::{DynamicImage, ImageBuffer};
use libwebp_sys::*;

use crate::{shared::*, Encoder};

pub struct AnimFrame<'a> {
    image: &'a [u8],
    layout: PixelLayout,
    width: u32,
    height: u32,
    timestamp: i32,
    config: Option<&'a WebPConfig>,
}
impl<'a> AnimFrame<'a> {
    pub fn new(
        image: &'a [u8],
        layout: PixelLayout,
        width: u32,
        height: u32,
        timestamp: i32,
        config: Option<&'a WebPConfig>,
    ) -> Self {
        Self {
            image,
            layout,
            width,
            height,
            timestamp,
            config,
        }
    }
    #[cfg(feature = "img")]
    pub fn from_image(image: &'a DynamicImage, timestamp: i32) -> Result<Self, &'static str> {
        match image {
            DynamicImage::ImageLuma8(_) => Err("Unimplemented"),
            DynamicImage::ImageLumaA8(_) => Err("Unimplemented"),
            DynamicImage::ImageRgb8(image) => Ok(Self::from_rgb(
                image.as_ref(),
                image.width(),
                image.height(),
                timestamp,
            )),
            DynamicImage::ImageRgba8(image) => Ok(Self::from_rgba(
                image.as_ref(),
                image.width(),
                image.height(),
                timestamp,
            )),
            _ => Err("Unimplemented"),
        }
    }
    /// Creates a new encoder from the given image data in the RGB pixel layout.
    pub fn from_rgb(image: &'a [u8], width: u32, height: u32, timestamp: i32) -> Self {
        Self::new(image, PixelLayout::Rgb, width, height, timestamp, None)
    }
    /// Creates a new encoder from the given image data in the RGBA pixel layout.
    pub fn from_rgba(image: &'a [u8], width: u32, height: u32, timestamp: i32) -> Self {
        Self::new(image, PixelLayout::Rgba, width, height, timestamp, None)
    }
    pub fn get_image(&self) -> &[u8] {
        self.image
    }
    pub fn get_layout(&self) -> PixelLayout {
        self.layout
    }
    pub fn get_time_ms(&self) -> i32 {
        self.timestamp
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}
impl<'a> From<&'a AnimFrame<'a>> for Encoder<'a> {
    fn from(f: &'a AnimFrame) -> Self {
        Encoder::new(f.get_image(), f.layout, f.width, f.height)
    }
}
#[cfg(feature = "img")]
impl From<&AnimFrame<'_>> for DynamicImage {
    fn from(value: &AnimFrame<'_>) -> DynamicImage {
        if value.layout.is_alpha() {
            let image =
                ImageBuffer::from_raw(value.width(), value.height(), value.get_image().to_owned())
                    .expect("ImageBuffer couldn't be created");
            DynamicImage::ImageRgba8(image)
        } else {
            let image =
                ImageBuffer::from_raw(value.width(), value.height(), value.get_image().to_owned())
                    .expect("ImageBuffer couldn't be created");
            DynamicImage::ImageRgb8(image)
        }
    }
}
pub struct AnimEncoder<'a> {
    frames: Vec<AnimFrame<'a>>,
    width: u32,
    height: u32,
    config: &'a WebPConfig,
    muxparams: WebPMuxAnimParams,
}
impl<'a> AnimEncoder<'a> {
    pub fn new(width: u32, height: u32, config: &'a WebPConfig) -> Self {
        Self {
            frames: vec![],
            width,
            height,
            config,
            muxparams: WebPMuxAnimParams {
                bgcolor: 0,
                loop_count: 0,
            },
        }
    }
    pub fn set_bgcolor(&mut self, rgba: [u8; 4]) {
        let bgcolor = (u32::from(rgba[3]) << 24)
            + (u32::from(rgba[2]) << 16)
            + (u32::from(rgba[1]) << 8)
            + (u32::from(rgba[0]));
        self.muxparams.bgcolor = bgcolor;
    }
    pub fn set_loop_count(&mut self, loop_count: i32) {
        self.muxparams.loop_count = loop_count;
    }
    pub fn add_frame(&mut self, frame: AnimFrame<'a>) {
        self.frames.push(frame);
    }
    pub fn encode(&self) -> WebPMemory {
        self.try_encode().unwrap()
    }
    pub fn try_encode(&self) -> Result<WebPMemory, AnimEncodeError> {
        unsafe { anim_encode(self) }
    }
}

#[derive(Debug)]
pub enum AnimEncodeError {
    WebPEncodingError(WebPEncodingError),
    WebPMuxError(WebPMuxError),
    WebPAnimEncoderGetError(String),
}
unsafe fn anim_encode(all_frame: &AnimEncoder) -> Result<WebPMemory, AnimEncodeError> {
    let width = all_frame.width;
    let height = all_frame.height;
    let mut uninit = std::mem::MaybeUninit::<WebPAnimEncoderOptions>::uninit();

    let mux_abi_version = WebPGetMuxABIVersion();
    WebPAnimEncoderOptionsInitInternal(uninit.as_mut_ptr(), mux_abi_version);
    let encoder = WebPAnimEncoderNewInternal(
        width as i32,
        height as i32,
        uninit.as_ptr(),
        mux_abi_version,
    );
    let mut frame_pictures = vec![];
    for frame in all_frame.frames.iter() {
        let mut pic = crate::new_picture(frame.image, frame.layout, width, height);
        let config = frame.config.unwrap_or(all_frame.config);
        let ok = WebPAnimEncoderAdd(
            encoder,
            &mut *pic as *mut _,
            frame.timestamp as std::os::raw::c_int,
            config,
        );
        if ok == 0 {
            //ok == false
            WebPAnimEncoderDelete(encoder);
            return Err(AnimEncodeError::WebPEncodingError(pic.error_code));
        }
        frame_pictures.push(pic);
    }
    WebPAnimEncoderAdd(encoder, std::ptr::null_mut(), 0, std::ptr::null());

    let mut webp_data = std::mem::MaybeUninit::<WebPData>::uninit();
    let ok = WebPAnimEncoderAssemble(encoder, webp_data.as_mut_ptr());
    if ok == 0 {
        let err_ptr = WebPAnimEncoderGetError(encoder);
        let string = if !err_ptr.is_null() {
            unsafe { std::ffi::CStr::from_ptr(err_ptr) }
                .to_string_lossy()
                .into_owned()
        } else {
            String::from("Unknown error")
        };
        WebPAnimEncoderDelete(encoder);
        return Err(AnimEncodeError::WebPAnimEncoderGetError(string));
    }
    WebPAnimEncoderDelete(encoder);
    let mux = WebPMuxCreateInternal(webp_data.as_ptr(), 1, mux_abi_version);
    let mux_error = WebPMuxSetAnimationParams(mux, &all_frame.muxparams);
    if mux_error != WebPMuxError::WEBP_MUX_OK {
        return Err(AnimEncodeError::WebPMuxError(mux_error));
    }
    let mut raw_data: WebPData = webp_data.assume_init();
    WebPDataClear(&mut raw_data);
    let mut webp_data = std::mem::MaybeUninit::<WebPData>::uninit();
    WebPMuxAssemble(mux, webp_data.as_mut_ptr());
    WebPMuxDelete(mux);
    let raw_data: WebPData = webp_data.assume_init();
    Ok(WebPMemory(raw_data.bytes as *mut u8, raw_data.size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::PixelLayout;
    use crate::AnimDecoder;

    fn default_config() -> WebPConfig {
        let mut config = unsafe { std::mem::zeroed() };
        let ok = unsafe {
            WebPConfigInitInternal(
                &mut config,
                WebPPreset::WEBP_PRESET_DEFAULT,
                75.0,
                WEBP_ENCODER_ABI_VERSION as i32,
            )
        };
        assert_ne!(ok, 0, "WebPConfigInitInternal failed");
        config
    }

    #[test]
    fn test_animframe_new_and_accessors() {
        let img = [255u8, 0, 0, 255, 0, 255, 0, 255];
        let frame = AnimFrame::new(&img, PixelLayout::Rgba, 2, 1, 42, None);
        assert_eq!(frame.get_image(), &img);
        assert_eq!(frame.get_layout(), PixelLayout::Rgba);
        assert_eq!(frame.width(), 2);
        assert_eq!(frame.height(), 1);
        assert_eq!(frame.get_time_ms(), 42);
    }

    #[test]
    fn test_animframe_from_rgb_and_rgba() {
        let rgb = [1u8, 2, 3, 4, 5, 6];
        let rgba = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let f_rgb = AnimFrame::from_rgb(&rgb, 2, 1, 100);
        let f_rgba = AnimFrame::from_rgba(&rgba, 2, 1, 200);
        assert_eq!(f_rgb.get_layout(), PixelLayout::Rgb);
        assert_eq!(f_rgba.get_layout(), PixelLayout::Rgba);
        assert_eq!(f_rgb.get_time_ms(), 100);
        assert_eq!(f_rgba.get_time_ms(), 200);
    }

    #[test]
    fn test_animencoder_add_and_configure() {
        let config = default_config();
        let mut encoder = AnimEncoder::new(2, 1, &config);
        encoder.set_bgcolor([1, 2, 3, 4]);
        encoder.set_loop_count(3);

        let frame = AnimFrame::from_rgb(&[1, 2, 3, 4, 5, 6], 2, 1, 0);
        encoder.add_frame(frame);

        assert_eq!(encoder.frames.len(), 1);
        assert_eq!(encoder.width, 2);
        assert_eq!(encoder.height, 1);
        assert_eq!(encoder.muxparams.loop_count, 3);

        let expected_bg = (4u32 << 24) | (3u32 << 16) | (2u32 << 8) | 1u32;
        assert_eq!(encoder.muxparams.bgcolor, expected_bg);
    }

    #[test]
    fn test_animencoder_encode_error_on_empty() {
        let config = default_config();
        let encoder = AnimEncoder::new(2, 1, &config);
        let result = encoder.try_encode();
        assert!(
            result.is_err(),
            "Encoding with no frames should fail or error"
        );
    }

    #[test]
    fn test_animdecoder_decode_failure_on_invalid_data() {
        let data = vec![0u8; 10];
        let decoder = AnimDecoder::new(&data);
        let result = decoder.decode();
        assert!(result.is_err(), "Decoding should fail for invalid data");
    }
}
