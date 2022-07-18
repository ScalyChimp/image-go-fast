use clap::Parser;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use image::Pixel;
use image::Rgb;
use image::RgbImage;
use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to image.
    #[clap(short, long, parse(from_os_str), value_hint = clap::ValueHint::FilePath)]
    image_path: PathBuf,
    /// where to save new image
    #[clap(short, long, parse(from_os_str), value_hint = clap::ValueHint::FilePath)]
    new_image_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    println!("args: {:?}", args);
    println!("opening image");
    let image = ImageReader::open(&args.image_path)?.decode()?;
    println!("getting palette");
    let gruvbox = deserialize_palette_file(
        "/home/scalychimp/coding/themeifier/palettes/gruvbox-white.txt".into(),
    )?;
    let new_image = generate_image(image, gruvbox)?;
    println!("image generated");
    new_image.save(args.new_image_path)?;
    println!("image saved");
    Ok(())
}

fn generate_image(
    image: DynamicImage,
    mut palette: Vec<Rgb<u8>>,
) -> Result<RgbImage, Box<dyn Error>> {
    let mut buffer = image.into_rgb8();
    for pixel in buffer.pixels_mut() {
        palette.sort_by_key(|pix| color_dif(pixel, pix)); // Rust iterators my beloved.
        *pixel = palette[0]
    }
    Ok(buffer)
}

// fn generate_image_average(
//     image: DynamicImage,
//     palette: Vec<Rgb<u8>>,
// ) -> Result<RgbImage, Box<dyn Error>> {
//     Ok(())
// }

fn parse_hex_color(hex_color: &str) -> Result<Rgb<u8>, Box<dyn Error>> {
    let hex_color: &str = &hex_color[1..hex_color.len()]; // For the `#` at the start of hex strings.
    let array: [u8; 3] = hex::decode(hex_color)?.try_into().unwrap();
    Ok(*Rgb::<u8>::from_slice(&array))
}

fn color_dif(col1: &Rgb<u8>, col2: &Rgb<u8>) -> i32 {
    let chan1 = col1.channels();
    let chan2 = col2.channels();
    let vec: Vec<i32> = vec![
        chan1[0] as i32 - chan2[0] as i32,
        chan1[1] as i32 - chan2[1] as i32,
        chan1[2] as i32 - chan2[2] as i32,
    ];
    i32::abs(vec.into_iter().sum())
}

fn deserialize_palette_file(path: String) -> Result<Vec<Rgb<u8>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let colors = BufReader::new(file)
        .lines()
        .map(|line| parse_hex_color(line.unwrap().as_str()).expect("broken palette file"))
        .collect();
    Ok(colors)
}
