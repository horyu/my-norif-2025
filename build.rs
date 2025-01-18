#[cfg(not(target_os = "windows"))]
compile_error!("only windows is supported");

fn main() {
    use image::ImageReader;
    use std::{
        fs::{File, metadata},
        io::Write,
        path::Path,
    };

    let src_path = Path::new("icon.png");
    let dest_path = Path::new("src/icon_data.rs");

    let src_metadata = metadata(src_path).expect("Failed to get metadata for icon.png");
    let dest_metadata = metadata(dest_path).ok();

    let should_process = match dest_metadata {
        Some(dest_metadata) => {
            let src_modified = src_metadata
                .modified()
                .expect("Failed to get modification time for icon.png");
            let dest_modified = dest_metadata
                .modified()
                .expect("Failed to get modification time for src/icon_data.rs");
            src_modified > dest_modified
        }
        None => true,
    };

    if should_process {
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
