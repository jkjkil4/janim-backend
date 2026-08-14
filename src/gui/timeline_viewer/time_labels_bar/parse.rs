use std::collections::HashMap;

use super::label::{LabelId, LabelInfo, LabelLayout};
use super::paint::Color;
use crate::bridge::time_labels::{
    AnimTimeLabel, AudioTimeLabel, DebugTimeLabel, SubtimelineTimeLabel, TimelineTimeLabels,
};

pub fn parse_labels_to_layout(
    tl_duration: f32,
    time_labels: TimelineTimeLabels,
) -> (LabelLayout, LabelId) {
    let mut layout = LabelLayout::default();

    let subtimeline_group =
        parse_subtimeline_group(&mut layout, tl_duration, time_labels.subtimeline_labels);
    let debug_group = parse_debug_labels(&mut layout, tl_duration, time_labels.debug_labels);
    let audio_group = parse_audio_labels(&mut layout, tl_duration, time_labels.audio_labels);
    let anim_group = parse_anim_labels(&mut layout, tl_duration, time_labels.anim_labels);

    let mut children = Vec::new();
    if let Some(group) = subtimeline_group {
        children.push(group);
    }
    if let Some(group) = debug_group {
        children.push(group);
    }
    if let Some(group) = audio_group {
        children.push(group);
    }
    children.push(anim_group);

    let root_group = layout.create_group(
        LabelInfo {
            text: None,
            color: Color::default(),
            range: (0.0, tl_duration),
        },
        children,
        false,
        false,
        Color::default(),
    );
    (layout, root_group)
}

fn parse_subtimeline_group(
    layout: &mut LabelLayout,
    tl_duration: f32,
    py_labels: Vec<SubtimelineTimeLabel>,
) -> Option<LabelId> {
    if py_labels.is_empty() {
        return None;
    }

    let mut end = tl_duration;
    let labels = py_labels
        .into_iter()
        .map(|py_label| {
            end = end.max(py_label.range.1);
            layout.create_label(
                LabelInfo {
                    text: Some(py_label.label_desc),
                    color: Color::all(1.0, (177, 137, 198, 255), (177, 137, 198, 190)),
                    range: py_label.range,
                },
                None,
            )
        })
        .collect();

    Some(layout.create_group(
        LabelInfo {
            text: Some("sub-timeline".into()),
            color: Color::brush((177, 137, 198, 255)),
            range: (0.0, end),
        },
        labels,
        false,
        false,
        Color::default(),
    ))
}

fn parse_debug_labels(
    layout: &mut LabelLayout,
    tl_duration: f32,
    py_labels: Vec<DebugTimeLabel>,
) -> Option<LabelId> {
    if py_labels.is_empty() {
        return None;
    }

    let debug_labels = py_labels.into_iter().map(|py_label| {
        let visibility_labels = py_label
            .visibility
            .chunks(2)
            .map(|chunk| {
                (
                    *chunk.first().unwrap(),
                    *chunk.get(1).unwrap_or(&(tl_duration + 1.0)),
                )
            })
            .map(|chunk| {
                layout.create_label(
                    LabelInfo {
                        text: None,
                        color: Color::brush((255, 255, 128, 200)),
                        range: chunk,
                    },
                    Some(1),
                )
            })
            .collect();

        let visibility_group = layout.create_group(
            LabelInfo {
                text: None,
                color: Color::default(),
                range: (0.0, tl_duration),
            },
            visibility_labels,
            false,
            false,
            Color::default(),
        );

        let colors = [
            (251, 180, 174),
            (179, 205, 227),
            (204, 235, 197),
            (222, 203, 228),
            (254, 217, 166),
            (255, 255, 204),
            (229, 216, 189),
            (253, 218, 236),
            (242, 242, 242),
        ]
        .map(|(r, g, b)| Color::brush((r, g, b, 255)));

        let mut iter = colors.into_iter().cycle();
        let mut cache = HashMap::new();
        let mut get_color = |key| *cache.entry(key).or_insert_with(|| iter.next().unwrap());

        let mut anim_labels = Vec::new();
        let mut chunks = py_label.chunks.into_iter().peekable();
        while let Some(chunk) = chunks.next() {
            let end = chunks
                .peek()
                .map(|chunk| chunk.start)
                .unwrap_or(tl_duration + 1.0);
            let range = (chunk.start, end);

            for anim in chunk.list {
                anim_labels.push(layout.create_label(
                    LabelInfo {
                        text: Some(anim.1),
                        color: get_color(anim.0),
                        range,
                    },
                    None,
                ));
            }
        }

        let anim_group = layout.create_group(
            LabelInfo {
                text: None,
                color: Color::default(),
                range: (0.0, tl_duration),
            },
            anim_labels,
            false,
            false,
            Color::default(),
        );

        layout.create_group(
            LabelInfo {
                text: Some(py_label.item_repr),
                color: Color::brush((170, 148, 132, 255)),
                range: (0.0, tl_duration),
            },
            vec![visibility_group, anim_group],
            false,
            true,
            Color::all(3.0, (41, 171, 202, 255), (41, 171, 202, 40)),
        )
    });
    let debug_labels = debug_labels.collect();

    Some(layout.create_group(
        LabelInfo {
            text: Some("debug".into()),
            color: Color::brush((170, 148, 132, 255)),
            range: (0.0, tl_duration),
        },
        debug_labels,
        false,
        true,
        Color::all(3.0, (41, 171, 202, 255), (41, 171, 202, 40)),
    ))
}

fn parse_audio_labels(
    layout: &mut LabelLayout,
    tl_duration: f32,
    py_labels: Vec<AudioTimeLabel>,
) -> Option<LabelId> {
    if py_labels.is_empty() {
        return None;
    }

    let mut end = tl_duration;
    let labels: Vec<_> = py_labels
        .into_iter()
        .map(|py_label| {
            end = end.max(py_label.range.1);
            layout.create_label(
                LabelInfo {
                    text: Some(py_label.file_name),
                    color: Color::all(1.0, (85, 193, 167, 255), (85, 193, 167, 160)),
                    range: py_label.range,
                },
                None,
            )
        })
        .collect();

    Some(layout.create_group(
        LabelInfo {
            text: Some("audio".into()),
            color: Color::brush((85, 193, 167, 255)),
            range: (0.0, end),
        },
        labels,
        false,
        false,
        Color::default(),
    ))
}

fn parse_anim_labels(
    layout: &mut LabelLayout,
    tl_duration: f32,
    py_labels: Vec<AnimTimeLabel>,
) -> LabelId {
    fn make_label_from_anim(
        layout: &mut LabelLayout,
        tl_duration: f32,
        anim: AnimTimeLabel,
        header: bool,
    ) -> Option<(LabelId, (f32, f32))> {
        match anim {
            AnimTimeLabel::AnimGroup(info, sub_anims) => {
                let children_and_ranges: Vec<_> = sub_anims
                    .into_iter()
                    .filter_map(|sub| make_label_from_anim(layout, tl_duration, sub, true))
                    .collect();

                if children_and_ranges.is_empty() {
                    return None;
                }

                let children = children_and_ranges.iter().map(|(id, _)| *id).collect();
                let range_start = children_and_ranges
                    .iter()
                    .map(|(_, range)| range.0)
                    .fold(f32::INFINITY, f32::min);
                let range_end = children_and_ranges
                    .iter()
                    .map(|(_, range)| range.1)
                    .fold(f32::NEG_INFINITY, f32::max);

                let id = layout.create_group(
                    LabelInfo {
                        text: Some(info.name),
                        color: Color::brush(Color::to_rgba(info.color_rgb)),
                        range: (range_start, range_end),
                    },
                    children,
                    false,
                    header,
                    Color::all(3.0, (41, 171, 202, 255), (41, 171, 202, 40)),
                );
                Some((id, (range_start, range_end)))
            }
            AnimTimeLabel::Animation(info, at, end) => {
                let range = (at, end.unwrap_or(tl_duration));
                let id = layout.create_label(
                    LabelInfo {
                        text: Some(info.name),
                        color: Color::brush(Color::to_rgba(info.color_rgb)),
                        range,
                    },
                    None,
                );
                Some((id, range))
            }
        }
    }

    let labels = py_labels
        .into_iter()
        .filter_map(|anim| make_label_from_anim(layout, tl_duration, anim, true))
        .map(|(id, _)| id)
        .collect();

    layout.create_group(
        LabelInfo {
            text: None,
            color: Color::default(),
            range: (0.0, tl_duration),
        },
        labels,
        false,
        false,
        Color::default(),
    )
}
