use clap::Parser;
use image::io::Reader as ImageReader;
use palette::rgb::Rgb;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to image.
    #[clap(short, long, parse(from_os_str), value_hint = clap::ValueHint::FilePath)]
    image_path: PathBuf,
    /// Path to palette.
    #[clap(short, long, parse(from_os_str), value_hint = clap::ValueHint::FilePath)]
    palette_path: PathBuf,
}

fn main() {
    let args = Args::parse();
    println!("args: {:?}", args);
    let image = ImageReader::open(args.image_path).unwrap();
}
// let color_buffer: &mut [Srgb<u8>] = Pixel::from_raw_slice_mut(&mut image_buffer);
