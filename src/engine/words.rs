//! Word edits: which transcript words are cut, and the timeline edits that cut or restore them.
//! Pure functions over the timeline, so the rules are tested without a GPU.

use crate::project::model::{Slice, Timeline};
use crate::project::recording::Word;

/// Filler words, compared lowercased with punctuation removed.
const FILLERS: [&str; 8] = ["um", "uh", "uhm", "er", "erm", "ah", "hmm", "mm"];

fn midpoint(w: &Word) -> f64 {
    (w.start + w.end) / 2.0
}

/// Whether a time falls inside a kept slice.
fn kept_at(slices: &[Slice], t: f64) -> bool {
    slices.iter().any(|s| t >= s.start && t < s.end)
}

/// A word belongs to the recording when it starts before the recording ends. Words past the end
/// can't play, so the word view, its count and the subtitles leave them out.
pub fn in_recording(w: &Word, duration: f64) -> bool {
    w.start < duration
}

/// Whether a source span is kept: its midpoint lies inside a slice.
pub fn span_is_kept(slices: &[Slice], start: f64, end: f64) -> bool {
    kept_at(slices, (start + end) / 2.0)
}

/// A word is cut when it belongs to the recording and its midpoint lies outside every slice.
pub fn is_cut(w: &Word, slices: &[Slice], duration: f64) -> bool {
    in_recording(w, duration) && !kept_at(slices, midpoint(w))
}

/// The words of the recording that survive the cuts, in transcript order.
pub fn kept_words(words: &[Word], slices: &[Slice], duration: f64) -> Vec<Word> {
    words
        .iter()
        .filter(|w| in_recording(w, duration) && kept_at(slices, midpoint(w)))
        .cloned()
        .collect()
}

/// The caption lines a recording plays: lines built from the kept words only, so no caption
/// shows a cut word on the frames around its cut.
pub fn caption_lines(words: &[Word], slices: &[Slice], duration: f64) -> Vec<crate::engine::captions::Line> {
    crate::engine::captions::lines(&kept_words(words, slices, duration))
}

/// "word" for one, "words" otherwise, for labels and status lines.
pub fn noun(n: usize) -> &'static str {
    if n == 1 { "word" } else { "words" }
}

/// Whether a word is a filler: um, uh and similar, ignoring case and punctuation.
pub fn is_filler(text: &str) -> bool {
    let word: String = text.chars().filter(|c| c.is_alphabetic()).collect::<String>().to_lowercase();
    FILLERS.contains(&word.as_str())
}

/// The parts of `start..end` that no slice covers, in time order.
pub fn gaps_within(slices: &[Slice], start: f64, end: f64) -> Vec<(f64, f64)> {
    let mut covered: Vec<(f64, f64)> = slices
        .iter()
        .filter(|s| s.end > start && s.start < end)
        .map(|s| (s.start.max(start), s.end.min(end)))
        .collect();
    covered.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut gaps = Vec::new();
    let mut at = start;
    for (s, e) in covered {
        if s > at {
            gaps.push((at, s));
        }
        at = at.max(e);
    }
    if at < end {
        gaps.push((at, end));
    }
    gaps
}

/// Cut each source span out of the timeline, as `typing::cut` does. Returns whether anything changed.
pub fn cut_spans(tl: &mut Timeline, duration: f64, spans: &[(f64, f64)]) -> bool {
    let mut changed = false;
    for &(s, e) in spans {
        changed |= crate::engine::typing::cut(tl, duration, s, e);
    }
    changed
}

/// Restore each source span: every part of it that no slice covers gets a plain slice, at speed 1
/// and full volume. Kept audio is never overlapped. Returns whether anything was added.
pub fn restore_spans(tl: &mut Timeline, spans: &[(f64, f64)]) -> bool {
    let mut added = false;
    for &(s, e) in spans {
        for (gs, ge) in gaps_within(&tl.slices, s, e) {
            let id = tl.new_id();
            tl.slices.push(Slice { id, start: gs, end: ge, ..Default::default() });
            added = true;
        }
    }
    tl.slices.sort_by(|a, b| a.start.total_cmp(&b.start));
    added
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(start: f64, end: f64, text: &str) -> Word {
        Word { start, end, text: text.into() }
    }

    fn slice(start: f64, end: f64) -> Slice {
        Slice { start, end, ..Default::default() }
    }

    /// Slices that touch are merged, so a timeline reads as the stretches it keeps.
    fn coverage(tl: &Timeline) -> Vec<(f64, f64)> {
        let mut out: Vec<(f64, f64)> = Vec::new();
        for s in &tl.slices {
            match out.last_mut() {
                Some(last) if (last.1 - s.start).abs() < 1e-9 => last.1 = s.end,
                _ => out.push((s.start, s.end)),
            }
        }
        out
    }

    #[test]
    fn a_word_is_cut_when_its_midpoint_is_in_a_gap() {
        let slices = [slice(0.0, 1000.0), slice(2000.0, 5000.0)];
        assert!(is_cut(&word(900.0, 1100.0, "a"), &slices, 5000.0));
        assert!(!is_cut(&word(950.0, 1040.0, "b"), &slices, 5000.0));
        assert!(is_cut(&word(1200.0, 1400.0, "c"), &slices, 5000.0));
    }

    #[test]
    fn words_past_the_end_belong_to_no_edit() {
        let slices = [slice(0.0, 5000.0)];
        let late = word(5000.0, 5200.0, "late");
        assert!(!in_recording(&late, 5000.0));
        assert!(!is_cut(&late, &slices, 5000.0));
        let words = [word(4800.0, 4900.0, "on"), late];
        assert_eq!(kept_words(&words, &slices, 5000.0).len(), 1);
    }

    #[test]
    fn labels_say_word_for_one_and_words_otherwise() {
        assert_eq!(noun(0), "words");
        assert_eq!(noun(1), "word");
        assert_eq!(noun(2), "words");
    }

    #[test]
    fn fillers_ignore_case_and_punctuation() {
        assert!(is_filler("Um,"));
        assert!(is_filler("uh"));
        assert!(is_filler("HMM."));
        assert!(!is_filler("like"));
        assert!(!is_filler("uh-huh"));
    }

    #[test]
    fn gaps_are_the_uncovered_parts() {
        let slices = [slice(0.0, 1000.0), slice(2000.0, 3000.0), slice(3500.0, 4000.0)];
        assert_eq!(gaps_within(&slices, 500.0, 3800.0), vec![(1000.0, 2000.0), (3000.0, 3500.0)]);
        assert!(gaps_within(&slices, 0.0, 900.0).is_empty());
    }

    #[test]
    fn cut_then_restore_round_trips() {
        let mut tl = Timeline::default();
        assert!(cut_spans(&mut tl, 5000.0, &[(1000.0, 2000.0)]));
        assert_eq!(coverage(&tl), vec![(0.0, 1000.0), (2000.0, 5000.0)]);
        assert!(restore_spans(&mut tl, &[(1000.0, 2000.0)]));
        assert_eq!(coverage(&tl), vec![(0.0, 5000.0)]);
    }

    #[test]
    fn restore_fills_only_the_gap_and_never_overlaps_kept_audio() {
        let mut tl = Timeline::default();
        cut_spans(&mut tl, 5000.0, &[(1000.0, 2000.0)]);
        assert!(restore_spans(&mut tl, &[(500.0, 2500.0)]));
        assert_eq!(tl.slices.len(), 3);
        assert!(tl.slices.windows(2).all(|w| w[0].end <= w[1].start));
    }

    #[test]
    fn a_caption_that_spans_a_cut_keeps_only_the_kept_words() {
        let slices = [slice(0.0, 1000.0), slice(1100.0, 5000.0)];
        let words = [word(900.0, 990.0, "keep"), word(1020.0, 1080.0, "cut"), word(1120.0, 1300.0, "too")];
        let texts: Vec<String> = caption_lines(&words, &slices, 5000.0).iter().map(|l| l.text()).collect();
        assert_eq!(texts, vec!["keep too".to_string()]);
    }

    #[test]
    fn restoring_kept_audio_changes_nothing() {
        let mut tl = Timeline::default();
        cut_spans(&mut tl, 5000.0, &[(1000.0, 2000.0)]);
        let before = tl.slices.clone();
        assert!(!restore_spans(&mut tl, &[(2100.0, 2400.0)]));
        assert_eq!(tl.slices, before);
    }
}
