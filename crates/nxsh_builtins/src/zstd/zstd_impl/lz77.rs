use std::cmp::min;

#[derive(Debug, Clone, Copy)]
pub struct Match { pub len: u32, pub dist: u32 }

/// Enhanced greedy matcher with dictionary support
pub fn find_matches(input: &[u8], window_log: u8, min_match: usize) -> Vec<(usize, Match)> {
    find_matches_with_dict(input, None, window_log, min_match)
}

/// Find matches with optional dictionary context
pub fn find_matches_with_dict(input: &[u8], dict: Option<&[u8]>, window_log: u8, min_match: usize) -> Vec<(usize, Match)> {
    let window = 1usize << window_log;
    let mut i = 0usize;
    let mut res = Vec::new();
    
    while i + min_match <= input.len() {
        let start = i.saturating_sub(window);
        let mut best_len = 0usize;
        let mut best_dist = 0usize;
        let look_max = input.len() - i;
        let cur = &input[i..];
        
        // Search in input window (backwards)
        let mut pos = i;
        while pos > start {
            pos -= 1;
            let dist = i - pos;
            let maxl = min(look_max, window.min(128));
            let mut l = 0usize;
            while l < maxl && pos + l < input.len() && input[pos + l] == cur[l] { 
                l += 1; 
            }
            if l >= min_match && l > best_len {
                best_len = l;
                best_dist = dist;
                if l >= 32 { break; }
            }
        }
        
        // Search in dictionary if available and no good match found
        if let Some(dict_data) = dict {
            if best_len < 8 && !dict_data.is_empty() {
                // Search dictionary backwards from end
                let dict_len = dict_data.len();
                let search_len = min(dict_len, window);
                
                for d_pos in (dict_len.saturating_sub(search_len)..dict_len).rev() {
                    let maxl = min(look_max, dict_len - d_pos);
                    let maxl = min(maxl, 64); // Reasonable limit for dict matches
                    let mut l = 0usize;
                    
                    while l < maxl && d_pos + l < dict_len && dict_data[d_pos + l] == cur[l] {
                        l += 1;
                    }
                    
                    if l >= min_match && l > best_len {
                        best_len = l;
                        // Dictionary distance calculation: end of dict + pos from current
                        best_dist = dict_len - d_pos + i;
                        if l >= 16 { break; }
                    }
                }
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
