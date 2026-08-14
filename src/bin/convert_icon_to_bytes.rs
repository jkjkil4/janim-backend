use std::fs;

fn main() {
    let img = image::open("src/gui/assets/app_icon.png")
        .unwrap()
        .into_rgba8();

    let bytes = img.into_raw();

    fs::write("src/gui/assets/app_icon.rawbytes", bytes).unwrap();
}
