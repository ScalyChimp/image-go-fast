use gumdrop::Options;
use image::{imageops, io::Reader as ImageReader, DynamicImage, Pixel, Rgb, RgbImage};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    env::current_exe,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Options, Debug)]
struct Args {
    #[options(free, parse(from_str = "PathBuf::from"), help = "Path to input file")]
    input: PathBuf,

    #[options(
        free,
        parse(from_str = "PathBuf::from"),
        help = "Where to save output to. Image will be saved according to file extension"
    )]
    output: PathBuf,

    #[options(help = "Print help message")]
    help: bool,

    #[options(help = "Disables multithreading")]
    no_multithreading: bool,

    #[options(help = "Optional path to palette file")]
    palette_path: Option<PathBuf>,

    #[options(help = "Blur amount")]
    blur: Option<f32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse_args_default_or_exit();

    println!("Loading palette.");
    let palette = if let Some(path) = args.palette_path {
        deserialize_palette_file(path)?
    } else {
        let mut dir = current_exe().unwrap();
        dir.pop();
        dir.pop();
        dir.pop();
        let dir = dir.join("palettes/gruvbox.txt");
        println!("dir: {:?}", dir);
        deserialize_palette_file(dir.into())?
    };

    println!("opening image");
    let image = ImageReader::open(args.input)?.decode()?;
    println!("Generating image.");
    let mut image = match args.no_multithreading {
        true => generate_image(image, palette)?,
        false => generate_image_multithreaded(image, palette)?,
    };

    if let Some(blur) = args.blur {
        println!("Blurring image.");
        image = imageops::blur(&image, blur);
    }

    image.save(args.output)?;
    println!("Image saved.");
    Ok(())
}

fn generate_image(image: DynamicImage, palette: Vec<Rgb<u8>>) -> Result<RgbImage, Box<dyn Error>> {
    let mut buffer = image.into_rgb8();
    for pixel in buffer.pixels_mut() {
        *pixel = *palette
            .iter()
            .min_by_key(|pix| color_dif(pixel, pix))
            .unwrap();
    }
    Ok(buffer)
}

fn generate_image_multithreaded(
    image: DynamicImage,
    palette: Vec<Rgb<u8>>,
) -> Result<RgbImage, Box<dyn Error>> {
    let buffer: Vec<Rgb<u8>> = image.clone().into_rgb8().pixels().cloned().collect();

    let vec: Vec<u8> = buffer
        .par_iter()
        .flat_map(|pixel| {
            palette
                .iter()
                .min_by_key(|pix| color_dif(pixel, pix))
                .unwrap()
                .0
                .par_iter()
        })
        .cloned()
        .collect();

    Ok(RgbImage::from_vec(image.width(), image.height(), vec).unwrap())
}

fn color_dif(col1: &Rgb<u8>, col2: &Rgb<u8>) -> i32 {
    let chan1 = col1.channels();
    let chan2 = col2.channels();
    let vec: Vec<i32> = vec![
        i32::abs(chan1[0] as i32 - chan2[0] as i32),
        i32::abs(chan1[1] as i32 - chan2[1] as i32),
        i32::abs(chan1[2] as i32 - chan2[2] as i32),
    ];
    vec.into_iter().sum()
}

fn parse_hex_color(hex_color: &str) -> Result<Rgb<u8>, Box<dyn Error>> {
    let hex_color: &str = &hex_color[1..hex_color.len()]; // For the `#` at the start of hex strings.
    let array: [u8; 3] = hex::decode(hex_color)?.try_into().unwrap();
    Ok(*Rgb::<u8>::from_slice(&array))
}

fn deserialize_palette_file(path: PathBuf) -> Result<Vec<Rgb<u8>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let colors = BufReader::new(file)
        .lines()
        .map(|line| parse_hex_color(line.unwrap().as_str()).expect("broken palette file"))
        .collect();
    Ok(colors)
}
