use webp::AnimEncoder;
use webp::AnimFrame;
use webp::WebPConfig;
fn main() {
    let width = 512u32;
    let height = 512u32;

    fn dumy_image(width: u32, height: u32, frame: u32, total_frames: u32) -> Vec<u8> {
        let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
        for x in 0..width {
            for y in 0..height {
                let normalized_frame = frame as f32 / total_frames as f32;
                let normalized_x = x as f32 / width as f32;
                let normalized_y = y as f32 / height as f32;

                let r = ((normalized_frame + normalized_x + normalized_y) % 1.0 * 255.0) as u8;
                let g =
                    ((normalized_frame + normalized_x + normalized_y + 0.33) % 1.0 * 255.0) as u8;
                let b =
                    ((normalized_frame + normalized_x + normalized_y + 0.67) % 1.0 * 255.0) as u8;

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(255); // alpha channel, fully opaque
            }
        }
        pixels
    }

    let mut config = WebPConfig::new().unwrap();
    config.lossless = 1;
    config.alpha_compression = 0;
    config.quality = 100f32;
    let mut encoder = AnimEncoder::new(width as u32, height as u32, &config);
    encoder.set_bgcolor([255, 0, 0, 255]);
    encoder.set_loop_count(0);
    let mut time_ms = 0;

    for i in 0..120 {
        let image = dumy_image(width, height, i, 120);
        encoder
            .add_frame(AnimFrame::from_rgba(&image, width, height, time_ms))
            .unwrap();
        time_ms += 17;
    }

    let webp = encoder.encode();
    let output_path = std::path::Path::new("assets")
        .join("dumy_anim")
        .with_extension("webp");
    std::fs::write(&output_path, &*webp).unwrap();
}
