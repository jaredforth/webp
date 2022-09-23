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
        Self { image, layout, width, height }
    }

    #[cfg(feature = "img")]
    /// Creates a new encoder from the given image.
    pub fn from_image(image: &'a DynamicImage) -> Result<Self, &str> {
        match image {
            DynamicImage::ImageLuma8(_) => { Err("Unimplemented") }
            DynamicImage::ImageLumaA8(_) => { Err("Unimplemented") }
            DynamicImage::ImageRgb8(image) => {
                Ok(Self::from_rgb(image.as_ref(), image.width(), image.height()))
            }
            DynamicImage::ImageRgba8(image) => {
                Ok(Self::from_rgba(image.as_ref(), image.width(), image.height()))
            }
            _ => { Err("Unimplemented") }
        }
    }

    /// Creates a new encoder from the given image data in the RGB pixel layout.
    pub fn from_rgb(image: &'a [u8], width: u32, height: u32) -> Self {
        Self { image, layout: PixelLayout::Rgb, width, height }
    }

    /// Creates a new encoder from the given image data in the RGBA pixel layout.
    pub fn from_rgba(image: &'a [u8], width: u32, height: u32) -> Self {
        Self { image, layout: PixelLayout::Rgba, width, height }
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

	pub fn encode_simple(&self, lossless:bool, quality: f32)->Result<WebPMemory,WebPEncodingError>{
		let mut config = WebPConfig::new().unwrap();
		config.lossless = if lossless { 1 } else { 0 };
		config.alpha_compression = if lossless { 0 } else { 1 };
		config.quality = quality;
		self.encode_advanced(&config)
	}

	pub fn encode_advanced(&self,config:&WebPConfig)->Result<WebPMemory,WebPEncodingError>{
		unsafe{
			let picture=new_picture(self.image,self.layout,self.width,self.height);
			encode(picture,config)
		}
	}
}

pub struct AnimFrame<'a> {
    image: &'a [u8],
    layout: PixelLayout,
	delay_ms: i32,
	config:Option<&'a WebPConfig>,
}
impl<'a> AnimFrame<'a> {
	pub fn new(image: &'a [u8], layout: PixelLayout, delay_ms: i32,config: Option<&'a WebPConfig>) -> Self {
		Self {image,layout, delay_ms ,config}
	}
    #[cfg(feature = "img")]
    pub fn from_image(image: &'a DynamicImage, delay_ms: i32) -> Result<Self, &str> {
        match image {
            DynamicImage::ImageLuma8(_) => { Err("Unimplemented") }
            DynamicImage::ImageLumaA8(_) => { Err("Unimplemented") }
            DynamicImage::ImageRgb8(image) => {
                Ok(Self::from_rgb(image.as_ref(), delay_ms))
            }
            DynamicImage::ImageRgba8(image) => {
                Ok(Self::from_rgba(image.as_ref(), delay_ms))
            }
            _ => { Err("Unimplemented") }
        }
    }
	/// Creates a new encoder from the given image data in the RGB pixel layout.
	pub fn from_rgb(image: &'a [u8], delay_ms: i32) -> Self {
		Self::new(image,PixelLayout::Rgb,delay_ms,None)
	}
	/// Creates a new encoder from the given image data in the RGBA pixel layout.
	pub fn from_rgba(image: &'a [u8], delay_ms: i32) -> Self {
		Self::new(image,PixelLayout::Rgba,delay_ms,None)
	}
}
pub struct AnimEncoder<'a> {
	frames:Vec<AnimFrame<'a>>,
	width: u32,
	height: u32,
	config:&'a WebPConfig,
	muxparams:WebPMuxAnimParams,
}
impl<'a> AnimEncoder<'a> {
	pub fn new(width: u32,height: u32,config:&'a WebPConfig)->Self{
		Self{frames:vec![],width,height,config,muxparams:WebPMuxAnimParams{bgcolor:0,loop_count:0}}
	}
	pub fn set_bgcolor(&mut self,rgba:[u8;4]){
		let bgcolor=(u32::from(rgba[3])<<24)
		+(u32::from(rgba[2])<<16)
		+(u32::from(rgba[1])<<8)
		+(u32::from(rgba[0]));
		self.muxparams.bgcolor=bgcolor;
	}
	pub fn set_loop_count(&mut self,loop_count:i32){
		self.muxparams.loop_count=loop_count;
	}
	pub fn add_frame(&mut self,frame:AnimFrame<'a>){
		self.frames.push(frame);
	}
	pub fn encode(&self)->WebPMemory{
		unsafe{anim_encode(&self)}
	}
}
unsafe fn anim_encode(all_frame:&AnimEncoder)->WebPMemory{
	let width=all_frame.width;
	let height=all_frame.height;
	let mut uninit = std::mem::MaybeUninit::<WebPAnimEncoderOptions>::uninit();

	let mux_abi_version=WebPGetMuxABIVersion();
	WebPAnimEncoderOptionsInitInternal(uninit.as_mut_ptr(),mux_abi_version);
	let encoder=WebPAnimEncoderNewInternal(width as i32,height as i32,uninit.as_ptr(),mux_abi_version);
	let mut offset:std::os::raw::c_int=0;
	for frame in all_frame.frames.iter(){
		let mut pic=new_picture(frame.image,frame.layout,width,height);
		let config=frame.config.unwrap_or(all_frame.config);
		WebPAnimEncoderAdd(encoder,&mut pic,offset,config);
		offset+=frame.delay_ms;
	}
	WebPAnimEncoderAdd(encoder, std::ptr::null_mut(), offset, std::ptr::null());

	let mut webp_data=std::mem::MaybeUninit::<WebPData>::uninit();
	WebPAnimEncoderAssemble(encoder,webp_data.as_mut_ptr());
	let mux=WebPMuxCreateInternal(webp_data.as_ptr(), 1,mux_abi_version);
	WebPMuxSetAnimationParams(mux, &all_frame.muxparams);
	WebPMuxAssemble(mux, webp_data.as_mut_ptr());
	let raw_data:WebPData=webp_data.assume_init();
	WebPMemory(raw_data.bytes as *mut u8,raw_data.size)
}

unsafe fn new_picture(image: &[u8], layout: PixelLayout, width: u32, height: u32)->WebPPicture{
	let mut picture = WebPPicture::new().unwrap();
	picture.use_argb = 1;
	picture.width = width as i32;
	picture.height = height as i32;
	match layout{
		PixelLayout::Rgba=>{
			WebPPictureImportRGBA(&mut picture, image.as_ptr(),width as i32 * 4);
		},
		PixelLayout::Rgb=>{
			WebPPictureImportRGB(&mut picture, image.as_ptr(),width as i32 * 3);
		}
	}
	picture
}
unsafe fn encode(mut picture:WebPPicture,config:&WebPConfig) -> Result<WebPMemory,WebPEncodingError>{
	if WebPValidateConfig(config) == 0 {
		return Err(WebPEncodingError::VP8_ENC_ERROR_INVALID_CONFIGURATION);
	}
	let mut ww=std::mem::MaybeUninit::uninit();
	WebPMemoryWriterInit(ww.as_mut_ptr());
	picture.writer = Some(WebPMemoryWrite);
	picture.custom_ptr = ww.as_mut_ptr() as *mut std::ffi::c_void;
	let status=libwebp_sys::WebPEncode(config,&mut picture);
	let ww=ww.assume_init();
	let mem=WebPMemory(ww.mem , ww.size as usize);
	if status != VP8StatusCode::VP8_STATUS_OK as i32{
		Ok(mem)
	}else{
		Err(picture.error_code)
	}
}