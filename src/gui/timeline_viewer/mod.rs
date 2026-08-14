mod time_labels_bar;

use time_labels_bar::TimeLabelsBar;

use crate::{bridge::ExtractedTimeline, gui::timeline_viewer};

pub struct TimelineViewer {
    time_labels: TimeLabelsBar,

    timeline_name: String,

    progress: i32,
    frame_skip: bool,
}

impl TimelineViewer {
    pub fn new(ctx: &egui::Context, extracted: ExtractedTimeline) -> Self {
        Self {
            time_labels: TimeLabelsBar::new(ctx, extracted.duration, extracted.time_labels),
            timeline_name: extracted.timeline_name,
            progress: 0,
            frame_skip: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.timeline_name
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.status_bar(ui);

        let old_radius = ui.style().interaction.resize_grab_radius_side;
        ui.style_mut().interaction.resize_grab_radius_side = time_labels_bar::MARGIN_TOP;
        egui::Panel::bottom("time_labels")
            .frame(egui::Frame::NONE.inner_margin(0.0))
            .min_size(0.0)
            .default_size(160.0)
            .resizable(true)
            .show(ui, |ui| {
                self.time_labels.ui(ui);
            });
        ui.style_mut().interaction.resize_grab_radius_side = old_radius;

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(0.0))
            .show(ui, |ui| {
                ui.centered_and_justified(|ui| ui.label("Central Panel"));
            });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status_bar")
            .exact_size(26.0)
            .show_separator_line(false)
            .frame(
                egui::Frame::NONE
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(format!("FPS: {}", 60));
                    ui.add_space(8.0);
                    ui.checkbox(&mut self.frame_skip, t!("Frame skip"));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        use super::assets::{CAPTURE, EXPORT};

                        if TimelineViewer::icon_button(ui, CAPTURE).clicked() {
                            self.capture();
                        }

                        if TimelineViewer::icon_button(ui, EXPORT).clicked() {
                            self.export();
                        }

                        ui.add(egui::Button::new("1.2s"));

                        ui.add(
                            egui::TextEdit::singleline(&mut self.timeline_name)
                                .font(egui::FontId::proportional(11.0))
                                .background_color(ui.visuals().code_bg_color)
                                .desired_width(200.0),
                        );
                    });
                });
            });
    }

    fn icon_button<'a>(ui: &mut egui::Ui, image_src: impl Into<egui::Image<'a>>) -> egui::Response {
        let image: egui::Image<'a> = image_src.into();
        ui.add(egui::Button::image(image.tint(ui.visuals().text_color())))
    }

    pub fn capture(&mut self) {
        println!("Capture");
    }

    pub fn export(&mut self) {
        println!("Export");
    }
}
