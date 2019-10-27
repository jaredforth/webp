use std::fmt::{Debug, Error, Formatter};
use std::ops::{Deref, DerefMut};

use image::*;
use libwebp_sys::WebPFree;

pub struct WebPMemory<'a>(pub(crate) &'a mut [u8]);

impl<'a> Debug for WebPMemory<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("WebpMemory").finish()
    }
}

impl<'a> Drop for WebPMemory<'a> {
    fn drop(&mut self) {
        unsafe {
            WebPFree(self.0.as_mut_ptr() as _)
        }
    }
}

impl<'a> Deref for WebPMemory<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for WebPMemory<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct WebPImage<'a> {
    data: WebPMemory<'a>,
    color: Channels,
    width: u32,
    height: u32,
}

impl<'a> WebPImage<'a> {
    pub(crate) fn new(data: &'a mut [u8], color: Channels, width: u32, height: u32) -> Self {
        Self { data: WebPMemory(data), color, width, height }
    }

    #[cfg(feature = "image-conversion")]
    pub fn as_image(&self) -> DynamicImage {
        if self.color.is_alpha() {
            let image = ImageBuffer::from_raw(
                self.width,
                self.height,
                self.data.to_owned(),
            ).expect("ImageBuffer couldn't be created");

            DynamicImage::ImageRgba8(image)
        } else {
            let image = ImageBuffer::from_raw(
                self.width,
                self.height,
                self.data.to_owned(),
            ).expect("ImageBuffer couldn't be created");

            DynamicImage::ImageRgb8(image)
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl<'a> Deref for WebPImage<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Channels {
    Rgb,
    Rgba,
}

impl Channels {
    pub fn is_alpha(&self) -> bool {
        self == &Channels::Rgba
    }
}