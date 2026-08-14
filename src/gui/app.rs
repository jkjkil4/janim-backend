use std::sync::Arc;

use egui::{Button, Margin, Panel, ScrollArea, Stroke, Vec2, mutex::Mutex};
use pyo3::{PyResult, Python};

use crate::{
    bridge::{AppArgs, ExtractedTimeline},
    gui::{
        GlobalTimerManager, KEY_TIMER_MANAGER,
        menus::{MenuCallbacks, Menus},
        timeline_viewer::TimelineViewer,
    },
};

struct Tabs {
    viewers: Vec<TimelineViewer>,
    active_index: usize,
}

impl Tabs {
    fn new(viewers: Vec<TimelineViewer>) -> Self {
        assert!(!viewers.is_empty());
        Self {
            viewers,
            active_index: 0,
        }
    }

    fn with_active<F>(&mut self, f: F)
    where
        F: FnOnce(&mut TimelineViewer),
    {
        f(self.viewers.get_mut(self.active_index).unwrap());
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        if self.viewers.len() != 1 {
            Panel::top("tabs")
                .frame(
                    egui::Frame::NONE
                        .fill(ui.visuals().code_bg_color)
                        .inner_margin(Margin::symmetric(8, 0)),
                )
                .exact_size(24.0)
                .show(ui, |ui| {
                    let spacing = ui.spacing_mut();
                    spacing.item_spacing.x = 6.0;
                    spacing.scroll.bar_width = 4.0;

                    let visuals = ui.visuals_mut();
                    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
                    visuals.widgets.active.bg_stroke = Stroke::NONE;

                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (i, viewer) in self.viewers.iter().enumerate() {
                                let button =
                                    Button::selectable(self.active_index == i, viewer.name())
                                        .corner_radius(0.0)
                                        .min_size(Vec2 { x: 20.0, y: 24.0 });
                                if ui.add(button).clicked() {
                                    self.active_index = i;
                                }
                            }
                        });
                    });
                });
        }

        self.with_active(|v| v.ui(ui));
    }
}

pub struct App {
    timer_manager: GlobalTimerManager,

    menus: Menus,
    tabs: Arc<Mutex<Tabs>>,
}

impl App {
    pub fn new(ctx: &egui::Context, args: AppArgs) -> PyResult<Self> {
        if args.setup_built_timelines.is_empty() {
            panic!("Setup timelines must be non-empty");
        }

        let viewers = Python::attach(|py| -> PyResult<Vec<TimelineViewer>> {
            let mut result = Vec::new();
            for built in args.setup_built_timelines {
                let extracted = args.callbacks.extract_information.call1(py, (built,))?;
                let r_extracted = ExtractedTimeline::resolve_any(extracted.into_bound(py))?;
                result.push(TimelineViewer::new(ctx, r_extracted));
            }
            Ok(result)
        })?;

        let tabs_inner = Tabs::new(viewers);
        let tabs = Arc::new(Mutex::new(tabs_inner));

        let callback = |f: fn(&mut TimelineViewer)| {
            let t = tabs.clone();
            Box::new(move || {
                let mut guard = t.lock();
                guard.with_active(f);
            })
        };

        Ok(Self {
            timer_manager: ctx
                .data(|data| data.get_temp(egui::Id::new(KEY_TIMER_MANAGER)).unwrap()),
            menus: Menus::new(MenuCallbacks {
                rebuild: Box::new(|| {
                    println!("Rebuild");
                }),
                export: callback(TimelineViewer::export),
                capture: callback(TimelineViewer::capture),
                set_in_point: Box::new(|| {}),
                set_out_point: Box::new(|| {}),
                reset_in_out_point: Box::new(|| {}),
                clear_font_cache: Box::new(|| {}),
                subitem_selector: Box::new(|| {}),
                profiler_first: Box::new(|| {}),
                profiler_second: Box::new(|| {}),
                rich_text_editor: Box::new(|| {}),
                font_list: Box::new(|| {}),
                color: Box::new(|| {}),
                copy_time_point: Box::new(|| {}),
            }),
            tabs,
        })
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.timer_manager.lock().update();
        self.menus.render(ui);
        self.tabs.lock().ui(ui);
    }
}
