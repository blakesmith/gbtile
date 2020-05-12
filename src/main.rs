use clap::{App, Arg};
use log;
use log::Level;
use png::Decoder;
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io;

const GB_MAX_COLOR_COUNT: usize = 4;

#[derive(Debug)]
enum OutputType {
    GBDK,
}

#[derive(Debug)]
struct CommandArguments {
    pub input: String,
    pub output: String,
    pub output_type: OutputType,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash, Ord, Eq)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    fn round(&self) -> RGB {
        RGB {
            r: (self.r / 32) * 32,
            g: (self.g / 32) * 32,
            b: (self.b / 32) * 32,
        }
    }
}

struct DecodedImage {
    image_data: Vec<RGB>,
    color_numbers: HashMap<RGB, u8>,
}

#[derive(Debug)]
enum ImageReadError {
    Png(png::DecodingError),
    Io(io::Error),
    UnsupportedColorType(png::ColorType),
    TooManyColors,
}

impl From<io::Error> for ImageReadError {
    fn from(err: io::Error) -> Self {
        ImageReadError::Io(err)
    }
}

impl From<png::DecodingError> for ImageReadError {
    fn from(err: png::DecodingError) -> Self {
        ImageReadError::Png(err)
    }
}

fn rgbs_to_color_number(unique_colors: &BTreeSet<RGB>) -> HashMap<RGB, u8> {
    let mut color_numbers = HashMap::new();
    for (i, rgb) in unique_colors.iter().enumerate() {
        color_numbers.insert(*rgb, i as u8);
    }
    color_numbers
}

fn read_image_data(info: png::OutputInfo, image_buf: Vec<u8>) -> Result<Vec<RGB>, ImageReadError> {
    log::debug!("PNG info: {:?}", info);
    let mut image_data = Vec::new();
    match info.color_type {
        png::ColorType::RGB => {
            for (i, color) in image_buf.chunks(3).enumerate() {
                let rgb = RGB {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                };
                image_data.push(rgb.round());
            }
        }
        png::ColorType::RGBA => {
            for color in image_buf.chunks(4) {
                let rgb = RGB {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                };
                image_data.push(rgb.round());
            }
        }
        png::ColorType::Grayscale => {
            for color in image_buf {
                let rgb = RGB {
                    r: color,
                    g: color,
                    b: color,
                };
                image_data.push(rgb.round());
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for color in image_buf.chunks(2) {
                let rgb = RGB {
                    r: color[0],
                    g: color[0],
                    b: color[0],
                };
                image_data.push(rgb.round());
            }
        }
        color_type => {
            return Err(ImageReadError::UnsupportedColorType(color_type));
        }
    }

    Ok(image_data)
}

fn decode_image(image_input: &str) -> Result<DecodedImage, ImageReadError> {
    let file = File::open(image_input)?;
    let mut unique_colors = BTreeSet::new();
    let decoder = Decoder::new(file);
    let (info, mut png_reader) = decoder.read_info()?;

    let mut image_buf = vec![0; info.buffer_size()];
    png_reader.next_frame(&mut image_buf)?;
    let image_data = read_image_data(info, image_buf)?;

    log::debug!("Image data size is: {}", image_data.len());

    for (i, color) in image_data.iter().enumerate() {
        unique_colors.insert(*color);
        if unique_colors.len() > GB_MAX_COLOR_COUNT {
            log::debug!("Unique colors are: {:?}, stopped at: {}", unique_colors, i,);
            return Err(ImageReadError::TooManyColors);
        }
    }
    let color_numbers = rgbs_to_color_number(&unique_colors);
    log::debug!("Color numbers are: {:?}", color_numbers);

    let decoded = DecodedImage {
        image_data,
        color_numbers,
    };
    Ok(decoded)
}

fn main() {
    let matches = App::new("Gameboy Tile Generator")
        .version("0.1")
        .author("Blake Smith <blakesmith0@gmail.com>")
        .about("Generate GBDK Game Boy tiles from PNG images")
        .arg(
            Arg::with_name("debug")
                .help("Enable debug logging")
                .short("d"),
        )
        .arg(
            Arg::with_name("input")
                .help("The PNG image to generate tiles from")
                .short("i")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .help("The output file to generate")
                .short("o")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output-type")
                .help("The output type. Defaults to 'gbdk'")
                .takes_value(true)
                .short("t"),
        )
        .get_matches();

    if matches.is_present("debug") {
        simple_logger::init_with_level(Level::Debug).unwrap();
    } else {
        simple_logger::init_with_level(Level::Info).unwrap();
    }
    let output_type = match matches.value_of("output-type") {
        Some("gbdk") => OutputType::GBDK,
        _ => OutputType::GBDK,
    };

    let args = CommandArguments {
        input: matches.value_of("input").unwrap().to_string(),
        output: matches.value_of("output").unwrap().to_string(),
        output_type: output_type,
    };

    let decoded_image = decode_image(&args.input).expect("Could not decode image");

    println!("Arguments are: {:?}", args);
}
