//! Mapping between playback time (the edited video) and source time (the
//! recording). The edit is an ordered list of kept slices; gaps are cuts and
//! each slice may play faster or slower.

use crate::project::model::Slice;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Link {
    pub pb_start: f64,
    pub pb_end: f64,
    pub src_start: f64,
    pub src_end: f64,
    pub speed: f64,
    /// Index into the project's slice list.
    pub slice: usize,
}

#[derive(Clone, Debug, Default)]
pub struct TimeMap {
    pub links: Vec<Link>,
    /// Lead-in card before the recording, playback ms. The recording's first
    /// frame shows at this playback time.
    pub intro_ms: f64,
    /// Lead-out card after the recording, playback ms.
    pub outro_ms: f64,
    /// The app's credit at the very end, playback ms.
    pub credit_ms: f64,
}

impl TimeMap {
    /// Build from slices; with no slices the whole recording plays once.
    pub fn new(slices: &[Slice], duration_ms: f64) -> Self {
        /// Source ms eased at each end of a ramped slice, and the sub-steps.
        const RAMP_MAX_SRC: f64 = 800.0;
        const RAMP_STEPS: usize = 8;
        let mut order: Vec<usize> = (0..slices.len()).collect();
        order.sort_by(|&a, &b| slices[a].start.total_cmp(&slices[b].start));
        let mut links = vec![];
        let mut pb = 0.0;
        for i in order {
            let s = &slices[i];
            let (start, end) = (s.start.max(0.0), s.end.min(duration_ms));
            if end <= start {
                continue;
            }
            let speed = if s.speed > 0.0 { s.speed } else { 1.0 };
            let ramp_src = if s.ease && (speed - 1.0).abs() > 1e-3 { RAMP_MAX_SRC.min((end - start) / 3.0) } else { 0.0 };
            let push = |links: &mut Vec<Link>, pb: &mut f64, a: f64, b: f64, sp: f64| {
                let len = (b - a) / sp;
                links.push(Link { pb_start: *pb, pb_end: *pb + len, src_start: a, src_end: b, speed: sp, slice: i });
                *pb += len;
            };
            if ramp_src < 50.0 {
                push(&mut links, &mut pb, start, end, speed);
                continue;
            }
            let sm = |t: f64| t * t * (3.0 - 2.0 * t);
            for k in 0..RAMP_STEPS {
                let (t0, t1) = (k as f64 / RAMP_STEPS as f64, (k + 1) as f64 / RAMP_STEPS as f64);
                let sp = 1.0 + (speed - 1.0) * sm((t0 + t1) / 2.0);
                push(&mut links, &mut pb, start + ramp_src * t0, start + ramp_src * t1, sp);
            }
            let (hold_a, hold_b) = (start + ramp_src, end - ramp_src);
            if hold_b > hold_a {
                push(&mut links, &mut pb, hold_a, hold_b, speed);
            }
            for k in 0..RAMP_STEPS {
                let (t0, t1) = (k as f64 / RAMP_STEPS as f64, (k + 1) as f64 / RAMP_STEPS as f64);
                let sp = speed + (1.0 - speed) * sm((t0 + t1) / 2.0);
                push(&mut links, &mut pb, end - ramp_src + ramp_src * t0, end - ramp_src + ramp_src * t1, sp);
            }
        }
        if links.is_empty() && duration_ms > 0.0 {
            links.push(Link {
                pb_start: 0.0,
                pb_end: duration_ms,
                src_start: 0.0,
                src_end: duration_ms,
                speed: 1.0,
                slice: usize::MAX,
            });
        }
        TimeMap { links, intro_ms: 0.0, outro_ms: 0.0, credit_ms: 0.0 }
    }

    /// Hold the app's credit card after everything else.
    pub fn with_credit(mut self, credit_ms: f64) -> Self {
        self.credit_ms = credit_ms.clamp(0.0, 60_000.0);
        self
    }

    /// Shift the whole recording later by an intro card and hold a silent outro
    /// card at the end. Every link moves, so zooms, masks, sounds and the
    /// recorded audio all keep their places *inside* the video.
    pub fn with_cards(mut self, intro_ms: f64, outro_ms: f64) -> Self {
        let intro = intro_ms.clamp(0.0, 600_000.0);
        if intro > 0.0 {
            for l in &mut self.links {
                l.pb_start += intro;
                l.pb_end += intro;
            }
        }
        self.intro_ms = intro;
        self.outro_ms = outro_ms.clamp(0.0, 600_000.0);
        self
    }

    /// Playback time the recording starts at (the end of the intro card).
    pub fn recording_start(&self) -> f64 {
        self.intro_ms
    }

    /// Playback time the recording ends at (the start of the outro card).
    pub fn recording_end(&self) -> f64 {
        self.links.last().map_or(0.0, |l| l.pb_end)
    }

    /// Total playback length, including both cards.
    pub fn duration(&self) -> f64 {
        self.recording_end() + self.outro_ms + self.credit_ms
    }

    /// Playback time the app's credit starts at.
    pub fn credit_start(&self) -> f64 {
        self.recording_end() + self.outro_ms
    }

    /// Whether `pb` falls on an intro or outro card rather than the recording.
    pub fn on_card(&self, pb: f64) -> bool {
        pb < self.intro_ms || (self.outro_ms > 0.0 && pb >= self.recording_end())
    }

    /// Whether `pb` is inside the app's credit card.
    pub fn on_credit(&self, pb: f64) -> bool {
        self.credit_ms > 0.0 && pb >= self.credit_start()
    }

    pub fn link_at_playback(&self, pb: f64) -> Option<&Link> {
        if self.links.is_empty() {
            return None;
        }
        let i = self.links.partition_point(|l| l.pb_end <= pb);
        Some(&self.links[i.min(self.links.len() - 1)])
    }

    /// Source time shown at a playback time.
    pub fn to_source(&self, pb: f64) -> f64 {
        match self.link_at_playback(pb) {
            Some(l) => (l.src_start + (pb - l.pb_start).max(0.0) * l.speed).min(l.src_end),
            None => pb,
        }
    }

    /// Playback time of a source time, or None if that moment was cut.
    pub fn to_playback(&self, src: f64) -> Option<f64> {
        self.links
            .iter()
            .find(|l| src >= l.src_start && src <= l.src_end)
            .map(|l| l.pb_start + (src - l.src_start) / l.speed)
    }

    /// Nearest playback time for a source time (cut moments snap to the next kept frame).
    pub fn to_playback_nearest(&self, src: f64) -> f64 {
        if let Some(t) = self.to_playback(src) {
            return t;
        }
        for l in &self.links {
            if src < l.src_start {
                return l.pb_start;
            }
        }
        self.duration()
    }

    pub fn speed_at_playback(&self, pb: f64) -> f64 {
        self.link_at_playback(pb).map_or(1.0, |l| l.speed)
    }

    pub fn source_start(&self) -> f64 {
        self.links.first().map_or(0.0, |l| l.src_start)
    }

    pub fn source_end(&self) -> f64 {
        self.links.iter().map(|l| l.src_end).fold(0.0, f64::max)
    }
}

/// The timeline a project plays: its slices over the recording, with the
/// intro, outro and end credit around it. Everything that plays a project
/// builds its timeline this way, so they all agree on its length.
pub fn for_project(project: &crate::project::model::Project, duration_ms: f64) -> TimeMap {
    use crate::project::model::Card;
    let card_ms = |c: &Option<Card>| c.as_ref().map_or(0.0, |c| c.ms.clamp(0.0, Card::MAX_MS));
    let style = &project.style;
    let credit = if style.credit { style.credit_ms.clamp(2000.0, 6000.0) } else { 0.0 };
    TimeMap::new(&project.timeline.slices, duration_ms)
        .with_cards(card_ms(&style.intro), card_ms(&style.outro))
        .with_credit(credit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slice(start: f64, end: f64, speed: f64) -> Slice {
        Slice { start, end, speed, ..Default::default() }
    }

    #[test]
    fn identity_without_slices() {
        let m = TimeMap::new(&[], 5000.0);
        assert_eq!(m.duration(), 5000.0);
        assert_eq!(m.to_source(1234.0), 1234.0);
    }

    #[test]
    fn cuts_and_speed() {
        // Keep 0..1000 at 1x, cut 1000..3000, keep 3000..5000 at 2x.
        let m = TimeMap::new(&[slice(3000.0, 5000.0, 2.0), slice(0.0, 1000.0, 1.0)], 6000.0);
        assert_eq!(m.duration(), 2000.0);
        assert_eq!(m.to_source(500.0), 500.0);
        assert_eq!(m.to_source(1000.0), 3000.0);
        assert_eq!(m.to_source(1500.0), 4000.0);
        assert_eq!(m.to_playback(2000.0), None);
        assert_eq!(m.to_playback(4000.0), Some(1500.0));
        assert_eq!(m.to_playback_nearest(2000.0), 1000.0);
        assert_eq!(m.speed_at_playback(1999.0), 2.0);
    }

    #[test]
    fn cards_pad_both_ends() {
        let m = TimeMap::new(&[], 5000.0).with_cards(2000.0, 1500.0);
        assert_eq!(m.duration(), 8500.0);
        assert_eq!(m.recording_start(), 2000.0);
        assert_eq!(m.recording_end(), 7000.0);
        // On the intro card the recording is held at its first frame.
        assert!(m.on_card(500.0));
        assert_eq!(m.to_source(500.0), 0.0);
        // On the outro card it is held at its last frame.
        assert!(m.on_card(8000.0));
        assert_eq!(m.to_source(8000.0), 5000.0);
        // And the recording itself is untouched, just moved later.
        assert!(!m.on_card(3000.0));
        assert_eq!(m.to_source(3000.0), 1000.0);
        assert_eq!(m.to_playback(1000.0), Some(3000.0));
    }

    #[test]
    fn a_project_is_as_long_as_its_cards_and_credit() {
        let mut p = crate::project::model::Project::default();
        p.style.credit = true;
        p.style.credit_ms = 3400.0;
        assert_eq!(for_project(&p, 2000.0).duration(), 5400.0);
        p.style.credit = false;
        assert_eq!(for_project(&p, 2000.0).duration(), 2000.0);
    }

    #[test]
    fn no_cards_is_the_old_behaviour() {
        let m = TimeMap::new(&[], 5000.0);
        assert_eq!(m.duration(), 5000.0);
        assert_eq!(m.recording_start(), 0.0);
        assert!(!m.on_card(0.0) && !m.on_card(5000.0));
    }

    #[test]
    fn eased_speed_ramps_in_and_out() {
        let slices = [Slice { start: 0.0, end: 3000.0, speed: 3.0, ease: true, ..Default::default() }];
        let m = TimeMap::new(&slices, 3000.0);
        assert!(m.links.len() > 3, "a ramped slice is several links");
        assert!(m.links.first().unwrap().speed < 1.5, "eases in from ~1x");
        assert!(m.links.last().unwrap().speed < 1.5, "and out again");
        assert!(m.links.iter().any(|l| (l.speed - 3.0).abs() < 1e-6), "holds at full speed");
        assert!((m.source_end() - 3000.0).abs() < 1e-6, "no frames lost");
        assert!(m.duration() < 3000.0, "still faster than 1x overall");
        // Without ease it is a single, flat link.
        let plain = TimeMap::new(&[Slice { start: 0.0, end: 3000.0, speed: 3.0, ..Default::default() }], 3000.0);
        assert_eq!(plain.links.len(), 1);
        assert_eq!(plain.duration(), 1000.0);
    }
}
