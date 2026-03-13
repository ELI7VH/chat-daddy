//! TTF font rendering via fontdue — loads configurable font from system fonts.

use fontdue::{Font, FontSettings};
use std::collections::HashMap;
use std::fs;

/// Pre-rasterized glyph: coverage bitmap + metrics
pub struct Glyph {
    pub bitmap: Vec<u8>,   // alpha coverage, row-major
    pub width: usize,
    pub height: usize,
    pub x_off: i32,        // horizontal bearing
    pub y_off: i32,        // vertical bearing (from baseline, positive = up)
}

pub struct FontAtlas {
    font: Font,
    pub size: f32,
    pub advance: i32,      // monospace advance width in pixels
    pub line_height: i32,  // line height in pixels
    pub ascent: i32,       // pixels above baseline
    cache: HashMap<char, Glyph>,
}

impl FontAtlas {
    pub fn with_font(size: f32, font_name: &str, weight: u16) -> Self {
        let font_bytes = find_font(font_name, weight).unwrap_or_else(|| {
            panic!(
                "No monospace font found (tried {} and platform fallbacks). \
                 Install Fira Code or edit ~/.chat-daddy/config.json to set \
                 \"font\" to an installed monospace font.",
                font_name
            );
        });
        let font = Font::from_bytes(font_bytes, FontSettings::default())
            .expect("failed to parse TTF");

        // compute metrics at this size
        let metrics = font.horizontal_line_metrics(size).unwrap();
        let ascent = metrics.ascent.ceil() as i32;
        let descent = (-metrics.descent).ceil() as i32;
        let line_height = ascent + descent + 2; // +2px breathing room

        // advance: rasterize 'M' to get monospace cell width
        let (m_metrics, _) = font.rasterize('M', size);
        let advance = m_metrics.advance_width.ceil() as i32;

        FontAtlas {
            font,
            size,
            advance,
            line_height,
            ascent,
            cache: HashMap::new(),
        }
    }

    pub fn glyph(&mut self, ch: char) -> &Glyph {
        if !self.cache.contains_key(&ch) {
            let (metrics, bitmap) = self.font.rasterize(ch, self.size);
            let g = Glyph {
                bitmap,
                width: metrics.width,
                height: metrics.height,
                x_off: metrics.xmin,
                y_off: metrics.ymin,
            };
            self.cache.insert(ch, g);
        }
        self.cache.get(&ch).unwrap()
    }
}

/// Map weight number to TTF filename suffix
fn weight_suffix(weight: u16) -> &'static str {
    match weight {
        0..=250 => "Thin",
        251..=350 => "Light",
        351..=450 => "Regular",
        451..=550 => "Medium",
        551..=650 => "SemiBold",
        651..=750 => "Bold",
        _ => "Regular",
    }
}

/// Ordered list of weight suffixes to try, starting from the requested weight
fn weight_fallbacks(weight: u16) -> Vec<&'static str> {
    let primary = weight_suffix(weight);
    let all = ["Light", "Regular", "Retina", "Medium", "SemiBold", "Bold", "Thin"];
    let mut out = vec![primary];
    for &w in &all {
        if w != primary {
            out.push(w);
        }
    }
    out
}

/// Platform-specific font directories
fn font_dirs() -> Vec<String> {
    let mut dirs = Vec::new();
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();

    #[cfg(target_os = "macos")]
    {
        dirs.push(format!("{}/Library/Fonts", home));
        dirs.push("/Library/Fonts".to_string());
        dirs.push("/System/Library/Fonts".to_string());
        dirs.push("/System/Library/Fonts/Supplemental".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        dirs.push(format!("{}/.local/share/fonts", home));
        dirs.push(format!("{}/.fonts", home));
        dirs.push("/usr/share/fonts/truetype".to_string());
        dirs.push("/usr/local/share/fonts".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        dirs.push("C:/Windows/Fonts".to_string());
        dirs.push(format!(
            "{}/AppData/Local/Microsoft/Windows/Fonts",
            home
        ));
    }

    dirs
}

/// Try to load a named font from system font dirs with weight fallbacks
fn try_load_font(font_name: &str, weight: u16) -> Option<Vec<u8>> {
    let file_base: String = font_name.split_whitespace().collect();
    let dirs = font_dirs();
    let suffixes = weight_fallbacks(weight);

    // try each weight suffix
    for suffix in &suffixes {
        let filename = format!("{}-{}.ttf", file_base, suffix);
        for dir in &dirs {
            let path = format!("{}/{}", dir, filename);
            if let Ok(data) = fs::read(&path) {
                return Some(data);
            }
        }
    }

    // try bare name (e.g. "FiraCode.ttf")
    let bare = format!("{}.ttf", file_base);
    for dir in &dirs {
        let path = format!("{}/{}", dir, bare);
        if let Ok(data) = fs::read(&path) {
            return Some(data);
        }
    }

    None
}

/// Common monospace fonts to try as fallbacks, per platform
fn fallback_fonts() -> Vec<(&'static str, u16)> {
    let mut fonts = Vec::new();

    #[cfg(target_os = "macos")]
    {
        fonts.push(("SF Mono", 400));
        fonts.push(("Menlo", 400));
        fonts.push(("Monaco", 400));
    }

    #[cfg(target_os = "linux")]
    {
        fonts.push(("DejaVu Sans Mono", 400));
        fonts.push(("Liberation Mono", 400));
        fonts.push(("Ubuntu Mono", 400));
    }

    #[cfg(target_os = "windows")]
    {
        fonts.push(("Consolas", 400));
        fonts.push(("Courier New", 400));
    }

    fonts
}

fn find_font(font_name: &str, weight: u16) -> Option<Vec<u8>> {
    // try the configured font first
    if let Some(data) = try_load_font(font_name, weight) {
        return Some(data);
    }

    // fall back to platform-default monospace fonts
    eprintln!(
        "warning: {} (weight {}) not found — trying fallback fonts",
        font_name, weight
    );
    for (name, w) in fallback_fonts() {
        if let Some(data) = try_load_font(name, w) {
            eprintln!("  using fallback font: {}", name);
            return Some(data);
        }
    }

    None
}
