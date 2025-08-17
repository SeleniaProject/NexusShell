use std::cmp::min;

#[derive(Debug, Clone, Copy)]
pub struct Match { pub len: u32, pub dist: u32 }

/// Very simple greedy matcher (placeholder for HC). Window limited by `window_log`.
pub fn find_matches(input: &[u8], window_log: u8, min_match: usize) -> Vec<(usize, Match)> {
    let window = 1usize << window_log;
    let mut i = 0usize;
    let mut res = Vec::new();
    while i + min_match <= input.len() {
        let start = if i > window { i - window } else { 0 };
        let mut best_len = 0usize;
        let mut best_dist = 0usize;
        let look_max = input.len() - i;
        // naive backward scan
        let cur = &input[i..];
        let mut pos = i;
        while pos > start {
            pos -= 1;
            let dist = i - pos;
            let maxl = min(look_max, window.min(128));
            let mut l = 0usize;
            while l < maxl && input[pos + l] == cur[l] { l += 1; }
            if l >= min_match && l > best_len {
                best_len = l;
                best_dist = dist;
                if l >= 32 { break; }
            }
        }
        if best_len >= min_match {
            res.push((i, Match { len: best_len as u32, dist: best_dist as u32 }));
            i += best_len;
        } else {
            i += 1;
        }
    }
    res
}
