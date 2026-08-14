mod app;
mod assets;
mod menus;
mod timeline_viewer;

mod timer;

use std::sync::Arc;

use pyo3::prelude::*;

use eframe::{
    AppCreator, WgpuConfiguration,
    egui_wgpu::{WgpuSetup, WgpuSetupCreateNew},
    wgpu,
};
use egui::{
    ViewportBuilder,
    mutex::{Mutex, MutexGuard},
};
use egui_extras::install_image_loaders;

use assets::APP_ICON_RAW_BYTES;
use timer::TimerManager;

use crate::bridge::AppArgs;

const KEY_WGPU_TARGET_FORMAT: &str = "wgpu_target_format";
const KEY_TIMER_MANAGER: &str = "timer_manager";

#[derive(Clone)]
struct GlobalTimerManager {
    inst: Arc<Mutex<TimerManager>>,
}

impl GlobalTimerManager {
    pub fn new(ctx: egui::Context) -> Self {
        Self {
            inst: Arc::new(Mutex::new(TimerManager::new(ctx))),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, TimerManager> {
        self.inst.lock()
    }
}

pub fn run(args: AppArgs) -> eframe::Result {
    #[cfg(not(wasm))]
    run_native(args)?;

    #[cfg(wasm)]
    run_web(args);

    Ok(())
}

#[cfg(not(wasm))]
fn run_native(args: AppArgs) -> eframe::Result {
    let icon = egui::IconData {
        rgba: APP_ICON_RAW_BYTES.to_vec(),
        width: 64,
        height: 64,
    };

    #[allow(unused_mut)]
    let mut viewport = ViewportBuilder::default()
        .with_min_inner_size([400.0, 200.0])
        .with_inner_size([800.0, 600.0])
        .with_icon(icon);

    // Disable native decorations to show custom title on desktop targets
    #[cfg(egui_desktop)]
    {
        viewport = viewport.with_decorations(false);
    }

    let options = eframe::NativeOptions {
        wgpu_options: WgpuConfiguration {
            wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                instance_descriptor: wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::GL,
                    ..wgpu::InstanceDescriptor::new_without_display_handle()
                },
                ..WgpuSetupCreateNew::without_display_handle()
            }),
            ..Default::default()
        },
        viewport,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "JAnim Graphics",
        options,
        Box::new(|cc| {
            install_image_loaders(&cc.egui_ctx);

            let _ = egui_chinese_font::setup_chinese_fonts(&cc.egui_ctx);
            egui_system_fonts::add_auto(&cc.egui_ctx, egui_system_fonts::FontStyle::Sans);

            let Some(render_state) = cc.wgpu_render_state.as_ref() else {
                return Err(t!("Failed to obtain `wgpu_render_state`").into());
            };
            cc.egui_ctx.data_mut(|data| {
                data.insert_temp(
                    egui::Id::new(KEY_WGPU_TARGET_FORMAT),
                    render_state.target_format,
                );
            });

            let timer_manager = GlobalTimerManager::new(cc.egui_ctx.clone());
            cc.egui_ctx.data_mut(|data| {
                data.insert_temp(egui::Id::new(KEY_TIMER_MANAGER), timer_manager);
            });

            let app = app::App::new(&cc.egui_ctx, args)?;
            Ok(Box::new(app))
        }),
    )
}

#[cfg(wasm)]
fn run_web(args: AppArgs) {
    wasm_bindgen_futures::spawn_local(async {
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("the_canvas_id")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    let app = app::App::new(&cc.egui_ctx, args);
                    Ok(Box::new(app))
                }),
            )
            .await
            .expect("failed to start egui");
    });
}

#[pymodule]
pub mod gui {
    use pyo3::exceptions::PyException;
    use pyo3::prelude::*;

    use crate::bridge::{AppArgs, PyAppArgs};

    #[pyfunction]
    pub fn exec(args: Bound<'_, PyAppArgs>) -> PyResult<()> {
        let r_args = AppArgs::resolve(args)?;
        super::run(r_args).map_err(|e| PyException::new_err(e.to_string()))
    }
}
