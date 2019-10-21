use libwebp_sys::*;

use crate::shared::*;

pub struct Encoder<'a> {
    image: &'a [u8],
    color: Channels,
    width: u32,
    height: u32,
}

impl<'a> Encoder<'a> {
    pub fn from_rgb(image: &'a [u8], width: u32, height: u32) -> Self {
        Self { image, color: Channels::Rgb, width, height }
    }

    pub fn from_rgba(image: &'a [u8], width: u32, height: u32) -> Self {
        Self { image, color: Channels::Rgba, width, height }
    }

    pub fn encode(&self, quality: f32) -> WebPMemory<'a> {
        let image = unsafe { encode(self.image, self.color, self.width, self.height, quality) };
        WebPMemory(image)
    }

    pub fn encode_lossless(&self) -> WebPMemory<'a> {
        let image = unsafe { encode(self.image, self.color, self.width, self.height, -1.0) };
        WebPMemory(image)
    }
}

unsafe fn encode(image: &[u8], color: Channels, width: u32, height: u32, quality: f32) -> &mut [u8] {
    let width = width as _;
    let height = height as _;

    let mut out_buf = std::ptr::null_mut::<u8>();

    let len = match color {
        Channels::Rgb if quality < 0.0 => {
            let stride = width * 3;
            WebPEncodeLosslessRGB(image.as_ptr(), width, height, stride, &mut out_buf as *mut _)
        }
        Channels::Rgb => {
            let stride = width * 3;
            WebPEncodeRGB(image.as_ptr(), width, height, stride, quality, &mut out_buf as *mut _)
        }
        Channels::Rgba if quality < 0.0 => {
            let stride = width * 4;
            WebPEncodeLosslessBGRA(image.as_ptr(), width, height, stride, &mut out_buf as *mut _)
        }
        Channels::Rgba => {
            let stride = width * 4;
            WebPEncodeRGBA(image.as_ptr(), width, height, stride, quality, &mut out_buf as *mut _)
        }
    };

    return std::slice::from_raw_parts_mut(out_buf, len);
}