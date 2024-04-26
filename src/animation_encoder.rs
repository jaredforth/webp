use std::ffi::CString;

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
    pub fn from_image(image: &'a DynamicImage, timestamp: i32) -> Result<Self, &str> {
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
        //ok == false
        let cstring = WebPAnimEncoderGetError(encoder);
        let cstring = CString::from_raw(cstring as *mut _);
        let string = cstring.to_string_lossy().to_string();
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
