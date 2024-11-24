use std::{
    fs,
    io::{Read, Write},
};

/// This will take a ppm file (nothing else for now) and extract the pixel data from it.
///
/// To use with the build.rs files of the apps
///
/// TODO : take a file and make it in a column

pub fn extract_data_from_file(input_file: &'static str, output_file: &'static str) {
    let mut file = fs::File::open(input_file).expect("Failed to read input image.");
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents)
        .expect("Failed to read input image.");

    let pixel_data_start = find_pixel_data_start(&contents).expect("Failed to parse PPM File.");

    let pixel_data = &contents[pixel_data_start..];
    let modified_data = get_rgb565_data(pixel_data);

    let mut output = fs::File::create(&output_file).expect("Failed to create output file");
    output
        .write_all(&modified_data)
        .expect("Failed to write output file.");

    println!("cargo:rerun-if-changed={}", input_file);
}

/// Finds the start of the pixel data in a PPM file by parsing its header.
fn find_pixel_data_start(data: &[u8]) -> Option<usize> {
    // Parse the PPM header
    let mut index = 0;

    // Check for the magic number
    if &data[index..index + 2] != b"P6" {
        return None;
    }
    index += 2;

    // Skip whitespace after the magic number
    while index < data.len() && data[index].is_ascii_whitespace() {
        index += 1;
    }

    // Parse width and height (skip them)
    for _ in 0..2 {
        while index < data.len() && !data[index].is_ascii_whitespace() {
            index += 1;
        }
        while index < data.len() && data[index].is_ascii_whitespace() {
            index += 1;
        }
    }

    // Parse max color value (skip it)
    while index < data.len() && !data[index].is_ascii_whitespace() {
        index += 1;
    }
    while index < data.len() && data[index].is_ascii_whitespace() {
        index += 1;
    }

    // The remaining bytes are pixel data
    Some(index)
}

fn get_rgb565_data(data: &[u8]) -> Vec<u8> {
    let mut res = Vec::new();
    for (r, g, b) in data
        .iter()
        .enumerate()
        .step_by(3)
        .map(|(i, x)| (x, data.get(i + 1).unwrap(), data.get(i + 2).unwrap()))
    {
        let pixel: u16 =
            ((*r as u16 & 0b1111_1000) << 8) | ((*g as u16 & 0b1111_1100) << 3) | (*b as u16 >> 3);
        let up = (pixel >> 8) as u8;
        let down = (pixel) as u8;
        res.push(up);
        res.push(down);
    }
    return res;
}
