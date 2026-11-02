//! Recorded keyboard presses, embedded in the binary.
//!
//! Two CC0 (public domain) packs, taken from the clatterbox project: a muted
//! office keyboard and a tactile mechanical one. See `assets/sounds/CREDITS.md`
//! for provenance and licences. The presses are real single-key recordings, so
//! typing sounds like a keyboard rather than a machine gun: each key code picks
//! a different recording, keyed round-robin like a real board's variation.
//!
//! Space, return and delete land heavier, as on a real board.

use std::sync::OnceLock;

use crate::capture::mixer::RATE;

/// A recorded keyboard.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pack {
    Office,
    Tactile,
}

const OFFICE: [&[u8]; 16] = [
    include_bytes!("../../assets/sounds/office/down-01.wav"),
    include_bytes!("../../assets/sounds/office/down-02.wav"),
    include_bytes!("../../assets/sounds/office/down-03.wav"),
    include_bytes!("../../assets/sounds/office/down-04.wav"),
    include_bytes!("../../assets/sounds/office/down-05.wav"),
    include_bytes!("../../assets/sounds/office/down-06.wav"),
    include_bytes!("../../assets/sounds/office/down-07.wav"),
    include_bytes!("../../assets/sounds/office/down-08.wav"),
    include_bytes!("../../assets/sounds/office/down-09.wav"),
    include_bytes!("../../assets/sounds/office/down-10.wav"),
    include_bytes!("../../assets/sounds/office/down-11.wav"),
    include_bytes!("../../assets/sounds/office/down-12.wav"),
    include_bytes!("../../assets/sounds/office/down-13.wav"),
    include_bytes!("../../assets/sounds/office/down-14.wav"),
    include_bytes!("../../assets/sounds/office/down-15.wav"),
    include_bytes!("../../assets/sounds/office/down-16.wav"),
];

const TACTILE: [&[u8]; 12] = [
    include_bytes!("../../assets/sounds/tactile/down-01.wav"),
    include_bytes!("../../assets/sounds/tactile/down-02.wav"),
    include_bytes!("../../assets/sounds/tactile/down-03.wav"),
    include_bytes!("../../assets/sounds/tactile/down-04.wav"),
    include_bytes!("../../assets/sounds/tactile/down-05.wav"),
    include_bytes!("../../assets/sounds/tactile/down-06.wav"),
    include_bytes!("../../assets/sounds/tactile/down-07.wav"),
    include_bytes!("../../assets/sounds/tactile/down-08.wav"),
    include_bytes!("../../assets/sounds/tactile/down-09.wav"),
    include_bytes!("../../assets/sounds/tactile/down-10.wav"),
    include_bytes!("../../assets/sounds/tactile/down-11.wav"),
    include_bytes!("../../assets/sounds/tactile/down-12.wav"),
];

/// A key press for `code`, ready to mix: a recording from `pack`, picked by
/// key code, with the small gain variation a real board has and a heavier
/// space and return.
pub fn key(pack: Pack, code: u16) -> Vec<[f32; 2]> {
    let h = hash32(code as u32);
    let (src, pitch, gain) = match code {
        // Space is the heaviest key on any board, and return is close behind.
        49 => (special(pack, Special::Heavy), 0.86, 1.30),
        36 | 76 => (special(pack, Special::Heavy), 0.93, 1.15),
        51 | 117 => (special(pack, Special::Crisp), 1.06, 1.08),
        _ => (round_robin(pack, h), 1.0, 0.94 + ((h >> 8) % 3) as f32 * 0.06),
    };
    resample(src, pitch, gain)
}

enum Special {
    Heavy,
    Crisp,
}

fn special(pack: Pack, which: Special) -> &'static [f32] {
    match (pack, which) {
        (Pack::Tactile, Special::Heavy) => tactile_enter(),
        (Pack::Tactile, Special::Crisp) => tactile_backspace(),
        // The office pack has no keys of its own: take a recording and shift it.
        (Pack::Office, _) => round_robin(Pack::Office, 3),
    }
}

fn round_robin(pack: Pack, h: u32) -> &'static [f32] {
    let all = pack_samples(pack);
    &all[(h as usize) % all.len()]
}

fn pack_samples(pack: Pack) -> &'static [Vec<f32>] {
    static OFFICE_C: OnceLock<Vec<Vec<f32>>> = OnceLock::new();
    static TACTILE_C: OnceLock<Vec<Vec<f32>>> = OnceLock::new();
    match pack {
        Pack::Office => OFFICE_C.get_or_init(|| OFFICE.iter().map(|b| press(b)).collect()),
        Pack::Tactile => TACTILE_C.get_or_init(|| TACTILE.iter().map(|b| press(b)).collect()),
    }
}

fn tactile_enter() -> &'static [f32] {
    static C: OnceLock<Vec<f32>> = OnceLock::new();
    C.get_or_init(|| press(include_bytes!("../../assets/sounds/tactile/enter-down-01.wav")))
}

fn tactile_backspace() -> &'static [f32] {
    static C: OnceLock<Vec<f32>> = OnceLock::new();
    C.get_or_init(|| press(include_bytes!("../../assets/sounds/tactile/backspace-down-01.wav")))
}

/// Decode one recording and cut its silent tail.
fn press(wav: &[u8]) -> Vec<f32> {
    trim(decode(wav))
}

/// Decode a 16-bit mono PCM WAV (what `afconvert` writes).
fn decode(wav: &[u8]) -> Vec<f32> {
    let mut i = 12;
    while i + 8 <= wav.len() {
        let id = &wav[i..i + 4];
        let size = u32::from_le_bytes(wav[i + 4..i + 8].try_into().unwrap()) as usize;
        if id == b"data" {
            let end = (i + 8 + size).min(wav.len());
            return wav[i + 8..end].chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0).collect();
        }
        // Chunks are padded to an even length.
        i += 8 + size + (size & 1);
    }
    Vec::new()
}

/// Cut the silence at both ends. The head is cut to the attack (some
/// recordings carry up to 30 ms of room tone before the press, which would
/// otherwise land late), the tail to where it dies away, and the length is
/// capped.
fn trim(mut wav: Vec<f32>) -> Vec<f32> {
    /// Below this is room tone, not the press.
    const ATTACK: f32 = 0.02;
    const FLOOR: f32 = 0.001;
    let lead = 8;
    let tail = (RATE * 0.008) as usize;
    let len = wav.len();
    let start = wav.iter().position(|v| v.abs() > ATTACK).map_or(0, |i| i.saturating_sub(lead)).min(len.saturating_sub(1));
    let end = wav.iter().rposition(|v| v.abs() > FLOOR).map_or(len, |i| (i + tail).min(len));
    wav.drain(..start);
    let keep = end.saturating_sub(start).clamp(1, (RATE * 0.3) as usize).min(wav.len());
    wav.truncate(keep);
    wav
}

/// Resample by `pitch` and scale by `gain`, keeping the length natural.
fn resample(src: &[f32], pitch: f32, gain: f32) -> Vec<[f32; 2]> {
    let n = (src.len() as f32 / pitch) as usize;
    (0..n)
        .map(|i| {
            let x = i as f32 * pitch;
            let j = x as usize;
            let f = x - j as f32;
            let a = src[j.min(src.len() - 1)];
            let b = src.get(j + 1).copied().unwrap_or(a);
            let v = (a + (b - a) * f) * gain;
            [v, v]
        })
        .collect()
}

fn hash32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pack_decodes_to_short_audible_presses() {
        for pack in [Pack::Office, Pack::Tactile] {
            let all = pack_samples(pack);
            assert!(all.len() >= 12, "{} samples", all.len());
            for s in all {
                assert!(!s.is_empty(), "empty sample");
                let peak = s.iter().fold(0.0f32, |m, v| m.max(v.abs()));
                assert!(peak > 0.05 && peak <= 1.0, "peak {peak}");
                // Short: nothing like a whole second of audio.
                assert!(s.len() < (RATE * 0.31) as usize, "{} samples long", s.len());
                // Starts near the attack, not with silence.
                assert!(s[..32.min(s.len())].iter().any(|v| v.abs() > 0.005));
            }
        }
    }

    #[test]
    fn keys_vary_and_space_lands_heavier() {
        let a = key(Pack::Tactile, 8);
        let b = key(Pack::Tactile, 40);
        assert_ne!(a, b, "different keys pick different recordings");
        let rms = |s: &[[f32; 2]]| (s.iter().map(|v| v[0] * v[0]).sum::<f32>() / s.len() as f32).sqrt();
        assert!(rms(&key(Pack::Tactile, 49)) > rms(&key(Pack::Tactile, 8)), "space is heavier");
        assert!(rms(&key(Pack::Office, 49)) > rms(&key(Pack::Office, 8)), "space is heavier here too");
    }

    #[test]
    fn decoding_a_bad_file_gives_nothing_rather_than_panicking() {
        assert!(decode(&[]).is_empty());
        assert!(decode(b"RIFFxxxxWAVE").is_empty());
        assert!(!key(Pack::Office, 3).is_empty());
    }
}
