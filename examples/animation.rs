use webp::WebPConfig;
use webp::AnimEncoder;
use webp::AnimFrame;
fn main(){
	let width=32u32;
	let height=32u32;
	fn dumy_image(width: u32,height: u32,color:[u8;4])->Vec<u8>{
		let mut pixels=Vec::with_capacity(width as usize*height as usize*4);
		for _ in 0..(width*height){
			pixels.push(color[0]);//red
			pixels.push(color[1]);//green
			pixels.push(color[2]);//blue
			pixels.push(color[3]);//alpha
		}
		pixels
	}
	let mut config = WebPConfig::new().unwrap();
	config.lossless = 1;
	config.alpha_compression = 0;
	config.quality = 75f32;
	let mut encoder=AnimEncoder::new(width as u32,height as u32,&config);
	encoder.set_bgcolor([255,0,0,255]);
	encoder.set_loop_count(3);
	let mut time_ms=1000;

	let v=dumy_image(width,height,[255,0,0,255]);
	encoder.add_frame(AnimFrame::from_rgba(&v,width,height,time_ms));
	time_ms+=750;

	let v=dumy_image(width,height,[0,255,0,255]);
	encoder.add_frame(AnimFrame::from_rgba(&v,width,height,time_ms));
	time_ms+=500;

	let v=dumy_image(width,height,[0,0,255,255]);
	encoder.add_frame(AnimFrame::from_rgba(&v,width,height,time_ms));
	time_ms+=250;

	let v=dumy_image(width,height,[0,0,0,0]);
	encoder.add_frame(AnimFrame::from_rgba(&v,width,height,time_ms));

	let webp=encoder.encode();
	let output_path = std::path::Path::new("assets").join("dumy_anim").with_extension("webp");
	std::fs::write(&output_path, &*webp).unwrap();
}
