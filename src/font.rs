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
                "{} (weight {}) not found — install it to system fonts",
                font_name, weight
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

fn find_font(font_name: &str, weight: u16) -> Option<Vec<u8>> {
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    // normalize font name: "Fira Code" -> "FiraCode"
    let file_base: String = font_name.split_whitespace().collect();

    let suffixes = weight_fallbacks(weight);

    for suffix in &suffixes {
        let filename = format!("{}-{}.ttf", file_base, suffix);
        let candidates = [
            format!("C:/Windows/Fonts/{}", filename),
            format!("{}/AppData/Local/Microsoft/Windows/Fonts/{}", home, filename),
        ];
        for path in &candidates {
            if let Ok(data) = fs::read(path) {
                return Some(data);
            }
        }
    }

    // last resort: try the bare name
    let bare = format!("{}.ttf", file_base);
    let bare_paths = [
        format!("C:/Windows/Fonts/{}", bare),
        format!("{}/AppData/Local/Microsoft/Windows/Fonts/{}", home, bare),
    ];
    for path in &bare_paths {
        if let Ok(data) = fs::read(path) {
            return Some(data);
        }
    }

    None
}
