use std::process::Command;

/// This constant is massively important !!!
///
/// Needed for the preprocessor to know where to cut a tile.
///
/// (used for optimisation)
const TILESIZE: u16 = 55;

fn main() {
    // Turn icon.png into icon.nwi
    println!("cargo:rerun-if-changed=src/data/icon.png");
    println!("cargo:rerun-if-changed=src/data/image.ppm");
    let output = Command::new("nwlink")
        .args(["png-nwi", "src/data/icon.png", "target/icon.nwi"])
        .output()
        .expect("Failure to launch process");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Turn image.ppm into image.nppm
    nppm_decoder::decoder::extract_data_from_file(
        "src/data/image.ppm",
        "src/data/image.nppm",
        TILESIZE,
    );
}
