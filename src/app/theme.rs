use std::collections::HashMap;
use std::fs;
use std::path::Path;

use eframe::{egui::{Color32, Context, FontDefinitions, Visuals}};
use palette::{encoding::Srgb, oklch::Oklch, FromColor, IntoColor, Srgb as LinSrgb};
use regex::Regex;

pub struct SkeletonTheme {
    pub colors: HashMap<String, Color32>,
    pub fonts: FontDefinitions,
    pub visuals: Visuals,
}

impl SkeletonTheme {
    pub fn from_css_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::from_css_str(&content)
    }

    pub fn from_css_str(css: &str) -> anyhow::Result<Self> {
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

        // Apply a few known defaults to visuals
        let mut visuals = Visuals::light();
        if let Some(bg) = colors.get("color-surface-50") {
            visuals.widgets.inactive.bg_fill = *bg;
            visuals.extreme_bg_color = *bg;
            visuals.panel_fill = *bg;
        }
        if let Some(fg) = colors.get("color-surface-950") {
            visuals.override_text_color = Some(*fg);
        }

        // You can expand this to more mappings

        let fonts = FontDefinitions::default(); // could customize if desired

        Ok(Self {
            colors,
            fonts,
            visuals,
        })
    }

    pub fn apply(&self, ctx: &Context) {
        ctx.set_fonts(self.fonts.clone());
        ctx.set_visuals(self.visuals.clone());
    }
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
    let l2 = l / 100.0;
    let c: f32 = parts[1].parse().ok()?;
    let h: f32 = parts[2].trim_end_matches("deg").parse().ok()?;

    let oklch = Oklch::new(l2, c, h.to_radians());
    let lin_rgb: LinSrgb = LinSrgb::from_color(oklch);
    

    Some(Color32::from_rgb(
        (lin_rgb.red).clamp(0.0, 255.0) as u8,
        (lin_rgb.green).clamp(0.0, 255.0) as u8,
        (lin_rgb.blue).clamp(0.0, 255.0) as u8,
    ))
}

