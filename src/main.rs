use clap::{App, Arg};
use log;
use log::Level;
use png::Decoder;
use std::collections::{BTreeSet, HashMap};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::path::Path;

const GB_MAX_COLOR_COUNT: usize = 4;

#[derive(Copy, Clone, Debug)]
enum OutputType {
    Gbdk,
    Rgbds,
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
    input_filename: String,
    info: png::OutputInfo,
    image_data: Vec<RGB>,
    color_numbers: HashMap<RGB, u8>,
}

struct EncodedTile {
    input_filename: String,
    tile_data: Vec<u8>,
}

impl DecodedImage {
    fn lookup_color(&self, pixel: &RGB) -> u8 {
        *self.color_numbers.get(&pixel).unwrap()
    }
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

fn map_2bit(rgb: &RGB) -> u8 {
    let sum: u16 = rgb.r as u16 + rgb.g as u16 + rgb.b as u16;
    if sum <= 191 {
        3
    } else if sum > 191 && sum <= 382 {
        2
    } else if sum > 382 && sum <= 573 {
        1
    } else {
        0
    }
}

fn rgbs_to_color_number(unique_colors: &BTreeSet<RGB>) -> HashMap<RGB, u8> {
    let mut color_numbers = HashMap::new();
    for rgb in unique_colors.iter() {
        color_numbers.insert(*rgb, map_2bit(rgb));
    }
    color_numbers
}

fn read_image_data(info: &png::OutputInfo, image_buf: Vec<u8>) -> Result<Vec<RGB>, ImageReadError> {
    log::debug!("PNG info: {:?}", info);
    let mut image_data = Vec::new();
    match info.color_type {
        png::ColorType::RGB => {
            for color in image_buf.chunks(3) {
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
    let image_data = read_image_data(&info, image_buf)?;

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
        input_filename: image_input.to_string(),
        image_data,
        info,
        color_numbers,
    };
    Ok(decoded)
}

const PIXELS_PER_LINE: u8 = 8;

fn encode_tile(decoded_image: DecodedImage) -> EncodedTile {
    let rows = decoded_image.info.height / 8;
    let columns = decoded_image.info.width / 8;
    log::info!(
        "File: {}, Tile rows: {}, columns: {}, unique colors: {}",
        decoded_image.input_filename,
        rows,
        columns,
        decoded_image.color_numbers.len()
    );
    let mut tile_data = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            for tile_row in 0..8 {
                let mut low_byte = 0;
                let mut high_byte = 0;
                for tile_column in 0..8 {
                    let pixel_index = (column * 8 + tile_column)
                        + ((decoded_image.info.width * tile_row)
                            + (row * 8 * decoded_image.info.width));
                    let pixel = decoded_image.image_data[pixel_index as usize];
                    let color = decoded_image.lookup_color(&pixel);
                    low_byte |= (color & 0x01) << (PIXELS_PER_LINE - tile_column as u8 - 1);
                    high_byte |= ((color >> 1) & 0x01) << (PIXELS_PER_LINE - tile_column as u8 - 1);
                }
                tile_data.push(low_byte);
                tile_data.push(high_byte);
            }
        }
    }

    let input_filename = decoded_image.input_filename.clone();

    EncodedTile {
        input_filename,
        tile_data,
    }
}

fn write_tile_gbdk(variable_name: &str, encoded_tile: &EncodedTile) -> String {
    let preamble = format!("unsigned char {}[] = {{", variable_name);
    let mut body = Vec::new();
    for line in encoded_tile.tile_data.chunks(16) {
        let mut formatted_bytes = Vec::new();
        for byte in line {
            formatted_bytes.push(format!("{:#04X}", byte));
        }
        body.push(format!("    {}", formatted_bytes.join(",")));
    }

    format!("{}\n{}\n}};\n", preamble, body.join(",\n"))
}

fn write_tile_rgbds(variable_name: &str, encoded_tile: &EncodedTile) -> String {
    let end_symbol = format!("{}_end", variable_name);
    let preamble = format!(
        "SECTION \"Tiles for '{}'\", ROM0\n\nEXPORT {}, {}\n\n{}:",
        variable_name, variable_name, end_symbol, variable_name
    );
    let mut body = Vec::new();
    for line in encoded_tile.tile_data.chunks(16) {
        let mut formatted_bytes = Vec::new();
        for byte in line {
            formatted_bytes.push(format!("${:02x}", byte));
        }
        body.push(format!("    db {}", formatted_bytes.join(",")));
    }

    format!("{}\n{}\n{}:\n", preamble, body.join(",\n"), end_symbol)
}

fn write_tile(
    encoded_tile: &EncodedTile,
    out_file: &str,
    output_type: OutputType,
) -> Result<(), io::Error> {
    let variable_name = Path::new(&encoded_tile.input_filename)
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .expect(&format!(
            "Invalid file name: {}",
            encoded_tile.input_filename
        ));
    let formatted_result = match output_type {
        OutputType::Gbdk => write_tile_gbdk(&variable_name, encoded_tile),
        OutputType::Rgbds => write_tile_rgbds(&variable_name, encoded_tile),
    };
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_file)?;
    file.write_all(formatted_result.as_bytes())?;
    Ok(())
}

fn main() {
    let matches = App::new("Gameboy Tile Generator")
        .version("0.2.0")
        .author("Blake Smith <blakesmith0@gmail.com>")
        .about("Generate GBDK or RGBDS Game Boy tiles from PNG images")
        .arg(
            Arg::with_name("debug")
                .help("Enable debug logging")
                .short("d"),
        )
        .arg(
            Arg::with_name("input")
                .help("The PNG image to generate tiles from. Example: 'image.png'")
                .short("i")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .help("The output file to generate. Usually something like 'tiles.h' for GBDK output, or 'tiles.asm' for RGBDS")
                .short("o")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output-type")
                .help("The output type. Either 'gbdk' or 'rgbds'. Defaults to 'gbdk'")
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
        Some("gbdk") => OutputType::Gbdk,
        Some("rgbds") => OutputType::Rgbds,
        _ => OutputType::Gbdk,
    };

    let args = CommandArguments {
        input: matches.value_of("input").unwrap().to_string(),
        output: matches.value_of("output").unwrap().to_string(),
        output_type: output_type,
    };

    let decoded_image = decode_image(&args.input).expect("Could not decode image");
    let encoded_tile = encode_tile(decoded_image);
    write_tile(&encoded_tile, &args.output, args.output_type).expect("Could not write out tile");

    log::debug!("Arguments are: {:?}", args);
}
