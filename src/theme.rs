//! Plasma theme.
//!
//! Taken from <https://github.com/scruffykat/egui-Themes>.

use eframe::egui::{
    epaint::Shadow,
    style::{
        HandleShape, Interaction, Margin, ScrollStyle, Selection, Spacing, WidgetVisuals, Widgets,
    },
    Color32, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Vec2, Visuals,
};

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
            item_spacing: Vec2 { x: 6.0, y: 6.0 },
            window_margin: Margin {
                left: 8.0,
                right: 8.0,
                top: 8.0,
                bottom: 8.0,
            },
            button_padding: Vec2 { x: 16.0, y: 5.0 },
            menu_margin: Margin {
                left: 6.0,
                right: 6.0,
                top: 6.0,
                bottom: 6.0,
            },
            indent: 18.0,
            interact_size: Vec2 { x: 40.0, y: 18.0 },
            slider_width: 100.0,
            combo_width: 100.0,
            text_edit_width: 280.0,
            icon_width: 16.0,
            icon_width_inner: 8.0,
            icon_spacing: 4.0,
            tooltip_width: 600.0,
            indent_ends_with_horizontal_line: false,
            combo_height: 200.0,
            scroll: ScrollStyle::floating(),
        },
        interaction: Interaction {
            resize_grab_radius_side: 5.0,
            resize_grab_radius_corner: 10.0,
            show_tooltips_only_when_still: true,
            tooltip_delay: 0.0,
        },
        visuals: Visuals {
            dark_mode: true,
            override_text_color: None,
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    bg_fill: Color32::from_rgba_premultiplied(53, 47, 68, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(53, 47, 68, 255),
                    bg_stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgba_premultiplied(92, 84, 112, 255),
                    },
                    rounding: Rounding {
                        nw: 2.0,
                        ne: 2.0,
                        sw: 2.0,
                        se: 2.0,
                    },
                    fg_stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgba_premultiplied(250, 240, 230, 255),
                    },
                    expansion: 0.0,
                },
                inactive: WidgetVisuals {
                    bg_fill: Color32::from_rgba_premultiplied(39, 37, 45, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(44, 43, 43, 255),
                    bg_stroke: Stroke {
                        width: 0.0,
                        color: Color32::from_rgba_premultiplied(0, 0, 0, 0),
                    },
                    rounding: Rounding {
                        nw: 2.0,
                        ne: 2.0,
                        sw: 2.0,
                        se: 2.0,
                    },
                    fg_stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgba_premultiplied(205, 205, 205, 255),
                    },
                    expansion: 0.0,
                },
                hovered: WidgetVisuals {
                    bg_fill: Color32::from_rgba_premultiplied(131, 132, 144, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(156, 154, 205, 255),
                    bg_stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgba_premultiplied(255, 255, 255, 255),
                    },
                    rounding: Rounding {
                        nw: 3.0,
                        ne: 3.0,
                        sw: 3.0,
                        se: 3.0,
                    },
                    fg_stroke: Stroke {
                        width: 1.5,
                        color: Color32::from_rgba_premultiplied(245, 245, 245, 255),
                    },
                    expansion: 1.0,
                },
                active: WidgetVisuals {
                    bg_fill: Color32::from_rgba_premultiplied(70, 70, 70, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(70, 70, 70, 255),
                    bg_stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgba_premultiplied(255, 255, 255, 255),
                    },
                    rounding: Rounding {
                        nw: 2.0,
                        ne: 2.0,
                        sw: 2.0,
                        se: 2.0,
                    },
                    fg_stroke: Stroke {
                        width: 2.0,
                        color: Color32::from_rgba_premultiplied(255, 255, 255, 255),
                    },
                    expansion: 1.0,
                },
                open: WidgetVisuals {
                    bg_fill: Color32::from_rgba_premultiplied(53, 47, 68, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(53, 47, 68, 255),
                    bg_stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgba_premultiplied(119, 119, 119, 255),
                    },
                    rounding: Rounding {
                        nw: 2.0,
                        ne: 2.0,
                        sw: 2.0,
                        se: 2.0,
                    },
                    fg_stroke: Stroke {
                        width: 1.0,
                        color: Color32::from_rgba_premultiplied(229, 229, 229, 255),
                    },
                    expansion: 0.0,
                },
            },
            selection: Selection {
                bg_fill: Color32::from_rgba_premultiplied(139, 127, 218, 255),
                stroke: Stroke {
                    width: 1.0,
                    color: Color32::from_rgba_premultiplied(255, 255, 255, 255),
                },
            },
            hyperlink_color: Color32::from_rgba_premultiplied(156, 154, 205, 255),
            faint_bg_color: Color32::from_rgba_premultiplied(2, 2, 2, 0),
            extreme_bg_color: Color32::from_rgba_premultiplied(26, 25, 25, 255),
            code_bg_color: Color32::from_rgba_premultiplied(64, 64, 64, 255),
            warn_fg_color: Color32::from_rgba_premultiplied(255, 143, 0, 255),
            error_fg_color: Color32::from_rgba_premultiplied(255, 0, 0, 255),
            window_rounding: Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: 0.0,
                se: 0.0,
            },
            window_shadow: Shadow {
                extrusion: 32.0,
                color: Color32::from_rgba_premultiplied(0, 0, 0, 96),
            },
            window_fill: Color32::from_rgba_premultiplied(30, 30, 30, 255),
            window_stroke: Stroke {
                width: 1.0,
                color: Color32::from_rgba_premultiplied(38, 38, 38, 255),
            },
            menu_rounding: Rounding {
                nw: 6.0,
                ne: 6.0,
                sw: 6.0,
                se: 6.0,
            },
            panel_fill: Color32::from_rgba_premultiplied(27, 27, 27, 255),
            popup_shadow: Shadow {
                extrusion: 16.0,
                color: Color32::from_rgba_premultiplied(0, 0, 0, 96),
            },
            resize_corner_size: 12.0,
            text_cursor: Stroke {
                width: 2.0,
                color: Color32::from_rgba_premultiplied(255, 255, 255, 255),
            },
            interact_cursor: None,
            image_loading_spinners: true,
            text_cursor_preview: false,
            clip_rect_margin: 3.0,
            button_frame: true,
            collapsing_header_frame: false,
            indent_has_left_vline: true,
            striped: false,
            slider_trailing_fill: false,
            handle_shape: HandleShape::Circle,
        },
        animation_time: 0.083,
        explanation_tooltips: false,
        ..Default::default()
    }
}
