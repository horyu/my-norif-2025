#[cfg(not(target_os = "windows"))]
compile_error!("only windows is supported");

use image::ImageReader;
use std::{env, ffi::OsString, fs::File, io::Write, path::Path};

fn main() -> Result<(), std::io::Error> {
    println!("cargo::rerun-if-changed=icon.png");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let img = ImageReader::open("icon.png")
        .expect("Failed to open icon.png")
        .decode()
        .expect("Failed to decode icon.png");
    generate_ico_file(&out_dir, &img);
    generate_icon_data(&out_dir, &img);

    winres::WindowsResource::new()
        .set_icon("icon.ico")
        .compile()
}

fn generate_ico_file(out_dir: &OsString, img: &image::DynamicImage) {
    let dest_path = Path::new(out_dir).join("icon.ico");
    img.save(dest_path).expect("Failed to save icon.ico");
}

fn generate_icon_data(out_dir: &OsString, img: &image::DynamicImage) {
    let dest_path = Path::new(&out_dir).join("icon_data.rs");

    let height = img.height();
    let width = img.width();
    let rgba = img.to_rgba8().into_raw();

    let mut file = File::create(dest_path).expect("Failed to create file icon_data.rs");
    write!(
        file,
        "const ICON_WIDTH: u32 = {};\nconst ICON_HEIGHT: u32 = {};\nconst ICON_RGBA: &[u8] = &{:?};",
        width, height, rgba
    ).expect("Failed to write to file");
}
