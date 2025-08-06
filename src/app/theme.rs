use std::{collections::HashMap, string};
use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui::style::{NumericColorSpace, Selection, TextCursorStyle, WidgetVisuals, Widgets};
use eframe::egui::{vec2, Shadow, Stroke};
use eframe::epaint::tessellator::path;
use eframe::epaint::AlphaFromCoverage;
use eframe::{egui::{Color32, Context, FontDefinitions, Visuals}};
use palette::{oklch::Oklch, oklab::Oklab, IntoColor, Srgb as LinSrgb};
use regex::Regex;

#[derive(Clone)]
pub struct SkeletonTheme {
    pub name: String,
    pub colors: HashMap<String, Color32>,
    pub fonts: FontDefinitions,
    pub visuals: Visuals,
}

impl SkeletonTheme {
    pub fn from_css_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let name= path.as_ref().file_name().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(path)?;
        Self::from_css_str(&content, &name)
    }

    pub fn get_visuals(&self) -> &Visuals {
        &self.visuals
    }

    pub fn from_css_str(css: &str, name: &str) -> anyhow::Result<Self> {
        let mut colors = HashMap::new();
        let var_re = Regex::new(r"--([a-zA-Z0-9\-]+):\s*([^;]+);")?;

        for cap in var_re.captures_iter(css) {
            let key = cap[1].to_string();
            let value = cap[2].trim();

            if value.starts_with("oklch") {
                if let Some(color) = parse_oklch(value) {
                    colors.insert(key, color);
                }
            }
        }


        // Map CSS variables to egui Visuals fields, falling back to egui defaults if missing
        let vis = Visuals {
            dark_mode: true,
            text_alpha_from_coverage: AlphaFromCoverage::default(),
            override_text_color: colors.get("color-secondary-500").cloned(),
            weak_text_alpha: 0.5,
            weak_text_color: colors.get("color-surface-300").cloned(),
            widgets: Widgets {
                noninteractive: WidgetVisuals{
                    bg_fill: colors.get("color-surface-100").cloned().unwrap_or(Color32::from_gray(30)),
                    bg_stroke: Stroke::new(1.0, Color32::BLACK),
                    corner_radius: 4.0.into(),
                    expansion: 0.0,
                    fg_stroke: Stroke::new(1.0, Color32::BLACK),
                    weak_bg_fill: colors.get("color-surface-200").cloned().unwrap_or(Color32::from_gray(60)),

                },
                inactive: WidgetVisuals{
                    bg_fill: colors.get("color-surface-200").cloned().unwrap_or(Color32::from_gray(60)),
                    bg_stroke: Stroke::new(1.0, Color32::BLACK),
                    corner_radius: 4.0.into(),
                    expansion: 0.0,
                    fg_stroke: Stroke::new(1.0, Color32::BLACK),
                    weak_bg_fill: colors.get("color-secondary-300").cloned().unwrap_or(Color32::from_rgb(100, 100, 200)),
                },
                hovered: WidgetVisuals{
                    bg_fill: colors.get("color-secondary-300").cloned().unwrap_or(Color32::from_rgb(100, 100, 200)),
                    bg_stroke: Stroke::new(1.0, Color32::BLACK),
                    corner_radius: 4.0.into(),
                    expansion: 0.0,
                    fg_stroke: Stroke::new(1.0, Color32::BLACK),
                    weak_bg_fill: colors.get("color-secondary-500").cloned().unwrap_or(Color32::from_rgb(120, 120, 220)),
                },
                active: WidgetVisuals{
                    bg_fill: colors.get("color-secondary-500").cloned().unwrap_or(Color32::from_rgb(120, 120, 220)),
                    bg_stroke: Stroke::new(1.0, Color32::BLACK),
                    corner_radius: 4.0.into(),
                    expansion: 0.0,
                    fg_stroke: Stroke::new(1.0, Color32::BLACK),
                    weak_bg_fill: colors.get("color-secondary-700").cloned().unwrap_or(Color32::from_rgb(140, 140, 240)),
                },
                open: WidgetVisuals{
                    bg_fill: colors.get("color-secondary-700").cloned().unwrap_or(Color32::from_rgb(140, 140, 240)),
                    bg_stroke: Stroke::new(1.0, Color32::BLACK),
                    corner_radius: 4.0.into(),
                    expansion: 0.0,
                    fg_stroke: Stroke::new(1.0, Color32::BLACK),
                    weak_bg_fill: colors.get("color-secondary-900").cloned().unwrap_or(Color32::from_rgb(160, 160, 255)),
                },
                ..Default::default()
            },
            selection: Selection {
                bg_fill: colors.get("color-secondary-500").cloned().unwrap_or(Color32::from_rgb(120, 120, 220)),
                stroke: Stroke{
                    color: colors.get("color-secondary-700").cloned().unwrap_or(Color32::from_rgb(140, 140, 240)),
                    width: 1.0,
                    
                },
            },

            hyperlink_color: colors.get("color-link").cloned().unwrap_or(Color32::from_rgb(80, 160, 240)),
            faint_bg_color: colors.get("color-surface-100").cloned().unwrap_or(Color32::from_gray(20)),
            extreme_bg_color: colors.get("color-surface-900").cloned().unwrap_or(Color32::BLACK),
            text_edit_bg_color: colors.get("color-surface-200").cloned(),
            code_bg_color: colors.get("color-surface-300").cloned().unwrap_or(Color32::from_gray(60)),
            warn_fg_color: colors.get("color-warning").cloned().unwrap_or(Color32::YELLOW),
            error_fg_color: colors.get("color-error").cloned().unwrap_or(Color32::RED),


            window_corner_radius: 6.0.into(),
            window_shadow: Shadow{
                color: Color32::from_black_alpha(128),
                offset: [0, 2],
                blur: 8,
                spread: 0,
            },
            window_fill: colors.get("color-surface-800").cloned().unwrap_or(Color32::from_gray(30)),
            window_stroke: eframe::epaint::Stroke::new(1.0, Color32::BLACK),
            window_highlight_topmost: true,


            menu_corner_radius: 4.0.into(),
            panel_fill: colors.get("color-surface-700").cloned().unwrap_or(Color32::from_gray(40)),
            popup_shadow: Shadow{
                color: Color32::from_black_alpha(128),
                offset: [0, 2],
                blur: 8,
                spread: 0,
            },
            resize_corner_size: 12.0,
            text_cursor: TextCursorStyle{
                ..Default::default()
            },

            clip_rect_margin: 3.0,
            button_frame: true,
            collapsing_header_frame: true,
            indent_has_left_vline: true,
            striped: true,
            slider_trailing_fill: true,

            handle_shape: eframe::egui::style::HandleShape::Circle,
            interact_cursor: Some(eframe::egui::CursorIcon::PointingHand),
            image_loading_spinners: true,

            numeric_color_space: NumericColorSpace::Linear,
            disabled_alpha: 0.6,
        };
        // You can expand this to more mappings

        let fonts = FontDefinitions::default(); // could customize if desired

        Ok(Self {
            name: name.to_string(),
            colors,
            fonts,
            visuals: vis,
        })
    }

    pub fn apply(&self, ctx: &Context) {
        ctx.set_fonts(self.fonts.clone());
        ctx.set_visuals(self.visuals.clone());
    }
}

/// Manual OKLCH to sRGB conversion function
/// Takes OKLCH values: L (0.0-1.0), C (0.0-0.4 typically), H (0.0-360.0 degrees)
pub fn oklch_to_srgb(l: f32, c: f32, h_deg: f32) -> Color32 {
    // Convert hue from degrees to radians
    let h_rad = h_deg.to_radians();
    
    // OKLCH to OKLab conversion
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();
    
    // OKLab to Linear RGB conversion (using OK Lab matrices)
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    
    let l_cubed = l_.powi(3);
    let m_cubed = m_.powi(3);
    let s_cubed = s_.powi(3);
    
    // LMS to Linear RGB
    let r_linear = 4.0767416621 * l_cubed - 3.3077115913 * m_cubed + 0.2309699292 * s_cubed;
    let g_linear = -1.2684380046 * l_cubed + 2.6097574011 * m_cubed - 0.3413193965 * s_cubed;
    let b_linear = -0.0041960863 * l_cubed - 0.7034186147 * m_cubed + 1.7076147010 * s_cubed;
    
    // Linear RGB to sRGB gamma correction
    let linear_to_srgb = |x: f32| -> f32 {
        if x <= 0.0031308 {
            12.92 * x
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        }
    };
    
    let r_srgb = linear_to_srgb(r_linear);
    let g_srgb = linear_to_srgb(g_linear);
    let b_srgb = linear_to_srgb(b_linear);
    
    // Clamp to 0-1 and convert to 0-255
    let r = (r_srgb.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (g_srgb.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (b_srgb.clamp(0.0, 1.0) * 255.0).round() as u8;
    
    Color32::from_rgb(r, g, b)
}


fn parse_oklch(s: &str) -> Option<Color32> {
    let inner = s.strip_prefix("oklch(")?.strip_suffix(")")?;
    let parts: Vec<&str> = inner
        .split_whitespace()
        .map(|x| x.trim_end_matches('%'))
        .collect();

    if parts.len() < 3 {
        return None;
    }

    let l: f32 = parts[0].parse().ok()?;
    let l_normalized = l / 100.0; // Convert percentage to 0-1
    let c: f32 = parts[1].parse().ok()?;
    let h: f32 = parts[2].trim_end_matches("deg").parse().ok()?;

    println!("Parsing OKLCH: L={}, C={}, H={}", l_normalized, c, h);
    
    // Use our manual conversion
    let manual_color = oklch_to_srgb(l_normalized, c, h);
    println!("Manual conversion result: {:?}", manual_color);
    
    // For comparison, also try the palette library conversion
    let oklch = Oklch::new(l_normalized, c, h.to_radians());
    let oklab: Oklab = oklch.into_color();
    let lrgb: LinSrgb = oklab.into_color();
    let palette_color = Color32::from_rgb(
        (lrgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
        (lrgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
        (lrgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
    );
    println!("Palette library result: {:?}", palette_color);
    
    Some(manual_color)
}

