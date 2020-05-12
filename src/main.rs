use clap::{App, Arg};

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

fn main() {
    let matches = App::new("Gameboy Tile Generator")
        .version("0.1")
        .author("Blake Smith <blakesmith0@gmail.com>")
        .about("Generate GBDK Game Boy tiles from PNG images")
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

    let output_type = match matches.value_of("output-type") {
        Some("gbdk") => OutputType::GBDK,
        _ => OutputType::GBDK,
    };

    let args = CommandArguments {
        input: matches.value_of("input").unwrap().to_string(),
        output: matches.value_of("output").unwrap().to_string(),
        output_type: output_type,
    };

    println!("Arguments are: {:?}", args);
}
