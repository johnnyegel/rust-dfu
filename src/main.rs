mod util;

mod usb;

use std::{path::Path};
use std::ffi::OsStr;
use clap::{Arg, App};

use util::{parse, UnwrapOrDie};



/// Enumeration defining the supported image formats
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
            .author("Created by: Johnny Egeland (c) 2021")
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

    // Some simple USB enumeration here.
    for device in rusb::devices().unwrap().iter() {

        // Get the device descriptor
        let device_desc_result = device.device_descriptor();
        if let Err(e) = device_desc_result {
            println!("Unable to get device descriptor: {}", e);
            continue;
        }

        // Make sure there is a active configuration
        let active_config_result = device.active_config_descriptor();
        if let Err(e) = active_config_result {
            println!("Unable to get active config: {}", e);
            continue;
        }

        // Now unwrap should work
        let device_desc = device_desc_result.unwrap();
        let active_config = active_config_result.unwrap();

        // Grab the interfaces
        let interfaces = active_config.interfaces();
        let mut is_device_listed = false;

        // Create a mutable vector to store our string indices
        let mut string_ix_list: Vec<u8> = Vec::new();

        // Iterate the interfaces, and grab
        for interface in interfaces {
            for if_desc in interface.descriptors() {
                // Skip interfaces which are not DFU
                if if_desc.class_code() != 0xFE || if_desc.sub_class_code() != 0x01 { 
                    continue; 
                }

                // Display the device if not already done
                if !is_device_listed {

                    println!("Bus {:03} Device {:03} ID {:04x}:{:04x}, Class {:02X}:{:02X}",
                        device.bus_number(),
                        device.address(),
                        device_desc.vendor_id(),
                        device_desc.product_id(),
                        device_desc.class_code(),
                        device_desc.sub_class_code()
                    );

                    is_device_listed = true;
                }

                let string_index = if_desc.description_string_index().unwrap_or(0xFF);

                string_ix_list.push(string_index);

                // Display interface info
                println!(" - Interface [{}]: Class: {:02X}:{:02X}, Protocol: {:02X}, String: {:02X}",
                    interface.number(),
                    if_desc.class_code(),
                    if_desc.sub_class_code(),
                    if_desc.protocol_code(),
                    string_index
                );
            }
        }

        // If device was listed, we need to read strings
        let dev_desc_result = device.open();
        if let Err(e) = dev_desc_result {
            println!("Unable to open USB device: {}", e);
            continue;
        }
        
        let dev_desc = dev_desc_result.unwrap();

        //dev_desc.read_interface_string(, interface, timeout)



        //dev_desc.close();
    }


}

/// Returns the file extension in lower case, or the default value as a string
/// # Arguments
/// * `filename` - The filename to get extension for
/// * `default` - The default value to return if no extension
/// 
/// # Return
/// The file extension in lower case, or the given default string (also in lower case)
fn get_file_extension(filename: &str, default: &str) -> String {
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
