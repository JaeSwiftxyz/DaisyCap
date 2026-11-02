//! The word selection in the Captions tab: which transcript spans are selected, and where a Delete
//! press goes. The selection holds recording times rather than list positions, so it stays right
//! when the transcript is replaced.

use crate::project::model::Slice;

/// The transcript spans the user has selected, in recording milliseconds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WordSelection {
    pub spans: Vec<(f64, f64)>,
    /// Start time of the word the last plain click landed on. A shift-click extends from it.
    pub anchor: Option<f64>,
}

impl WordSelection {
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    pub fn clear(&mut self) {
        self.spans.clear();
        self.anchor = None;
    }
}

/// What a Delete or Backspace press does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeleteTarget {
    /// Cut the selected words.
    Words,
    /// Delete the selected timeline item, as before.
    Timeline,
}

/// Delete cuts words only while the Captions section shows and a kept word is selected.
pub fn delete_target(section_shown: bool, has_kept_selected: bool) -> DeleteTarget {
    if section_shown && has_kept_selected {
        DeleteTarget::Words
    } else {
        DeleteTarget::Timeline
    }
}

/// The selected spans that are still kept. Cut words are left out, so Delete never cuts twice.
pub fn kept_spans(spans: &[(f64, f64)], slices: &[Slice]) -> Vec<(f64, f64)> {
    spans
        .iter()
        .copied()
        .filter(|&(s, e)| crate::engine::words::span_is_kept(slices, s, e))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slice(start: f64, end: f64) -> Slice {
        Slice { start, end, ..Default::default() }
    }

    #[test]
    fn delete_cuts_words_only_while_the_section_shows() {
        assert_eq!(delete_target(true, true), DeleteTarget::Words);
        assert_eq!(delete_target(false, true), DeleteTarget::Timeline);
        assert_eq!(delete_target(true, false), DeleteTarget::Timeline);
        assert_eq!(delete_target(false, false), DeleteTarget::Timeline);
    }

    #[test]
    fn clearing_drops_the_spans_and_the_anchor() {
        let mut sel = WordSelection { spans: vec![(1.0, 2.0)], anchor: Some(1.0) };
        assert!(!sel.is_empty());
        sel.clear();
        assert!(sel.is_empty());
        assert_eq!(sel.anchor, None);
    }

    #[test]
    fn only_kept_spans_are_cut_again_when_deleting() {
        let slices = [slice(0.0, 1000.0)];
        assert_eq!(kept_spans(&[(100.0, 200.0), (1500.0, 1600.0)], &slices), vec![(100.0, 200.0)]);
    }
}
