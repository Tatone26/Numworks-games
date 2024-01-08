use std::{env, fs, io::Write};

macro_rules! file_error {
    () => {
        panic!("Input file seems to be wrong, or this program cannot read it for now.");
    };
}

fn read_number(input: &[u8], starting_index: usize) -> (u32, usize) {
    let mut i = starting_index;
    let mut result = 0;
    loop {
        if let Some(x) = input.get(i) {
            if x.is_ascii_digit() {
                result = result * 10 + (*x as char).to_digit(10).unwrap()
            } else if x.is_ascii_whitespace() {
                let mut j = i;
                while let Some(x) = input.get(j) {
                    if x.is_ascii_whitespace() {
                        j += 1;
                    } else {
                        break;
                    }
                }
                break (result, j);
            } else {
                file_error!();
            }
        } else {
            file_error!();
        }
        i += 1;
    }
}

fn read_pixel(input: &[u8], starting_index: usize, max_value: u32) -> Option<(u16, usize)> {
    if input.len() <= starting_index + 2 {
        return None;
    }
    let [r, g, b] = input[starting_index..(starting_index + 3)] else {
        return None;
    };
    let true_r = ((r as f32 / (max_value as f32)) * (31.0)).round() as u8;
    let true_b = ((b as f32 / (max_value as f32)) * (31.0)).round() as u8;
    let true_g = ((g as f32 / (max_value as f32)) * (63.0)).round() as u8;
    println!("truer {true_r} g {true_g} b {true_b:#04x} : rgb {r} {g} {b}");
    Some((
        ((true_r as u16) << 11 | (true_g as u16) << 5 | true_b as u16),
        starting_index + 3,
    ))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 && args[1] == "-h" {
        println!(
            "Creates two linked files from a PPM file :
        The first contains the useful info : 
            sizex (u16)
            sizey (u16)
            nb_of_colors_in_the_map (u8, for now, may get smaller next.)
            ... (u16 each)
            color map
            color map
            ...
        Of course, there is no whitespace, so it is easy to extract a struct from this file. 
        The second file is just a list of bytes, each being a pixel. 
            "
        );
        return;
    }
    if args.len() != 3 {
        println!("Not enough arguments ; need the bmp image and the output file.");
        return;
    }
    let input = fs::read(&args[1]).expect("The input file cannot be read.");
    if input.len() < 10 || input[0..2] != *"P6".as_bytes() || !input[2].is_ascii_whitespace() {
        file_error!();
    }

    let i = 3;
    let (width, i) = read_number(&input, i);
    let (height, i) = read_number(&input, i);
    let (max_value, mut i) = read_number(&input, i);
    assert!(max_value < 256); // otherwise, everything would take 2 bytes. easy to modify program to be able to read that if necessary.
                              // here, at i, is the data starting.
    println!("width : {width}, height : {height}, max_value : {max_value}");

    let mut data = vec![];
    while let Some((x, j)) = read_pixel(&input, i, max_value) {
        println!("pixel {x:#06x}");
        data.push(x);
        i = j;
    }
    let mut output_file =
        fs::File::create(&args[2]).expect("Could not create or open output file.");

    let mut info_as_bytes = [(width as u16).to_be_bytes(), (height as u16).to_be_bytes()].concat();
    let data_as_bytes = data.into_iter().flat_map(|u| u.to_be_bytes());
    info_as_bytes.extend(data_as_bytes);
    output_file
        .write_all(&info_as_bytes)
        .expect("Error writing to output file.");
}
