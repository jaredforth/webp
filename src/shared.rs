use std::fmt::{Debug, Error, Formatter};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "image-conversion")]
use image::*;
use libwebp_sys::WebPFree;

/// This struct represents a safe wrapper around memory owned by libwebp.
/// Its data contents can be accessed through the Deref and DerefMut traits.
pub struct WebPMemory(pub(crate) *mut u8, pub(crate) usize);

impl Debug for WebPMemory {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("WebpMemory").finish()
    }
}

impl Drop for WebPMemory {
    fn drop(&mut self) {
        unsafe {
            WebPFree(self.0 as _)
        }
    }
}

impl Deref for WebPMemory {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.0, self.1) }
    }
}

impl DerefMut for WebPMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.0, self.1) }
    }
}

/// This struct represents a decoded image.
/// Its data contents can be accessed through the Deref and DerefMut traits.
/// It is also possible to create an image::DynamicImage from this struct.
pub struct WebPImage {
    data: WebPMemory,
    color: Channels,
    width: u32,
    height: u32,
}

impl WebPImage {
    pub(crate) fn new(data: WebPMemory, color: Channels, width: u32, height: u32) -> Self {
        Self { data, color, width, height }
    }

    /// Creates a DynamicImage from this WebPImage.
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

    /// Returns the width of the image in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Deref for WebPImage {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

impl DerefMut for WebPImage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.deref_mut()
    }
}

/// Describes the pixel layout of an image.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Channels {
    Rgb,
    Rgba,
}

impl Channels {
    /// Returns true if the pixel contains an alpha channel.
    pub fn is_alpha(&self) -> bool {
        self == &Channels::Rgba
    }
}