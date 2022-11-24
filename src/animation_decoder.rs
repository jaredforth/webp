use libwebp_sys::*;

use crate::shared::PixelLayout;
use crate::AnimFrame;

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
				let frame=DecodeAnimFrame{img,width,height,layout,timestamp};
				list.push(frame);
			}
		}
		WebPAnimDecoderReset(dec);
		//let demuxer:WebPDemuxer=WebPAnimDecoderGetDemuxer(dec);
		// ... (Do something using 'demuxer'; e.g. get EXIF/XMP/ICC data).
		WebPAnimDecoderDelete(dec);
		let mut anim=DecodeAnimImage::from(list);
		anim.loop_count=anim_info.loop_count;
		anim.bg_color=anim_info.bgcolor;
		Ok(anim)
	}
}
struct DecodeAnimFrame{
	img: Vec<u8>,
	width:u32,
	height:u32,
	layout: PixelLayout,
	timestamp:i32,
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
	#[inline]
	pub fn get_frame(&self,index:usize)->Option<AnimFrame>{
		let f=self.frames.get(index)?;
		Some(AnimFrame::new(&f.img,f.layout,f.width,f.height,f.timestamp,None))
	}
	#[inline]
	pub fn get_frames(&self,index:core::ops::Range<usize>)->Option<Vec<AnimFrame>>{
		let dec_frames=self.frames.get(index)?;
		let mut frames=Vec::with_capacity(dec_frames.len());
		for f in dec_frames{
			frames.push(AnimFrame::new(&f.img,f.layout,f.width,f.height,f.timestamp,None));
		}
		Some(frames)
	}
	pub fn len(&self)->usize{
		self.frames.len()
	}
	pub fn has_animation(&self)->bool{
		self.len()>1
	}
	pub fn sort_by_time_stamp(&mut self){
		self.frames.sort_by(|a,b| a.timestamp.cmp(&b.timestamp));
	}
}
impl <'a> IntoIterator for &'a DecodeAnimImage{
	type Item = AnimFrame<'a>;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		let fs=self.get_frames(0..self.frames.len());
		if let Some(v)=fs{
			v.into_iter()
		}else{
			vec![].into_iter()
		}
	}
}
