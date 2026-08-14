mod label;
mod paint;
mod parse;

use std::sync::Arc;
use std::time::Duration;

use egui::Sense;

use crate::bridge::time_labels::TimelineTimeLabels;
use crate::gui::{GlobalTimerManager, KEY_TIMER_MANAGER, timer::TimerHandle};
use label::{LabelId, LabelLayout};
use paint::PaintParams;
use parse::parse_labels_to_layout;

pub const MARGIN_TOP: f32 = 8.0;

pub struct TimeLabelsBar {
    time_manager: GlobalTimerManager,
    key_timer: Option<Arc<TimerHandle>>,

    counter: i32,

    duration: f32,
    visible_range: (f32, f32),

    label_layout: LabelLayout,
    root_label_group: LabelId,
}

impl TimeLabelsBar {
    pub fn new(ctx: &egui::Context, duration: f32, time_labels: TimelineTimeLabels) -> Self {
        let time_manager =
            ctx.data(|data| data.get_temp(egui::Id::new(KEY_TIMER_MANAGER)).unwrap());

        let (label_layout, root_label_group) = parse_labels_to_layout(duration, time_labels);

        Self {
            time_manager,
            key_timer: None,
            counter: 0,
            duration,
            visible_range: (0.0, duration.min(20.0)),
            label_layout,
            root_label_group,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.process_key(ui);
        self.paint(ui);
    }

    fn process_key(&mut self, ui: &mut egui::Ui) {
        let (w, a, s, d) = ui.input(|i| {
            (
                i.key_down(egui::Key::W),
                i.key_down(egui::Key::A),
                i.key_down(egui::Key::S),
                i.key_down(egui::Key::D),
            )
        });
        let pressing = w || a || s || d;
        if self.key_timer.is_none() && pressing {
            self.key_timer = Some(
                self.time_manager
                    .lock()
                    .start(Duration::from_secs_f64(1.0 / 60.0)),
            );
        }
        if self.key_timer.is_some() && !pressing {
            self.key_timer = None;
        }
        if let Some(timer) = &self.key_timer
            && timer.timeout()
        {
            self.move_visible_range(w, a, s, d);
        }
    }

    fn move_visible_range(&mut self, w: bool, a: bool, s: bool, d: bool) {
        println!("{} {} {} {}", w, a, s, d);
        self.counter += 1;
    }

    fn paint(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

        // let mut top_margin_rect = response.rect;
        // top_margin_rect.max.y = top_margin_rect.min.y + MARGIN_TOP;

        // painter.rect_filled(top_margin_rect, 0.0, egui::Color32::GRAY);

        let mut layout_rect = response.rect;
        layout_rect.min.y += MARGIN_TOP;

        if layout_rect.min.y < layout_rect.max.y {
            painter.rect_filled(layout_rect, 0.0, egui::Color32::LIGHT_GRAY);
            // self.label_layout.paint(
            //     &painter.with_clip_rect(layout_rect),
            //     &PaintParams {
            //         rect: layout_rect,
            //         visible_range: (0.0, 5.0),
            //         y_pixel_offset: 0.0,
            //     },
            //     self.root_label_group,
            //     0,
            // );
        }

        painter.text(
            layout_rect.left_top() + egui::Vec2::new(50.0, 50.0),
            egui::Align2::LEFT_TOP,
            format!("Counter: {}", self.counter),
            egui::FontId::proportional(12.0),
            egui::Color32::BLACK,
        );
    }
}
