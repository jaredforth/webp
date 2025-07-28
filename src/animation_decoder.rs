#![allow(clippy::uninit_vec)]
use libwebp_sys::*;

use crate::AnimFrame;
use crate::shared::PixelLayout;

pub struct AnimDecoder<'a> {
    data: &'a [u8],
}
impl<'a> AnimDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
    pub fn decode(&self) -> Result<DecodeAnimImage, String> {
        unsafe { self.decode_internal(true) }
    }
    unsafe fn decode_internal(&self, mut has_alpha: bool) -> Result<DecodeAnimImage, String> {
        unsafe {
            let mut dec_options: WebPAnimDecoderOptions = std::mem::zeroed();
            dec_options.color_mode = if has_alpha {
                WEBP_CSP_MODE::MODE_RGBA
            } else {
                WEBP_CSP_MODE::MODE_RGB
            };
            let ok = WebPAnimDecoderOptionsInitInternal(&mut dec_options, WebPGetDemuxABIVersion());
            if ok == 0 {
                return Err(String::from("option init error"));
            }
            match dec_options.color_mode {
                WEBP_CSP_MODE::MODE_RGBA | WEBP_CSP_MODE::MODE_RGB => {}
                _ => return Err(String::from("unsupport color mode")),
            }
            has_alpha = dec_options.color_mode == WEBP_CSP_MODE::MODE_RGBA;
            let webp_data = WebPData {
                bytes: self.data.as_ptr(),
                size: self.data.len(),
            };
            let dec =
                WebPAnimDecoderNewInternal(&webp_data, &dec_options, WebPGetDemuxABIVersion());
            if dec.is_null() {
                return Err(String::from("null_decoder"));
            }
            let mut anim_info: WebPAnimInfo = std::mem::zeroed();
            let ok = WebPAnimDecoderGetInfo(dec, &mut anim_info);
            if ok == 0 {
                return Err(String::from("null info"));
            }
            let width = anim_info.canvas_width;
            let height = anim_info.canvas_height;
            let mut list: Vec<DecodeAnimFrame> = vec![];
            while WebPAnimDecoderHasMoreFrames(dec) > 0 {
                let mut buf: *mut u8 = std::ptr::null_mut();
                let mut timestamp: std::os::raw::c_int = 0;
                let ok = WebPAnimDecoderGetNext(dec, &mut buf, &mut timestamp);
                if ok != 0 {
                    let len = (if has_alpha { 4 } else { 3 } * width * height) as usize;
                    let mut img = Vec::with_capacity(len);
                    img.set_len(len);
                    buf.copy_to(img.as_mut_ptr(), len);
                    let layout = if has_alpha {
                        PixelLayout::Rgba
                    } else {
                        PixelLayout::Rgb
                    };
                    let frame = DecodeAnimFrame {
                        img,
                        width,
                        height,
                        layout,
                        timestamp,
                    };
                    list.push(frame);
                }
            }
            WebPAnimDecoderReset(dec);
            //let demuxer:WebPDemuxer=WebPAnimDecoderGetDemuxer(dec);
            // ... (Do something using 'demuxer'; e.g. get EXIF/XMP/ICC data).
            WebPAnimDecoderDelete(dec);
            let mut anim = DecodeAnimImage::from(list);
            anim.loop_count = anim_info.loop_count;
            anim.bg_color = anim_info.bgcolor;
            Ok(anim)
        }
    }
}
struct DecodeAnimFrame {
    img: Vec<u8>,
    width: u32,
    height: u32,
    layout: PixelLayout,
    timestamp: i32,
}
pub struct DecodeAnimImage {
    frames: Vec<DecodeAnimFrame>,
    pub loop_count: u32,
    pub bg_color: u32,
}
impl From<Vec<DecodeAnimFrame>> for DecodeAnimImage {
    fn from(frames: Vec<DecodeAnimFrame>) -> Self {
        DecodeAnimImage {
            frames,
            loop_count: 0,
            bg_color: 0,
        }
    }
}
impl DecodeAnimImage {
    #[inline]
    pub fn get_frame(&self, index: usize) -> Option<AnimFrame> {
        let f = self.frames.get(index)?;
        Some(AnimFrame::new(
            &f.img,
            f.layout,
            f.width,
            f.height,
            f.timestamp,
            None,
        ))
    }
    #[inline]
    pub fn get_frames(&self, index: core::ops::Range<usize>) -> Option<Vec<AnimFrame>> {
        let dec_frames = self.frames.get(index)?;
        let mut frames = Vec::with_capacity(dec_frames.len());
        for f in dec_frames {
            frames.push(AnimFrame::new(
                &f.img,
                f.layout,
                f.width,
                f.height,
                f.timestamp,
                None,
            ));
        }
        Some(frames)
    }
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn has_animation(&self) -> bool {
        self.len() > 1
    }
    pub fn sort_by_time_stamp(&mut self) {
        self.frames.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }
}
impl<'a> IntoIterator for &'a DecodeAnimImage {
    type Item = AnimFrame<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let fs = self.get_frames(0..self.frames.len());
        if let Some(v) = fs {
            v.into_iter()
        } else {
            vec![].into_iter()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_webp_animation() -> Vec<u8> {
        vec![
            0x52, 0x49, 0x46, 0x46, 0x84, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50,
            0x38, 0x58, 0x0a, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x41, 0x4e, 0x49, 0x4d, 0x06, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
            0x00, 0x00, 0x41, 0x4e, 0x4d, 0x46, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x02, 0x56, 0x50,
            0x38, 0x4c, 0x0f, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0x00, 0x00, 0x07, 0x10, 0xfd,
            0x8f, 0xfe, 0x07, 0x22, 0xa2, 0xff, 0x01, 0x00, 0x41, 0x4e, 0x4d, 0x46, 0x28, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x64, 0x00, 0x00, 0x00, 0x56, 0x50, 0x38, 0x4c, 0x0f, 0x00, 0x00, 0x00, 0x2f, 0x00,
            0x00, 0x00, 0x00, 0x07, 0x10, 0xd1, 0xff, 0xfe, 0x07, 0x22, 0xa2, 0xff, 0x01, 0x00,
        ]
    }

    #[test]
    fn test_decoder_creation() {
        let data = minimal_webp_animation();
        let decoder = AnimDecoder::new(&data);
        assert_eq!(decoder.data, &data[..]);
    }

    #[test]
    fn test_decode_success_and_metadata() {
        let data = minimal_webp_animation();
        let decoder = AnimDecoder::new(&data);
        let result = decoder.decode();
        assert!(result.is_ok(), "Decoding should succeed for valid data");
        let anim = result.unwrap();
        assert!(!anim.is_empty(), "Animation should have at least one frame");
        let _ = anim.loop_count;
        let _ = anim.bg_color;
    }

    #[test]
    fn test_get_frame_and_get_frames() {
        let data = minimal_webp_animation();
        let decoder = AnimDecoder::new(&data);
        let anim = decoder.decode().unwrap();
        let frame = anim.get_frame(0);
        assert!(frame.is_some(), "Should retrieve first frame");
        let frames = anim.get_frames(0..1);
        assert!(frames.is_some(), "Should retrieve frame range");
        assert_eq!(frames.unwrap().len(), 1);
    }

    #[test]
    fn test_has_animation_and_len() {
        let data = minimal_webp_animation();
        let decoder = AnimDecoder::new(&data);
        let anim = decoder.decode().unwrap();
        assert_eq!(anim.has_animation(), anim.len() > 1);
    }

    #[test]
    fn test_sort_by_time_stamp() {
        let data = minimal_webp_animation();
        let decoder = AnimDecoder::new(&data);
        let mut anim = decoder.decode().unwrap();
        anim.frames.reverse();
        anim.sort_by_time_stamp();
        let timestamps: Vec<_> = anim.frames.iter().map(|f| f.timestamp).collect();
        assert!(timestamps.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn test_iteration() {
        let data = minimal_webp_animation();
        let decoder = AnimDecoder::new(&data);
        let anim = decoder.decode().unwrap();
        let count = anim.into_iter().count();
        assert_eq!(count, anim.len());
    }

    #[test]
    fn test_decode_failure_on_invalid_data() {
        let data = vec![0u8; 10];
        let decoder = AnimDecoder::new(&data);
        let result = decoder.decode();
        assert!(result.is_err(), "Decoding should fail for invalid data");
    }
}
