#[cfg(egui_desktop)]
pub const APP_ICON: &[u8] = include_bytes!("app_icon.png");

pub const APP_ICON_RAW_BYTES: &[u8] = include_bytes!("app_icon.rawbytes");

pub const CAPTURE: egui::ImageSource = egui::include_image!("capture.png");
pub const EXPORT: egui::ImageSource = egui::include_image!("export.png");
