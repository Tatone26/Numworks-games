use nppm_decoder::decoder::extract_data_from_file;

fn main() {
    extract_data_from_file("image.ppm", "image.nppm", 35);
}
