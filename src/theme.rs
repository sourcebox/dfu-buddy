//! Plasma theme.
//!
//! Taken from <https://github.com/scruffykat/egui-Themes>.

use eframe::egui::{
    epaint::Shadow,
    style::{Interaction, Margin, Selection, Spacing, WidgetVisuals, Widgets},
    Color32, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Visuals,
};
use eframe::emath::vec2;

pub fn style() -> Style {
    Style {
        text_styles: [
            (
                TextStyle::Small,
                FontId::new(11.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Heading,
                FontId::new(18.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(14.0, FontFamily::Monospace),
            ),
        ]
        .into(),
        spacing: Spacing {
            item_spacing: vec2(6.0, 6.0),
            window_margin: Margin::same(8.0),
            button_padding: vec2(16.0, 5.0),
            icon_width: 16.0,
            ..Default::default()
        },
        interaction: Interaction {
            ..Default::default()
        },
        visuals: Visuals {
            dark_mode: true,
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    bg_fill: Color32::from_rgb(53, 47, 68),
                    weak_bg_fill: Color32::from_rgb(53, 47, 68),
                    bg_stroke: Stroke::new(1.0, Color32::from_rgb(92, 84, 112)),
                    rounding: Rounding::same(2.0),
                    fg_stroke: Stroke::new(1.0, Color32::from_rgb(250, 240, 230)),
                    expansion: 0.0,
                },
                inactive: WidgetVisuals {
                    bg_fill: Color32::from_rgb(39, 37, 45),
                    weak_bg_fill: Color32::from_rgb(44, 43, 43),
                    bg_stroke: Stroke::new(0.0, Color32::from_rgba_premultiplied(0, 0, 0, 0)),
                    rounding: Rounding::same(2.0),
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(205)),
                    expansion: 0.0,
                },
                hovered: WidgetVisuals {
                    bg_fill: Color32::from_rgb(131, 132, 144),
                    weak_bg_fill: Color32::from_rgb(156, 154, 205),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(255)),
                    rounding: Rounding::same(3.0),
                    fg_stroke: Stroke::new(1.5, Color32::from_gray(245)),
                    expansion: 1.0,
                },
                active: WidgetVisuals {
                    bg_fill: Color32::from_gray(70),
                    weak_bg_fill: Color32::from_gray(70),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(255)),
                    rounding: Rounding::same(2.0),
                    fg_stroke: Stroke::new(2.0, Color32::from_gray(255)),
                    expansion: 1.0,
                },
                open: WidgetVisuals {
                    bg_fill: Color32::from_rgb(53, 47, 68),
                    weak_bg_fill: Color32::from_rgb(53, 47, 68),
                    bg_stroke: Stroke::new(1.0, Color32::from_gray(119)),
                    rounding: Rounding::same(2.0),
                    fg_stroke: Stroke::new(1.0, Color32::from_gray(229)),
                    expansion: 0.0,
                },
            },
            selection: Selection {
                bg_fill: Color32::from_rgb(139, 127, 218),
                stroke: Stroke::new(1.0, Color32::from_gray(255)),
            },
            hyperlink_color: Color32::from_rgb(156, 154, 205),
            faint_bg_color: Color32::from_rgba_premultiplied(2, 2, 2, 0),
            extreme_bg_color: Color32::from_rgb(26, 25, 25),
            window_rounding: Rounding::same(0.0),
            window_shadow: Shadow {
                extrusion: 32.0,
                color: Color32::from_rgba_premultiplied(0, 0, 0, 96),
            },
            window_fill: Color32::from_gray(30),
            window_stroke: Stroke::new(1.0, Color32::from_gray(38)),
            panel_fill: Color32::from_gray(27),
            popup_shadow: Shadow {
                extrusion: 16.0,
                color: Color32::from_gray(0),
            },
            text_cursor: Stroke::new(2.0, Color32::from_gray(255)),
            ..Default::default()
        },
        ..Default::default()
    }
}
