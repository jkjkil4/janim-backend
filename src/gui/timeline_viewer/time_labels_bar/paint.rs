use egui::{Align2, Color32, FontId, Rect, StrokeKind};

use crate::gui::timeline_viewer::time_labels_bar::label::LABEL_GROUP_HEADER_EXPANDED_HEIGHT;

use super::label::LABEL_PIXEL_HEIGHT_PER_UNIT;

use super::label::{LabelData, LabelId, LabelLayout};

#[derive(Default, Clone, Copy)]
pub struct Color {
    pub pen: egui::Stroke,
    pub brush: egui::Color32,
}

#[allow(unused)]
impl Color {
    #[inline]
    pub fn to_rgba(rgb: (u8, u8, u8)) -> (u8, u8, u8, u8) {
        (rgb.0, rgb.1, rgb.2, 255)
    }
    #[inline]
    pub fn pen(width: f32, rgba: (u8, u8, u8, u8)) -> Self {
        Self {
            pen: egui::Stroke::new(width, Color::color32(rgba)),
            brush: egui::Color32::TRANSPARENT,
        }
    }
    #[inline]
    pub fn brush(rgba: (u8, u8, u8, u8)) -> Self {
        Self {
            pen: egui::Stroke::NONE,
            brush: Color::color32(rgba),
        }
    }
    #[inline]
    pub fn all(width: f32, pen_rgba: (u8, u8, u8, u8), brush_rgba: (u8, u8, u8, u8)) -> Self {
        Self {
            pen: egui::Stroke::new(width, Color::color32(pen_rgba)),
            brush: Color::color32(brush_rgba),
        }
    }
    #[inline]
    fn color32(rgba: (u8, u8, u8, u8)) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(rgba.0, rgba.1, rgba.2, rgba.3)
    }
}

pub struct PaintParams {
    pub rect: Rect,
    pub visible_range: (f32, f32),
    pub y_pixel_offset: f32,
}

impl LabelLayout {
    pub(crate) fn paint(
        &self,
        painter: &egui::Painter,
        params: &PaintParams,
        id: LabelId,
        global_offset: i32,
    ) {
        let node = self.node(id);
        match &node.data {
            LabelData::LabelItem(data) => {
                LabelLayout::paint_label_rect(
                    painter,
                    params,
                    node.info.range,
                    global_offset + self.node_y(id),
                    data.height,
                    node.info.color.clone(),
                    node.info.text.as_ref(),
                    None,
                );
            }
            LabelData::LabelGroup(data) => {
                // Draw child-labels
                if !data.collapsed {
                    let mut children_global_offset = global_offset + self.node_y(id);
                    if data.header {
                        children_global_offset += LABEL_GROUP_HEADER_EXPANDED_HEIGHT;
                    }

                    match &data.layers {
                        // Optimized version, iterate child-labels in layers;
                        // enable when there are too-many child-labels in a group
                        Some(layers) => {
                            for layer in layers {
                                let left = layer
                                    .partition_point(|x| {
                                        self.node(*x).at() < params.visible_range.0
                                    })
                                    .saturating_sub(1);
                                let right = (layer.partition_point(|x| {
                                    self.node(*x).end() <= params.visible_range.1
                                }) + 1)
                                    .min(layer.len());

                                for child in &layer[left..right] {
                                    self.paint(painter, params, *child, children_global_offset);
                                }
                            }
                        }
                        // Simple version, iterate child-labels directly
                        None => {
                            for child in &data.children {
                                self.paint(painter, params, *child, children_global_offset);
                            }
                        }
                    }
                }

                // Draw the header
                if data.header {
                    // TODO: paint_tip
                    LabelLayout::paint_label_rect(
                        painter,
                        params,
                        node.info.range,
                        global_offset,
                        data.header_height(),
                        node.info.color.clone(),
                        node.info.text.as_ref(),
                        Some(data.collapsed),
                    );
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_label_rect(
        painter: &egui::Painter,
        params: &PaintParams,
        range: (f32, f32),
        global_y: i32,
        height: i32,
        color: Color,
        text: Option<&String>,
        // `None` indicates `LabelItem`; `Some()` indicates `LabelGroup`
        label_group_collapsed: Option<bool>,
    ) {
        let range_px = PixelRange::from_time_range(params, range);
        let y_px = params.rect.min.y + (global_y as f32) * LABEL_PIXEL_HEIGHT_PER_UNIT
            - params.y_pixel_offset;
        let mut rect = Rect::from_min_size(
            egui::pos2(range_px.left, y_px),
            egui::vec2(
                range_px.width,
                (height as f32) * LABEL_PIXEL_HEIGHT_PER_UNIT,
            ),
        );

        let mut out_of_boundary = false;

        // Ensure ranges overflow past the bottom still have a visible border
        let maximum = params.rect.max.y - 4.0;
        if rect.min.y > maximum {
            rect.min.y = maximum;
            rect.max.y = rect.min.y + 4.0;
            out_of_boundary = true;
        }

        // Ensure ranges overflow past the top still have a visible border
        let minimum = params.rect.min.y + 4.0;
        if rect.max.y < minimum {
            rect.max.y = minimum;
            rect.min.y = rect.max.y - 4.0;
            out_of_boundary = true;
        }

        // The `else if` and `else` arms makes too-slim ranges still visible
        let x_adjust = if rect.width() > 5.0 {
            2.0
        } else if rect.width() > 1.0 {
            (rect.width() - 1.0) / 2.0
        } else {
            0.0
        };

        // Draw the background
        if !out_of_boundary {
            adjust_rect(&mut rect, x_adjust, 2.0, -x_adjust, -2.0);
        }
        painter.rect(rect, 0.0, color.brush, color.pen, StrokeKind::Inside);

        if !out_of_boundary && let Some(text) = text {
            // When part of a range is outside the visible area on the left,
            // keep the text aligned to the left edge of the screen instead of letting it go off-screen.
            if rect.min.x < params.rect.min.x {
                rect.min.x = params.rect.min.x
            }

            let mut font_size = 12.0;
            if let Some(collapsed) = label_group_collapsed
                && !collapsed
            {
                font_size *= 0.7;
            }

            adjust_rect(&mut rect, 1.0, 1.0, -1.0, -1.0);
            painter.with_clip_rect(rect).text(
                rect.left_center(),
                Align2::LEFT_CENTER,
                text,
                FontId::proportional(font_size),
                Color32::WHITE,
            );
        }
    }
}

struct PixelRange {
    pub left: f32,
    pub width: f32,
}

impl PixelRange {
    /// `range: (f32, f32)` indicates the start point and end point,
    /// which is different with [PixelRange]
    fn from_time_range(params: &PaintParams, range: (f32, f32)) -> Self {
        let visible_duration = params.visible_range.1 - params.visible_range.0;
        let left = params.rect.min.x
            + (range.0 - params.visible_range.0) / visible_duration * params.rect.width();
        let width = (range.1 - range.0) / visible_duration * params.rect.width();
        Self { left, width }
    }
}

fn adjust_rect(rect: &mut Rect, x1: f32, y1: f32, x2: f32, y2: f32) {
    rect.min.x += x1;
    rect.min.y += y1;
    rect.max.x += x2;
    rect.max.y += y2;
}
