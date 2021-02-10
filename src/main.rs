mod util;

use std::path::Path;
use std::ffi::OsStr;
use clap::{Arg, App};

use util::{parse, UnwrapOrDie};

/**
 * Enumeration defining the supported image formats
 */
#[derive(Debug)]
enum ImageFormat {
    Elf(Option<usize>),
    Hex(Option<usize>),
    Dfu(Option<usize>),
    Bin(Option<usize>)
}

// Define version here
const APP_NAME: &str = "Rust DFU Firmware Uploader";
const VERSION: &str = "1.0";

fn main() {
    // Create the CLI Parser
    let appdef = 
        App::new(APP_NAME)
            .version(VERSION)
            .author("Johnny Egeland")
            .about("Utility to upload Firmware images to DFU capable hardware. Supports a number of formats: iHEX, ELF, DFU and BIN")
            .arg(Arg::with_name("format")
                .short("f")
                .long("format")
                .value_name("FORMAT")
                .help("Explicitly specify the format (extension is used by default): dfu, hex, elf or bin")
                .takes_value(true))
            .arg(Arg::with_name("offset")
                .short("o")
                .long("offset")
                .value_name("OFFSET")
                .help("Explicitly specify the target offset to apply. For 'bin' files, this is the address to upload to. Use 0x<offset> to specify in hex.")
                .takes_value(true))
            .arg(Arg::with_name("image")
                .value_name("IMAGE")
                .help("The firmware image file to upload via DFU")
                .required(true)
                .index(1));

    // Get the matches object, and extract parameters needed
    let cli_matches = appdef.get_matches();

    // At this point, start by printing appname and version
    println!("{} v{}", APP_NAME, VERSION);

    // Parse the Offset
    let fw_offset = if let Some(offstr) = cli_matches.value_of("offset") {
        Some(parse::usize_from_string(offstr).unwrap_or_die(1, "Unable to parse the given offset parameter"))
    }
    else { None };

    // Get the image filename as a string
    let fw_image_file = cli_matches.value_of("image").unwrap().to_string();

    // Get the format value as string as well
    let fw_image_type = match cli_matches.value_of("format") {
        Some(s) => parse_image_type_from_extension(&s.to_string(), fw_offset),
        None => parse_image_type_from_extension(&get_file_extension(&fw_image_file, "elf"), fw_offset)
    };


    println!("Using image file: {}", fw_image_file);
    println!("Using format: {:?} @ 0x{:08X}", fw_image_type, fw_offset.unwrap_or(0));
    
}

/// Returns the file extension in lower case, or the default value as a string
/// # Arguments
/// * `filename` - The filename to get extension for
/// * `default` - The default value to return if no extension
/// 
/// # Return
/// The file extension in lower case, or the given default string (also in lower case)
fn get_file_extension(filename: &String, default: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or(default)
        .to_lowercase()
}

/// Parses the given type string, and returns the matching image format enum
/// An optional offset parameter can be given, which will be provided with the format type
/// # Arguments
/// * `extension` - The extension to get type for
/// * `offset` - Optional offset which is returned with the type
fn parse_image_type_from_extension(extension: &String, offset: Option<usize>) -> ImageFormat {
    match extension.as_str() {
        "dfu" => ImageFormat::Dfu(offset),
        "bin" => ImageFormat::Bin(offset),
        "hex" => ImageFormat::Hex(offset),
        &_ => ImageFormat::Elf(offset),
    }
}
