use core::str;
use std::{
    fs,
    io::{Read, Write},
};

/// This will take a ppm file (nothing else for now) and extract the pixel data from it.
///
/// To use with the build.rs files of the apps
///
/// TODO : read other things than ppm files (pngs)

pub fn extract_data_from_file(input_file: &'static str, output_file: &'static str, tile_size: u16) {
    if tile_size == 1 {
        panic!("Tilesize can't be equal to 1.");
    }
    let mut file = fs::File::open(input_file).expect("Failed to read input image.");
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents)
        .expect("Failed to read input image.");

    let (width, height, pixel_data_start) =
        find_pixel_data_start(&contents).expect("Failed to parse PPM File.");

    let pixel_data = &contents[pixel_data_start..];
    let rgb_data = get_rgb565_data(pixel_data);
    let translated_data = translate_pixel_data(rgb_data, width, height, tile_size)
        .expect("Width or height of image not a multiple of tilesize");
    let final_data = to_u8_data(translated_data);

    let mut output = fs::File::create(output_file).expect("Failed to create output file");
    output
        .write_all(&final_data)
        .expect("Failed to write output file.");
}

/// Needs tile size to be able to translate !!
/// -> how to do that ??
fn translate_pixel_data(
    rgb_data: Vec<u16>,
    width: u16,
    height: u16,
    tilesize: u16,
) -> Option<Vec<u16>> {
    let mut new_data = Vec::new();

    if width % tilesize != 0 || height % tilesize != 0 || tilesize == 1 {
        eprintln!("Warning : Width of tileset not a multiple of tilesize, or tilesize == 1.");
        return None;
    }
    /* let width_in_tiles: u16 = width / tilesize;
    let height_in_tiles: u16 = height / tilesize; */

    for y_tile in (0..height).step_by(tilesize as usize) {
        for x_tile in (0..width).step_by(tilesize as usize) {
            // all tiles, one after the other, line by line

            for y in y_tile..(y_tile + tilesize) {
                for x in x_tile..(x_tile + tilesize) {
                    // Every single pixel, line by line
                    new_data.push(*rgb_data.get((y * width + x) as usize).unwrap());
                }
            }
        }
    }

    Some(new_data)
}

fn to_u8_data(data: Vec<u16>) -> Vec<u8> {
    let mut new_vec = Vec::new();

    for d in data {
        new_vec.push(d as u8);
        new_vec.push((d >> 8) as u8);
    }

    new_vec
}

/// Finds the start of the pixel data in a PPM file by parsing its header.
fn find_pixel_data_start(data: &[u8]) -> Option<(u16, u16, usize)> {
    // Parse the PPM header
    let mut index = 0;
    let mut size: [u16; 2] = [0, 0];
    // Check for the magic number
    if &data[index..index + 2] != b"P6" {
        return None;
    }
    index += 2;

    // Skip whitespace after the magic number
    while index < data.len() && data[index].is_ascii_whitespace() {
        index += 1;
    }

    // Some commentaries could be there (thanks Gimp)
    if data[index] == b'#' {
        while index < data.len() && data[index] != b'\n' {
            index += 1;
        }
    }
    while index < data.len() && data[index].is_ascii_whitespace() {
        index += 1;
    }

    // Parse width and height
    for h in 0..2 {
        let mut word = Vec::new();
        while index < data.len() && !data[index].is_ascii_whitespace() {
            word.push(data[index]);
            index += 1;
        }
        let s = match str::from_utf8(&word) {
            Ok(v) => v,
            Err(_) => return None,
        };
        size[h] = s.parse().unwrap();

        // whitespace
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
    Some((size[0], size[1], index))
}

fn get_rgb565_data(data: &[u8]) -> Vec<u16> {
    let mut res = Vec::new();
    for (r, g, b) in data
        .iter()
        .enumerate()
        .step_by(3)
        .map(|(i, x)| (x, data.get(i + 1).unwrap(), data.get(i + 2).unwrap()))
    {
        let pixel: u16 =
            ((*r as u16 & 0b1111_1000) << 8) | ((*g as u16 & 0b1111_1100) << 3) | (*b as u16 >> 3);
        res.push(pixel);
    }
    res
}
