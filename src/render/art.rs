//! Built-in artwork generated in code: wallpapers and vector cursors.
//! (No third-party images are bundled.)

use super::Image;
use crate::engine::geom::{Vec2, v2};

/// Built-in wallpapers: (id, display name, base colour, blobs).
type Blob = ([u8; 3], f32, f32, f32);
pub const WALLPAPERS: [(&str, &str, [u8; 3], [Blob; 4]); 10] = [
    ("aurora", "Aurora", [16, 22, 58], [([60, 214, 190], 0.15, 0.85, 0.7), ([118, 76, 255], 0.85, 0.15, 0.75), ([40, 120, 255], 0.55, 0.65, 0.6), ([255, 120, 200], 0.95, 0.95, 0.45)]),
    ("sunset", "Sunset", [60, 18, 60], [([255, 140, 60], 0.2, 0.9, 0.75), ([255, 70, 120], 0.8, 0.6, 0.7), ([255, 210, 120], 0.5, 1.1, 0.6), ([120, 50, 160], 0.9, 0.05, 0.6)]),
    ("ocean", "Ocean", [6, 30, 60], [([0, 160, 255], 0.1, 0.2, 0.75), ([0, 220, 200], 0.9, 0.9, 0.7), ([30, 90, 200], 0.6, 0.4, 0.6), ([120, 200, 255], 0.3, 0.95, 0.45)]),
    ("mint", "Mint", [210, 245, 230], [([120, 230, 190], 0.1, 0.9, 0.7), ([170, 220, 255], 0.9, 0.1, 0.7), ([250, 255, 220], 0.6, 0.6, 0.6), ([140, 255, 220], 0.95, 0.95, 0.5)]),
    ("grape", "Grape", [28, 10, 48], [([170, 60, 255], 0.2, 0.3, 0.7), ([255, 60, 180], 0.8, 0.8, 0.7), ([80, 40, 200], 0.9, 0.1, 0.6), ([255, 150, 230], 0.1, 0.95, 0.45)]),
    ("dawn", "Dawn", [250, 225, 215], [([255, 180, 160], 0.1, 0.1, 0.8), ([200, 190, 255], 0.9, 0.8, 0.8), ([255, 230, 180], 0.5, 0.95, 0.6), ([255, 200, 230], 0.95, 0.15, 0.55)]),
    ("ember", "Ember", [30, 8, 8], [([255, 90, 30], 0.15, 0.85, 0.7), ([255, 40, 60], 0.85, 0.25, 0.65), ([255, 180, 60], 0.55, 0.55, 0.5), ([120, 20, 40], 0.9, 0.95, 0.6)]),
    ("lagoon", "Lagoon", [10, 50, 50], [([30, 200, 160], 0.2, 0.2, 0.75), ([20, 120, 200], 0.85, 0.75, 0.7), ([160, 255, 210], 0.6, 0.15, 0.5), ([10, 80, 120], 0.1, 0.95, 0.6)]),
    ("graphite", "Graphite", [28, 29, 33], [([70, 72, 82], 0.2, 0.2, 0.8), ([45, 47, 55], 0.85, 0.85, 0.8), ([95, 98, 110], 0.7, 0.2, 0.5), ([20, 20, 24], 0.2, 0.95, 0.6)]),
    ("snow", "Snow", [236, 238, 243], [([255, 255, 255], 0.2, 0.2, 0.8), ([214, 220, 232], 0.85, 0.85, 0.8), ([245, 246, 250], 0.7, 0.2, 0.5), ([225, 230, 240], 0.15, 0.9, 0.6)]),
];

/// Generate a wallpaper: soft colour blobs over a base colour, crossed by
/// translucent "glass" ribbons with crisp edges and a bright rim (so the art
/// has detail a background blur visibly softens), lightly dithered against
/// banding.
pub fn wallpaper(id: &str, w: u32, h: u32) -> Image {
    use std::f32::consts::TAU;
    let index = WALLPAPERS.iter().position(|wp| wp.0 == id).unwrap_or(0);
    let (_, _, base, blobs) = &WALLPAPERS[index];
    // Light wallpapers get darker glass so it still reads.
    let light = base.iter().map(|&c| c as f32).sum::<f32>() > 3.0 * 160.0;
    let (target, glass) = if light { (0.0, 0.16) } else { (255.0, 0.16) };
    let phase = index as f32 * 0.37;
    // Each ribbon's centre line and half-thickness depend only on x.
    let ribbons: Vec<[(f32, f32); 3]> = (0..w)
        .map(|x| {
            let u = x as f32 / w as f32;
            std::array::from_fn(|k| {
                let p = phase + k as f32 * 1.9;
                let centre = 0.22 + 0.28 * k as f32 + 0.09 * (TAU * (0.55 * u + p)).sin() + 0.035 * (TAU * (1.6 * u + 1.3 * p)).sin();
                (centre, 0.055 + 0.025 * (TAU * (0.8 * u + 0.7 * p)).sin())
            })
        })
        .collect();
    let aspect = w as f32 / h as f32;
    let px = h as f32;
    let row = |y: u32, out: &mut [u8]| {
        let v = y as f32 / h as f32;
        let mut seed: u32 = 0x9e37_79b9 ^ y.wrapping_mul(0x85eb_ca6b);
        for (x, o) in out.chunks_exact_mut(4).enumerate() {
            let u = x as f32 / w as f32;
            let mut c = base.map(|c| c as f32);
            for &(col, bx, by, r) in blobs {
                let dx = (u - bx) * aspect;
                let dy = v - by;
                let wgt = (-(dx * dx + dy * dy) / (r * r) * 2.2).exp();
                for i in 0..3 {
                    c[i] += (col[i] as f32 - c[i]) * wgt;
                }
            }
            for &(centre, half) in &ribbons[x] {
                // Coverage of the band, anti-aliased over about a pixel.
                let inside = (0.5 - ((v - centre).abs() - half) * px).clamp(0.0, 1.0);
                // A rim a few pixels wide along the band's upper edge.
                let rim = (1.0 - ((v - (centre - half)) * px - 2.0).abs() / 2.5).clamp(0.0, 1.0);
                if inside > 0.0 || rim > 0.0 {
                    for ch in c.iter_mut() {
                        *ch += (target - *ch) * inside * glass;
                        *ch += (255.0 - *ch) * rim * 0.38;
                    }
                }
            }
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let n = (seed & 0xff) as f32 / 255.0 - 0.5;
            for i in 0..3 {
                o[i] = (c[i] + n * 1.5).clamp(0.0, 255.0) as u8;
            }
            o[3] = 255;
        }
    };
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let stride = (w * 4) as usize;
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).min(16);
    let rows_per = (h as usize).div_ceil(threads).max(1);
    std::thread::scope(|sc| {
        for (i, chunk) in rgba.chunks_mut(rows_per * stride.max(1)).enumerate() {
            let row = &row;
            sc.spawn(move || {
                for (j, line) in chunk.chunks_exact_mut(stride).enumerate() {
                    row((i * rows_per + j) as u32, line);
                }
            });
        }
    });
    Image { width: w, height: h, rgba }
}

fn seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f64 {
    let ab = b - a;
    let t = (((p - a).x * ab.x + (p - a).y * ab.y) / (ab.x * ab.x + ab.y * ab.y)).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}

/// Signed distance to a polygon (negative inside).
fn poly_sd(p: Vec2, pts: &[Vec2]) -> f64 {
    let mut d = f64::MAX;
    let mut inside = false;
    let n = pts.len();
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        d = d.min(seg_dist(p, a, b));
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
    }
    if inside { -d } else { d }
}

/// Paint shapes (signed distance fn in shape units) as a cursor: dark fill,
/// white outline, soft drop shadow.
fn paint(units: Vec2, px_per_unit: f64, outline: f64, sd: impl Fn(Vec2) -> f64) -> Image {
    let pad = 4.0;
    let (w, h) = (((units.x + 2.0 * pad) * px_per_unit).ceil() as u32, ((units.y + 2.0 * pad) * px_per_unit).ceil() as u32);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let p = v2((x as f64 + 0.5) / px_per_unit - pad, (y as f64 + 0.5) / px_per_unit - pad);
            let aa = 0.75 / px_per_unit;
            let d = sd(p);
            let fill = ((-d) / aa + 0.5).clamp(0.0, 1.0);
            let ring = ((outline - d) / aa + 0.5).clamp(0.0, 1.0);
            let ds = sd(p - v2(0.0, 0.9));
            let shadow = (-(ds - outline).max(0.0).powi(2) / 2.5).exp() * 0.35;
            // Composite: shadow, white ring, black fill (premultiplied).
            let mut a = shadow;
            let mut c = 0.0;
            c = c * (1.0 - ring) + ring;
            a = a * (1.0 - ring) + ring;
            c *= 1.0 - fill;
            a = a * (1.0 - fill) + fill;
            let v = (c * 255.0) as u8;
            rgba.extend_from_slice(&[v, v, v, (a * 255.0) as u8]);
        }
    }
    Image { width: w, height: h, rgba }
}

/// A vector cursor image, its hotspot (normalised) and height relative to
/// the standard arrow.
pub fn vector_cursor(kind: Option<&str>) -> (Image, Vec2, f64) {
    const PX: f64 = 6.0;
    let pad = 4.0;
    match kind {
        Some("ibeam") => {
            let units = v2(9.0, 18.0);
            let img = paint(units, PX, 1.2, |p| {
                let bar = (p.x - 4.5).abs().max((p.y - 9.0).abs() - 8.0) - 0.7;
                let top = (p.x - 4.5).abs().max((p.y - 0.6).abs()) - 2.6 + 2.0;
                let bot = (p.x - 4.5).abs().max((p.y - 17.4).abs()) - 2.6 + 2.0;
                bar.min(top.max((p.y - 0.6).abs() - 0.6)).min(bot.max((p.y - 17.4).abs() - 0.6))
            });
            let hot = v2((4.5 + pad) / (units.x + 2.0 * pad), (9.0 + pad) / (units.y + 2.0 * pad));
            (img, hot, 1.0)
        }
        _ => {
            let pts = [
                v2(0.0, 0.0),
                v2(0.0, 16.8),
                v2(4.1, 12.9),
                v2(6.9, 19.4),
                v2(9.3, 18.4),
                v2(6.6, 12.0),
                v2(12.2, 12.0),
            ];
            let units = v2(13.0, 20.0);
            let img = paint(units, PX, 1.3, |p| poly_sd(p, &pts));
            let hot = v2(pad / (units.x + 2.0 * pad), pad / (units.y + 2.0 * pad));
            (img, hot, (units.y + 2.0 * pad) / 20.0)
        }
    }
}

/// A soft circle for touch-style recordings.
pub fn touch_cursor() -> (Image, Vec2, f64) {
    let s = 96u32;
    let mut rgba = Vec::with_capacity((s * s * 4) as usize);
    for y in 0..s {
        for x in 0..s {
            let d = ((x as f64 + 0.5 - 48.0).hypot(y as f64 + 0.5 - 48.0)) - 30.0;
            let fill = (0.5 - d).clamp(0.0, 1.0) * 0.45;
            let ring = (1.0 - (d + 2.0).abs() / 2.0).clamp(0.0, 1.0) * 0.9;
            let a = (fill + ring).min(1.0);
            let c = (0.55 * fill + ring).min(a);
            rgba.extend_from_slice(&[(c * 255.0) as u8, (c * 255.0) as u8, (c * 255.0) as u8, (a * 255.0) as u8]);
        }
    }
    (Image { width: s, height: s, rgba }, v2(0.5, 0.5), 1.4)
}

/// The hand's height in sprite units, so the drawn hand matches a standard
/// arrow's height whatever the sprite's padding is.
pub const HAND_UNITS: f64 = 20.0;

/// Signed distance to a rounded rect given its min corner and size.
fn round_rect(p: Vec2, min: Vec2, size: Vec2, r: f64) -> f64 {
    let (qx, qy) = ((p.x - min.x - size.x / 2.0).abs() - size.x / 2.0 + r, (p.y - min.y - size.y / 2.0).abs() - size.y / 2.0 + r);
    let (mx, my) = (qx.max(0.0), qy.max(0.0));
    (mx * mx + my * my).sqrt() + qx.max(qy).min(0.0) - r
}

/// The hand skin: a warm hand pointing up and to the left, fingertip at the
/// hotspot. The sprite ends at the wrist; the cuff and sleeve are the arm
/// layer's job (see `engine::scene`). Returns the image, the hotspot and the
/// height relative to a standard arrow.
pub fn hand_cursor() -> (Image, Vec2, f64) {
    const PX: f64 = 6.0;
    let pad = 4.0;
    let tip = v2(3.0, -3.0);
    let units = v2(17.0, 22.0);
    let (w, h) = (((units.x + 2.0 * pad) * PX).ceil() as u32, ((units.y + 2.0 * pad) * PX).ceil() as u32);

    // The silhouette: finger, palm, three curled knuckles, thumb, wrist stub.
    let hand = |p: Vec2| -> f64 {
        let mut d = seg_dist(p, tip, v2(6.6, 9.4)) - 2.3;
        d = d.min(round_rect(p, v2(4.6, 6.6), v2(11.0, 7.2), 3.6));
        for i in 0..3 {
            let x = 10.2 + i as f64 * 2.0;
            d = d.min(seg_dist(p, v2(x, 5.6), v2(x + 1.4, 9.6)) - 1.9);
        }
        d.min(seg_dist(p, v2(4.4, 9.6), v2(1.6, 13.4)) - 2.0).min(round_rect(p, v2(5.2, 13.4), v2(8.2, 3.4), 1.8))
    };
    let skin = [0.906, 0.706, 0.561];
    let shade = [0.831, 0.604, 0.451];
    let outline = [0.227, 0.173, 0.133];
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let p = v2((x as f64 + 0.5) / PX - pad, (y as f64 + 0.5) / PX - pad);
            let aa = 0.75 / PX;
            let cover = |d: f64| ((-d) / aa + 0.5).clamp(0.0, 1.0);
            let shadow = cover(hand(p - v2(0.45, 0.85))) * 0.28;
            let ring = cover(hand(p) - 0.55);
            let fill = cover(hand(p));
            let t = ((p.y - tip.y) / 19.0).clamp(0.0, 1.0);
            let col = [skin[0] + (shade[0] - skin[0]) * t, skin[1] + (shade[1] - skin[1]) * t, skin[2] + (shade[2] - skin[2]) * t];
            // Composite premultiplied, back to front: shadow, dark ring, skin.
            let mut a = shadow;
            let mut c = [0.0, 0.0, 0.0];
            let over = |c: &mut [f64; 3], a: &mut f64, col: [f64; 3], cov: f64| {
                for i in 0..3 {
                    c[i] = col[i] * cov + c[i] * (1.0 - cov);
                }
                *a = cov + *a * (1.0 - cov);
            };
            over(&mut c, &mut a, outline, ring);
            over(&mut c, &mut a, col, fill);
            rgba.extend_from_slice(&[(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, (a * 255.0) as u8]);
        }
    }
    let hot = v2((tip.x + pad) / (units.x + 2.0 * pad), (tip.y + pad) / (units.y + 2.0 * pad));
    (Image { width: w, height: h, rgba }, hot, (units.y + 2.0 * pad) / HAND_UNITS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_cursor_is_a_solid_hand_with_the_tip_ahead() {
        let (img, hot, scale) = hand_cursor();
        assert!(img.width > 0 && img.height > 0);
        assert!(hot.x > 0.0 && hot.x < 1.0 && hot.y > 0.0 && hot.y < 1.0, "{hot:?}");
        let alpha = |x: usize, y: usize| img.rgba[(y * img.width as usize + x) * 4 + 3];
        let opaque = img.rgba.chunks_exact(4).filter(|p| p[3] > 200).count();
        assert!(opaque > (img.width * img.height / 8) as usize, "the hand fills a good part of the sprite");
        assert!(alpha(0, 0) < 10, "the top-left corner stays clear");
        let tip = ((hot.x + 0.05) * img.width as f64) as usize;
        assert!(alpha(tip.min(img.width as usize - 1), 2) > 200, "the fingertip is solid");
        assert!((scale - (22.0 + 8.0) / HAND_UNITS).abs() < 1e-6, "scale matches the sprite height");
    }
}
