use libwebp_sys::*;

use crate::shared::PixelLayout;
use crate::{AnimFrame,Encoder};

#[cfg(feature = "img")]
use image::*;

pub struct AnimDecoder<'a> {
	data: &'a [u8],
}
impl<'a> AnimDecoder<'a> {
	pub fn new(data: &'a [u8]) -> Self {
		Self { data }
	}
	pub fn decode(&self) -> Result<DecodeAnimImage,String> {
		unsafe{self.decode_internal(true)}
	}
	unsafe fn decode_internal(&self,mut has_alpha:bool)->Result<DecodeAnimImage,String>{
		let mut dec_options: WebPAnimDecoderOptions =std::mem::zeroed();
		dec_options.color_mode=if has_alpha{WEBP_CSP_MODE::MODE_RGBA}else{WEBP_CSP_MODE::MODE_RGB};
		let ok=WebPAnimDecoderOptionsInitInternal(&mut dec_options,WebPGetDemuxABIVersion());
		if ok == 0{
			return Err(String::from("option init error"));
		}
		match dec_options.color_mode{
			WEBP_CSP_MODE::MODE_RGBA|WEBP_CSP_MODE::MODE_RGB=>{},
			_=>return Err(String::from("unsupport color mode"))
		}
		has_alpha=dec_options.color_mode==WEBP_CSP_MODE::MODE_RGBA;
		let webp_data=WebPData{
			bytes:self.data.as_ptr(),
			size:self.data.len()
		};
		let dec=WebPAnimDecoderNewInternal(&webp_data,&dec_options,WebPGetDemuxABIVersion());
		if dec.is_null(){
			return Err(String::from("null_decoder"));
		}
		let mut anim_info: WebPAnimInfo =std::mem::zeroed();
		let ok=WebPAnimDecoderGetInfo(dec,&mut anim_info);
		if ok == 0{
			return Err(String::from("null info"));
		}
		let width=anim_info.canvas_width;
		let height=anim_info.canvas_height;
		let mut list:Vec<DecodeAnimFrame>=vec![];
		while WebPAnimDecoderHasMoreFrames(dec) > 0 {
			let mut buf:*mut u8=std::ptr::null_mut();
			let mut timestamp:std::os::raw::c_int=0;
			let ok=WebPAnimDecoderGetNext(dec,&mut buf,&mut timestamp);
			if ok != 0{
				let len=(if has_alpha{4}else{3}*width*height) as usize;
				let mut img=Vec::with_capacity(len);
				img.set_len(len);
				buf.copy_to(img.as_mut_ptr(),len);
				let layout=if has_alpha{PixelLayout::Rgba}else{PixelLayout::Rgb};
				let frame=DecodeAnimFrame::new(img,width,height,layout,timestamp);
				list.push(frame.unwrap());
			}
		}
		WebPAnimDecoderReset(dec);
		//let demuxer:WebPDemuxer=WebPAnimDecoderGetDemuxer(dec);
		// ... (Do something using 'demuxer'; e.g. get EXIF/XMP/ICC data).
		WebPAnimDecoderDelete(dec);
		list.sort_by(|a,b| a.time_ms.cmp(&b.time_ms));
		let mut anim=DecodeAnimImage::from(list);
		anim.loop_count=anim_info.loop_count;
		anim.bg_color=anim_info.bgcolor;
		Ok(anim)
	}
}
pub struct DecodeAnimFrame{
	image: Vec<u8>,
	width:u32,
	height:u32,
	layout: PixelLayout,
	time_ms: i32,
}
#[cfg(feature = "img")]
impl Into<DynamicImage> for &DecodeAnimFrame{
	fn into(self) -> DynamicImage {
		if self.layout.is_alpha() {
			let image = ImageBuffer::from_raw(
				self.width(),
				self.height(),
				self.get_image().to_owned(),
			).expect("ImageBuffer couldn't be created");
			DynamicImage::ImageRgba8(image)
		} else {
			let image = ImageBuffer::from_raw(
				self.width(),
				self.height(),
				self.get_image().to_owned(),
			).expect("ImageBuffer couldn't be created");
			DynamicImage::ImageRgb8(image)
		}
	}
}
impl DecodeAnimFrame{
	pub fn new(image:Vec<u8>,width:u32,height:u32,layout:PixelLayout,time_ms:i32)->Option<DecodeAnimFrame>{
		let len=(if layout.is_alpha(){4}else{3})*width*height;
		if image.len()<len as usize{
			None
		}else{
			Some(DecodeAnimFrame{image,width,height,layout,time_ms})
		}
	}
	pub fn get_time_ms(&self)->i32{
		self.time_ms
	}
	pub fn get_image(&self)->&[u8]{
		&self.image
	}
	pub fn get_layout(&self)->PixelLayout{
		self.layout
	}
	pub fn width(&self) -> u32 {
		self.width
	}
	pub fn height(&self) -> u32 {
		self.height
	}
	pub fn to_encode_frame(&self,offset_time:i32)->AnimFrame{
		AnimFrame::new(&self.image,self.layout,self.time_ms-offset_time,None)
	}
}
impl <'a> From<&'a DecodeAnimFrame> for Encoder<'a>{
	fn from(f:&'a DecodeAnimFrame) -> Self {
		Encoder::new(&f.image,f.layout,f.width,f.height)
	}
}
pub struct DecodeAnimImage{
	frames:Vec<DecodeAnimFrame>,
	pub loop_count:u32,
	pub bg_color:u32,
}
impl From<Vec<DecodeAnimFrame>> for DecodeAnimImage{
	fn from(frames: Vec<DecodeAnimFrame>) -> Self {
		DecodeAnimImage{
			frames,
			loop_count:0,
			bg_color:0
		}
	}
}
impl DecodeAnimImage{
	pub fn get_frame(&self,index:usize)->Option<&DecodeAnimFrame>{
		self.frames.get(index)
	}
	pub fn len(&self)->usize{
		self.frames.len()
	}
	pub fn has_animation(&self)->bool{
		self.len()>1
	}
}
impl AsRef<Vec<DecodeAnimFrame>> for DecodeAnimImage{
	fn as_ref(&self) -> &Vec<DecodeAnimFrame> {
		&self.frames
	}
}
