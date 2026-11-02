//! The word view in the Captions tab: the transcript as text, one row per caption line. Cut words
//! are struck through and fillers underlined. A press selects a word, a shift-press extends the
//! selection from the anchor, and dragging selects a run. A press also moves the playhead there.

use egui::{RichText, Ui};

use super::editor::Editor;
use super::theme::pal;
use crate::engine::{captions, timemap, words};
use crate::project::recording::Word;

pub fn word_view(ui: &mut Ui, ed: &mut Editor) {
    let duration = ed.rec.meta.duration_ms;
    let list: Vec<Word> = ed
        .rec
        .transcript
        .iter()
        .filter(|w| words::in_recording(w, duration) && !w.text.trim().is_empty())
        .cloned()
        .collect();
    let slices = ed.project.timeline.slices.clone();
    let cut = list.iter().filter(|w| words::is_cut(w, &slices, duration)).count();
    ui.label(RichText::new(format!("{} {}, {} cut", list.len(), words::noun(list.len()), cut)).size(11.0).color(pal(ui).secondary));
    if list.is_empty() {
        return;
    }

    let rows = captions::lines(&list);
    let map = timemap::for_project(&ed.project, duration);
    let (pressed, down, released) = ui.input(|i| (i.pointer.primary_pressed(), i.pointer.primary_down(), i.pointer.primary_released()));
    let shift = ui.input(|i| i.modifiers.shift);
    let pointer = ui.input(|i| i.pointer.interact_pos());
    let drag_id = ui.id().with("word-drag");
    let mut drag: Option<usize> = ui.data(|d| d.get_temp::<Option<usize>>(drag_id)).flatten();
    let mut over: Option<usize> = None;
    let mut click: Option<usize> = None;

    egui::ScrollArea::vertical().id_salt("word-view").max_height(240.0).auto_shrink([false, true]).show(ui, |ui| {
        let mut index = 0;
        for row in &rows {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                for w in &row.words {
                    let selected = ed.word_sel.spans.iter().any(|&(s, e)| s == w.start && e == w.end);
                    let mut text = RichText::new(w.text.trim());
                    if words::is_cut(w, &slices, duration) {
                        text = text.strikethrough().color(pal(ui).secondary);
                    }
                    if words::is_filler(&w.text) {
                        text = text.underline();
                    }
                    let response = ui.selectable_label(selected, text);
                    if pointer.is_some_and(|p| response.rect.contains(p)) {
                        over = Some(index);
                        if pressed {
                            click = Some(index);
                        }
                    }
                    index += 1;
                }
            });
        }
    });

    let anchor = ed.word_sel.anchor.and_then(|a| list.iter().position(|w| w.start == a));
    if let Some(i) = click {
        let origin = match (shift, anchor) {
            (true, Some(a)) => a,
            _ => {
                ed.word_sel.anchor = Some(list[i].start);
                i
            }
        };
        drag = Some(origin);
        ed.word_sel.spans = spans(&list, origin, i);
        ed.seek(map.to_playback_nearest(list[i].start));
    } else if let (Some(origin), Some(i), true) = (drag, over, down) {
        ed.word_sel.spans = spans(&list, origin, i);
    }
    if released {
        drag = None;
    }
    ui.data_mut(|d| d.insert_temp(drag_id, drag));

    let selected_cut: Vec<(f64, f64)> = ed.word_sel.spans.iter().copied().filter(|&(s, e)| !words::span_is_kept(&slices, s, e)).collect();
    let fillers = list.iter().filter(|w| words::is_filler(&w.text) && !words::is_cut(w, &slices, duration)).count();
    ui.horizontal(|ui| {
        if !selected_cut.is_empty() && ui.button(format!("Restore {} {}", selected_cut.len(), words::noun(selected_cut.len()))).clicked() {
            ed.restore_words(&selected_cut);
        }
        if fillers > 0 && ui.button(format!("Cut {fillers} filler {}", words::noun(fillers))).clicked() {
            ed.cut_filler_words();
        }
    });
}

/// The source spans of the words from `a` to `b`, in either order.
fn spans(list: &[Word], a: usize, b: usize) -> Vec<(f64, f64)> {
    let (lo, hi) = (a.min(b), a.max(b));
    list[lo..=hi].iter().map(|w| (w.start, w.end)).collect()
}
