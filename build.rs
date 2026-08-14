use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        // For debugging directly on desktop
        egui_desktop: { not(any(wasm, feature = "no-egui-desktop")) },
    }

    // wesl::Wesl::new("src/shaders")
    //     .build_artifact(&"package::time_labels".parse().unwrap(), "time_labels");
}
