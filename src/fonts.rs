//! The interface font (Inter SemiBold). The renderer draws it and the layout
//! code measures it, so a line placed beside something lands where it's drawn.

use std::sync::OnceLock;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};

pub static SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.ttf");

fn font() -> &'static FontRef<'static> {
    static FONT: OnceLock<FontRef<'static>> = OnceLock::new();
    FONT.get_or_init(|| FontRef::try_from_slice(SEMIBOLD).expect("Inter font"))
}

/// The advance width of one line of `text` at `px`, in pixels.
pub fn line_width(text: &str, px: f32) -> f32 {
    let f = font().as_scaled(PxScale::from(px));
    let mut w = 0.0;
    let mut prev = None;
    for ch in text.chars() {
        let id = f.glyph_id(ch);
        if let Some(p) = prev {
            w += f.kern(p, id);
        }
        w += f.h_advance(id);
        prev = Some(id);
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_grow_with_text_and_size() {
        let short = line_width("Made", 40.0);
        let long = line_width("Made in DaisyCap", 40.0);
        assert!(short > 0.0 && long > short * 2.0, "{short} {long}");
        assert!((line_width("Made in DaisyCap", 80.0) - long * 2.0).abs() < 1.0, "scales with size");
        assert_eq!(line_width("", 40.0), 0.0);
    }
}
