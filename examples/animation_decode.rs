/*
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org/>
*/

use webp::Encoder;
use webp::AnimDecoder;
//animation webp to webp demo
fn main(){
	let src="dumy_anim";
	let input = std::path::Path::new("assets").join(src).with_extension("webp");
	let webp=std::fs::read(input).unwrap();
	match AnimDecoder::new(&webp).decode(){
		Ok(frames)=>{
			let mut file_number=0;
			println!("has_animation {}",frames.has_animation());
			println!("loop_count {}",frames.loop_count);
			println!("bg_color {}",frames.bg_color);
			let mut last_ms=0;
			for f in frames.into_iter(){
				let delay_ms=f.get_time_ms()-last_ms;
				println!("{}x{} {:?} time{}ms delay{}ms",f.width(),f.height(),f.get_layout(),f.get_time_ms(),delay_ms);
				last_ms+=delay_ms;
				let webp=Encoder::from(&f).encode_simple(true,100f32);
				let output = std::path::Path::new("assets").join(format!("{}{}",src,file_number)).with_extension("webp");
				file_number+=1;
				std::fs::write(&output, &*webp.unwrap()).unwrap();
			}
		},
		Err(mes)=>{
			println!("{}",mes);
		}
	}
}
