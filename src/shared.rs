use std::fmt::{Debug, Error, Formatter};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "img")]
use image::*;
use libwebp_sys::{WebPFree, WebPPicture, WebPPictureFree};

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
        unsafe { WebPFree(self.0 as _) }
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

#[derive(Debug)]
pub(crate) struct ManageedPicture(pub(crate) WebPPicture);

impl Drop for ManageedPicture {
    fn drop(&mut self) {
        unsafe { WebPPictureFree(&mut self.0 as _) }
    }
}

impl Deref for ManageedPicture {
    type Target = WebPPicture;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ManageedPicture {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// This struct represents a decoded image.
/// Its data contents can be accessed through the Deref and DerefMut traits.
/// It is also possible to create an image::DynamicImage from this struct.
pub struct WebPImage {
    data: WebPMemory,
    layout: PixelLayout,
    width: u32,
    height: u32,
}

impl WebPImage {
    pub(crate) fn new(data: WebPMemory, layout: PixelLayout, width: u32, height: u32) -> Self {
        Self {
            data,
            layout,
            width,
            height,
        }
    }

    /// Creates a DynamicImage from this WebPImage.
    #[cfg(feature = "img")]
    pub fn to_image(&self) -> DynamicImage {
        if self.layout.is_alpha() {
            let image = ImageBuffer::from_raw(self.width, self.height, self.data.to_owned())
                .expect("ImageBuffer couldn't be created");

            DynamicImage::ImageRgba8(image)
        } else {
            let image = ImageBuffer::from_raw(self.width, self.height, self.data.to_owned())
                .expect("ImageBuffer couldn't be created");

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

    pub fn is_alpha(&self) -> bool {
        self.layout.is_alpha()
    }

    pub fn layout(&self) -> PixelLayout {
        self.layout
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

/// Describes the pixel layout (the order of the color channels) of an image.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PixelLayout {
    Rgb,
    Rgba,
}

impl PixelLayout {
    /// Returns true if the pixel contains an alpha channel.
    pub fn is_alpha(self) -> bool {
        self == PixelLayout::Rgba
    }

    pub fn bytes_per_pixel(self) -> u8 {
        match self {
            PixelLayout::Rgb => 3,
            PixelLayout::Rgba => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_layout_is_alpha() {
        assert!(!PixelLayout::Rgb.is_alpha());
        assert!(PixelLayout::Rgba.is_alpha());
    }

    #[test]
    fn test_webpimage_accessors() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80];

        let mut boxed = data.clone().into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        let len = boxed.len();
        std::mem::forget(boxed);

        let mem = WebPMemory(ptr, len);
        let img = WebPImage::new(mem, PixelLayout::Rgba, 2, 1);

        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 1);
        assert!(img.is_alpha());
        assert_eq!(img.layout(), PixelLayout::Rgba);

        assert_eq!(&img[..], &data[..]);
    }

    #[test]
    fn test_webpimage_deref_mut() {
        let data = vec![1, 2, 3, 4];
        let mut boxed = data.clone().into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        let len = boxed.len();
        std::mem::forget(boxed);

        let mem = WebPMemory(ptr, len);
        let mut img = WebPImage::new(mem, PixelLayout::Rgb, 2, 1);

        img.deref_mut()[0] = 42;
        assert_eq!(img[0], 42);
    }

    #[test]
    fn test_webpmemory_drop_calls_webpfree() {
        let data = vec![1, 2, 3, 4];
        let mut boxed = data.clone().into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        let len = boxed.len();
        std::mem::forget(boxed);

        let _mem = WebPMemory(ptr, len);
    }

    #[test]
    fn test_pixel_layout_equality() {
        assert_eq!(PixelLayout::Rgb, PixelLayout::Rgb);
        assert_ne!(PixelLayout::Rgb, PixelLayout::Rgba);
    }

    #[test]
    fn test_webpmemory_debug_exact() {
        let data = vec![1u8, 2, 3];
        let mut boxed = data.into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        let len = boxed.len();
        std::mem::forget(boxed);

        let mem = WebPMemory(ptr, len);

        let dbg_str = format!("{mem:?}");

        assert_eq!(dbg_str, "WebpMemory");
    }

    #[test]
    fn test_manageedpicture_deref() {
        let pic = unsafe { std::mem::zeroed::<WebPPicture>() };
        let managed = ManageedPicture(pic);

        let inner_ref: &WebPPicture = &managed;
        let orig_ptr = &managed.0 as *const WebPPicture;
        let deref_ptr = inner_ref as *const WebPPicture;
        assert_eq!(orig_ptr, deref_ptr);
    }
}
