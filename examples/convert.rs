use image::*;
use std::path::Path;
use webp::*;

fn main() {
    // Using `image` crate, open the included .jpg file
    let img = image::open("assets/lake.jpg").unwrap();
    let (w, h) = img.dimensions();
    // Optionally, resize the existing photo and convert back into DynamicImage
    let size_factor = 1.0;
    let img: DynamicImage = image::DynamicImage::ImageRgba8(imageops::resize(
        &img,
        (w as f64 * size_factor) as u32,
        (h as f64 * size_factor) as u32,
        imageops::FilterType::Triangle,
    ));

    // Create the WebP encoder for the above image
    let encoder: Encoder = Encoder::from_image(&img).unwrap();
    // Encode the image at a specified quality 0-100
    let webp: WebPMemory = encoder.encode(90f32);
    // Define and write the WebP-encoded file to a given path
    let output_path = Path::new("assets").join("lake").with_extension("webp");
    std::fs::write(&output_path, &*webp).unwrap();
}
