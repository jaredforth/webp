//! This crate provides provides functionality for encoding and decoding images from or into the webp format.
//! It is implemented as a safe wrapper around the libwebp-sys crate.
//! Currently only a subset of the features supported by libwebp are available.
//! The simple encoding and decoding apis are implemented which use the default configuration of libwebp.

mod decoder;
#[doc(inline)]
pub use decoder::*;

mod encoder;
#[doc(inline)]
pub use encoder::*;
pub use libwebp_sys::WebPEncodingError;

mod shared;
#[doc(inline)]
pub use shared::*;

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use image::*;

    use crate::*;

    fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [u8; 3] {
        let h = (h - h.floor()) * 6.0;
        let f = h - h.floor();
        let u = (v * 255.0) as u8;
        let p = (v * (1.0 - s) * 255.0) as u8;
        let q = (v * (1.0 - s * f) * 255.0) as u8;
        let t = (v * (1.0 - s * (1.0 - f)) * 255.0) as u8;

        match h as u8 {
            0 => [u, t, p],
            1 => [q, u, p],
            2 => [p, u, t],
            3 => [p, q, u],
            4 => [t, p, u],
            5 => [u, p, q],
            _ => unreachable!("h must be between 0.0 and 1.0"),
        }
    }

    fn generate_color_wheel(width: u32, height: u32, background_alpha: bool) -> DynamicImage {
        let f = |x, y| {
            let width = width as f64;
            let height = height as f64;

            let x = x as f64 - width / 2.0;
            let y = y as f64 - height / 2.0;

            let theta = y.atan2(x);
            let tau = 2.0 * std::f64::consts::PI;
            let h = (theta + std::f64::consts::PI) / tau;
            let s = (4.0 * (x * x + y * y) / width / height).sqrt();
            let v = 1.0;

            if s > 1.0 {
                Rgba([0, 0, 0, if background_alpha { 0 } else { 255 }])
            } else {
                let [r, g, b] = hsv_to_rgb(h, s, v);
                Rgba([r, g, b, 255])
            }
        };

        DynamicImage::ImageRgba8(ImageBuffer::from_fn(width, height, f))
    }

    const SIZE: u32 = 96;

    #[test]
    fn encode_decode() {
        let test_image_no_alpha = generate_color_wheel(SIZE, SIZE, false);
        let encoded = Encoder::from_image(&test_image_no_alpha).unwrap().encode_lossless();

        let decoded = Decoder::new(encoded.deref()).decode().unwrap().to_image().to_rgb8();
        assert_eq!(test_image_no_alpha.to_rgb8().deref(), decoded.deref());


        let test_image_alpha = generate_color_wheel(SIZE, SIZE, true);
        let encoded = Encoder::from_image(&test_image_alpha).unwrap().encode_lossless();

        let decoded = Decoder::new(encoded.deref()).decode().unwrap().to_image().to_rgba8();

        // To achieve better compression, the webp library changes the rgb values in transparent regions
        // This means we have to exclusively compare the opaque regions
        // See the note for WebPEncodeLossless* at https://developers.google.com/speed/webp/docs/api#simple_encoding_api
        fn compare(p1: &Rgba<u8>, p2: &Rgba<u8>) -> bool {
            // two pixels are equal if they are fully transparent
            if p1.channels()[3] == 0 && p2.channels()[3] == 0 {
                true
            } else { // or if they otherwise equal
                p1 == p2
            }
        }

        for (p1, p2) in test_image_alpha.to_rgba8().pixels().zip(decoded.pixels()) {
            assert!(compare(p1, p2))
        }
    }

    #[test]
    fn get_info() {
        let test_image_no_alpha = generate_color_wheel(SIZE, SIZE, false);
        let encoded = Encoder::from_image(&test_image_no_alpha).unwrap().encode_lossless();

        let features = BitstreamFeatures::new(encoded.deref()).unwrap();
        assert_eq!(features.width(), SIZE);
        assert_eq!(features.height(), SIZE);
        assert!(!features.has_alpha());
        assert!(!features.has_animation());


        let test_image_alpha = generate_color_wheel(SIZE, SIZE, true);
        let encoded = Encoder::from_image(&test_image_alpha).unwrap().encode_lossless();

        let features = BitstreamFeatures::new(encoded.deref()).unwrap();
        assert_eq!(features.width(), SIZE);
        assert_eq!(features.height(), SIZE);
        assert!(features.has_alpha());
        assert!(!features.has_animation());
    }
}