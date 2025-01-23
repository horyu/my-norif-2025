#[cfg(not(target_os = "windows"))]
compile_error!("only windows is supported");

use image::ImageReader;
use std::{
    fs::{File, metadata},
    io::Write,
    path::Path,
};

const SOURCE_ICON: &str = "icon.png";
const DEST_ICON_DATA: &str = "src/icon_data.rs";
const DEST_ICO: &str = "icon.ico";

fn main() -> Result<(), std::io::Error> {
    generate_icon_data();
    generate_ico_file();

    winres::WindowsResource::new().set_icon(DEST_ICO).compile()
}

fn generate_icon_data() {
    let src_path = Path::new(SOURCE_ICON);
    let dest_path = Path::new(DEST_ICON_DATA);

    if should_regenerate_file(src_path, dest_path) {
        let img = ImageReader::open(src_path)
            .expect("Failed to open icon.png")
            .decode()
            .expect("Failed to decode icon.png");

        let height = img.height();
        let width = img.width();
        let rgba = img.into_rgba8().into_raw();

        let mut file = File::create(dest_path).expect("Failed to create file");
        write!(
            file,
            "#![cfg_attr(any(), rustfmt::skip)]\npub const ICON_WIDTH: u32 = {};\npub const ICON_HEIGHT: u32 = {};\npub const ICON_RGBA: &[u8] = &{:?};",
            width, height, rgba
        ).expect("Failed to write to file");
    }
}

fn generate_ico_file() {
    let src_path = Path::new(SOURCE_ICON);
    let dest_path = Path::new(DEST_ICO);

    if should_regenerate_file(src_path, dest_path) {
        let img = ImageReader::open(src_path)
            .expect("Failed to open icon.png")
            .decode()
            .expect("Failed to decode icon.png");

        img.save(dest_path).expect("Failed to save icon.ico");
    }
}

fn should_regenerate_file(src_path: &Path, dest_path: &Path) -> bool {
    let src_metadata =
        metadata(src_path).unwrap_or_else(|_| panic!("Failed to get metadata for {src_path:?}"));
    let dest_metadata = metadata(dest_path).ok();

    match dest_metadata {
        Some(dest_metadata) => {
            let src_modified = src_metadata
                .modified()
                .unwrap_or_else(|_| panic!("Failed to get modification time for {src_path:?}"));
            let dest_modified = dest_metadata
                .modified()
                .unwrap_or_else(|_| panic!("Failed to get modification time for {dest_path:?}"));
            dest_modified < src_modified
        }
        None => true,
    }
}
