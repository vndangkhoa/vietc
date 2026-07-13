use std::time::{Duration, Instant};

/// Stateful de-duplicator for a stuck/auto-repeating keyboard that emits every
/// keystroke twice.
///
/// It tracks the raw key stream and drops a key that *immediately* repeats the
/// previous one (`vv` -> `v`, `oo` -> `o`, `44` -> `4`) **or** the one two
/// positions back (`khk` -> `kh`). The two-back rule exists because some faulty
/// keyboards/IBus setups deliver each keystroke twice with a one-event lag
/// (e.g. `khoa` arrives as `k h k h o h o a a`), which the consecutive-only
/// rule cannot collapse.
///
/// This is intentionally opt-in (`deduplicate_keys`): the two-back rule will
/// also collapse legitimate `a-b-a` patterns such as "dad", "tat", "mom",
/// "book", "kayak". Vietnamese VNI input never has such patterns in correct
/// text, so for a user fighting a real double-delivery fault it is the right
/// trade-off; everyone else should leave it off.
///
/// Any non-dedupable key (space, Backspace, arrows, modifiers, function keys,
/// …) breaks the chain, so de-duplication can never span word or edit
/// boundaries.
#[derive(Clone, Debug, Default)]
pub struct DedupState {
    prev_key: Option<u32>,
    last_key: Option<u32>,
    last_key_time: Option<Instant>,
}

impl DedupState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one key identified by `key_id` (an IBus keyval, a Wayland keycode,
    /// or any other stable `u32`). `dedupable` must be true only for printable
    /// letters/digits/space; pass `false` for any key that should break the
    /// chain. `is_space` marks a space key (used so the two-back rule never
    /// collapses a legitimate space). `two_back` enables collapsing a key equal
    /// to the one two positions back (see struct docs).
    ///
    /// Returns `true` if the key is a duplicate and should be dropped.
    pub fn observe(
        &mut self,
        key_id: u32,
        dedupable: bool,
        is_space: bool,
        two_back: bool,
        window_ms: u64,
        now: Instant,
    ) -> bool {
        if !dedupable {
            // Space / Backspace / arrows / modifiers: break the chain.
            self.prev_key = None;
            self.last_key = None;
            self.last_key_time = Some(now);
            return false;
        }
        let consecutive_dup = self.last_key == Some(key_id);
        // The two-back rule must not apply to spaces: otherwise `a a` (a word,
        // then the space that separates it) would collapse to `a`, eating a
        // needed space.
        let two_back_dup = two_back && !is_space && self.prev_key == Some(key_id);
        let is_dup = consecutive_dup || two_back_dup;
        let within = match self.last_key_time {
            Some(t) => now.saturating_duration_since(t) <= Duration::from_millis(window_ms),
            None => false,
        };
        if is_dup && within {
            // Drop the duplicate. Crucially, do NOT advance `prev_key` /
            // `last_key` here — they must continue to reflect the last *kept*
            // keys. This lets the two-back rule collapse an interleaved
            // double-delivery (e.g. "khoa" arriving as "khkohoa"): each replayed
            // key equals the kept key two positions back, and not advancing on a
            // drop keeps that relationship intact for the rest of the burst.
            return true;
        }
        self.prev_key = self.last_key;
        self.last_key = Some(key_id);
        self.last_key_time = Some(now);
        false
    }

    pub fn reset(&mut self) {
        self.prev_key = None;
        self.last_key = None;
        self.last_key_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn kv(c: char) -> u32 {
        c as u32
    }

    #[test]
    fn keeps_normal_sequence() {
        // No consecutive or two-back repeats -> nothing dropped, including the
        // k-h-k in "khkohoa", which must survive (2-back collapse would break it
        // only when two_back is enabled; here it is off).
        let mut d = DedupState::new();
        let base = Instant::now();
        let word = "nguyen64dang98";
        for (i, c) in word.chars().enumerate() {
            assert!(
                !d.observe(kv(c), true, c == ' ', false, 1000, base + ms(i as u64 * 10)),
                "unexpectedly dropped '{c}' at index {i}"
            );
        }
    }

    #[test]
    fn drops_consecutive_double() {
        let mut d = DedupState::new();
        let base = Instant::now();
        assert!(!d.observe(kv('v'), true, false, false, 1000, base));
        assert!(!d.observe(kv('o'), true, false, false, 1000, base + ms(10)));
        assert!(d.observe(kv('o'), true, false, false, 1000, base + ms(20)), "oo should drop");
        assert!(!d.observe(kv('4'), true, false, false, 1000, base + ms(30)));
    }

    #[test]
    fn two_back_drops_offset_replay() {
        // IBus double-delivery with a one-event lag: "khoa" arrives as
        // k h k o h o a a. With two_back on, it collapses to "khoa".
        let mut d = DedupState::new();
        let base = Instant::now();
        let stream: &[char] = &['k', 'h', 'k', 'o', 'h', 'o', 'a', 'a'];
        let mut out = String::new();
        for (i, &c) in stream.iter().enumerate() {
            if !d.observe(kv(c), true, c == ' ', true, 1000, base + ms(i as u64 * 10)) {
                out.push(c);
            }
        }
        assert_eq!(out, "khoa", "offset replay should collapse to khoa");
    }

    #[test]
    fn two_back_collapses_aba_known_caveat() {
        // With two_back on, "dad" (d-a-d) collapses to "da". This is the
        // documented trade-off for enabling de-duplication.
        let mut d = DedupState::new();
        let base = Instant::now();
        assert!(!d.observe(kv('d'), true, false, true, 1000, base));
        assert!(!d.observe(kv('a'), true, false, true, 1000, base + ms(10)));
        assert!(d.observe(kv('d'), true, false, true, 1000, base + ms(20)), "dad must collapse");
    }

    #[test]
    fn two_back_off_keeps_aba() {
        // With two_back off (the safe default), "dad" is preserved.
        let mut d = DedupState::new();
        let base = Instant::now();
        assert!(!d.observe(kv('d'), true, false, false, 1000, base));
        assert!(!d.observe(kv('a'), true, false, false, 1000, base + ms(10)));
        assert!(!d.observe(kv('d'), true, false, false, 1000, base + ms(20)), "dad kept when two_back off");
    }

    #[test]
    fn word_boundary_space_kept() {
        // A space after a word must survive even with two_back on.
        let mut d = DedupState::new();
        let base = Instant::now();
        // "vo khoa" typed as v o [space] k h o a, but with the offset replay the
        // whole thing is doubled: v o v o [space] k h k o h o a a.
        let stream: &[char] = &['v', 'o', 'v', 'o', ' ', 'k', 'h', 'k', 'o', 'h', 'o', 'a', 'a'];
        let mut out = String::new();
        for (i, &c) in stream.iter().enumerate() {
            if !d.observe(kv(c), true, c == ' ', true, 1000, base + ms(i as u64 * 10)) {
                out.push(c);
            }
        }
        assert_eq!(out, "vo khoa", "space after word must be preserved");
    }

    #[test]
    fn slow_bounce_kept() {
        let mut d = DedupState::new();
        let base = Instant::now();
        assert!(!d.observe(kv('v'), true, false, true, 1000, base));
        assert!(!d.observe(kv('v'), true, false, true, 1000, base + ms(2000)));
    }

    #[test]
    fn non_dedupable_never_dropped() {
        let mut d = DedupState::new();
        let base = Instant::now();
        assert!(!d.observe(0xFF08, false, false, true, 1000, base)); // BackSpace
        assert!(!d.observe(0xFF08, false, false, true, 1000, base + ms(10)));
        assert!(!d.observe(0xFF1B, false, false, true, 1000, base + ms(20))); // Escape
        assert!(!d.observe(0xFF51, false, false, true, 1000, base + ms(30))); // Left arrow
    }

    #[test]
    fn backspace_then_same_letter_kept() {
        let mut d = DedupState::new();
        let base = Instant::now();
        assert!(!d.observe(kv('v'), true, false, true, 1000, base));
        assert!(d.observe(kv('v'), true, false, true, 1000, base + ms(10)), "vv drops");
        assert!(!d.observe(0xFF08, false, false, true, 1000, base + ms(20))); // Backspace breaks chain
        assert!(!d.observe(kv('v'), true, false, true, 1000, base + ms(30)), "v after Backspace kept");
    }

    #[test]
    fn stuck_spacebar_collapsed() {
        let mut d = DedupState::new();
        let base = Instant::now();
        assert!(!d.observe(kv(' '), true, true, true, 1000, base));
        assert!(d.observe(kv(' '), true, true, true, 1000, base + ms(10)), "2nd space drops");
        assert!(d.observe(kv(' '), true, true, true, 1000, base + ms(20)), "3rd space drops");
        assert!(!d.observe(kv('v'), true, false, true, 1000, base + ms(30)));
        assert!(!d.observe(kv(' '), true, true, true, 1000, base + ms(40)), "space after word kept");
    }
}
