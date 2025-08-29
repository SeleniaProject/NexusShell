//! Huffman coding algorithms - math-intensive loops optimized for correctness over style
#![allow(clippy::needless_range_loop)]

#[derive(Clone)]
pub struct HuffmanTable {
    // For each literal value 0..=255, an optional (code, bitlen)
    pub codes: Vec<Option<(u16, u8)>>,
    // Highest literal index present in this table (inclusive)
    pub last_symbol: usize,
    // Weights for 0..=last_symbol (last one is present but not emitted in direct header)
    pub weights: Vec<u8>,
}

// Build a Huffman table for literals and return (table, Huffman_Tree_Description bytes) using
// Direct Weights header (RFC 8878 4.2.1.1). Constraints:
// - Max code length <= 11
// - Highest literal value <= 127 (so Number_of_Symbols <= 128)
// - At least 2 distinct symbols (otherwise RLE/RAW path is preferable)
pub fn build_literals_huffman(data: &[u8]) -> Option<(HuffmanTable, Vec<u8>)> {
    if data.len() < 2 {
        return None;
    }
    // Histogram
    let mut freq = [0u32; 256];
    for &b in data {
        freq[b as usize] = freq[b as usize].saturating_add(1);
    }
    // Find last present symbol
    let last_sym = (0..256).rfind(|&i| freq[i] > 0)?;
    // If only one symbol present, prefer RLE literals
    let present_count = freq.iter().filter(|&&f| f > 0).count();
    if present_count <= 1 {
        return None;
    }
    // Direct-weights headerは最高リテラル値が127以下でないと不可
    if last_sym > 127 {
        return None;
    }

    // Build Huffman code lengths with a standard Huffman tree; if exceeds 11,
    // fallback to a length-limited redistribution (cap at 11).
    let mut lengths = [0u8; 256];

    // Collect nodes (symbol, freq). Zero-freq symbols are excluded from tree building.
    #[derive(Clone, Copy)]
    struct Node {
        freq: u32,
        sym: i16,
        left: i16,
        right: i16,
    }
    let mut nodes: Vec<Node> = Vec::new();
    for i in 0..=last_sym {
        if freq[i] > 0 {
            nodes.push(Node {
                freq: freq[i],
                sym: i as i16,
                left: -1,
                right: -1,
            });
        }
    }
    if nodes.len() == 1 {
        return None;
    }
    // Min-heap of indices into nodes
    use std::cmp::Ordering;
    #[derive(Clone, Copy)]
    struct HeapItem {
        freq: u32,
        idx: usize,
    }
    impl PartialEq for HeapItem {
        fn eq(&self, o: &Self) -> bool {
            self.freq == o.freq && self.idx == o.idx
        }
    }
    impl Eq for HeapItem {}
    impl PartialOrd for HeapItem {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for HeapItem {
        fn cmp(&self, o: &Self) -> Ordering {
            // reverse for min-heap using BinaryHeap (max-heap)
            o.freq.cmp(&self.freq).then_with(|| o.idx.cmp(&self.idx))
        }
    }
    let mut heap: std::collections::BinaryHeap<HeapItem> = std::collections::BinaryHeap::new();
    for (i, n) in nodes.iter().enumerate() {
        heap.push(HeapItem {
            freq: n.freq,
            idx: i,
        });
    }
    // Build tree by adding internal nodes appended to nodes vector
    while heap.len() > 1 {
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        let new_idx = nodes.len();
        let new_node = Node {
            freq: a.freq.saturating_add(b.freq),
            sym: -1,
            left: a.idx as i16,
            right: b.idx as i16,
        };
        nodes.push(new_node);
        heap.push(HeapItem {
            freq: nodes[new_idx].freq,
            idx: new_idx,
        });
    }
    let root_idx = heap.pop().unwrap().idx;
    // Traverse to get depths
    fn assign_depths(nodes: &Vec<Node>, idx: usize, depth: u8, lengths: &mut [u8; 256]) {
        let n = nodes[idx];
        if n.sym >= 0 {
            lengths[n.sym as usize] = depth.max(1);
            return;
        }
        if n.left >= 0 {
            assign_depths(nodes, n.left as usize, depth + 1, lengths);
        }
        if n.right >= 0 {
            assign_depths(nodes, n.right as usize, depth + 1, lengths);
        }
    }
    assign_depths(&nodes, root_idx, 0, &mut lengths);
    // Compute MaxBits
    let mut max_bits = 0u8;
    for i in 0..=last_sym {
        let l = lengths[i];
        if freq[i] > 0 && l > max_bits {
            max_bits = l;
        }
    }
    if max_bits == 0 {
        return None;
    }

    // If max_bits exceeds 11, clamp using a redistribution of bit-length counts.
    if max_bits > 11 {
        // Collect (sym, freq) for present symbols and sort by descending freq, then sym asc
        let mut present: Vec<(usize, u32)> = (0..=last_sym)
            .filter(|&i| freq[i] > 0)
            .map(|i| (i, freq[i]))
            .collect();
        present.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        // Build unconstrained bl_count first
        let mut bl_count = [0i32; 64];
        for i in 0..=last_sym {
            let l = lengths[i];
            if l > 0 {
                bl_count[l as usize] += 1;
            }
        }
        let mut cur_max = max_bits as usize;
        // Redistribute counts so that no length > 11
        while cur_max > 11 {
            if bl_count[cur_max] > 0 {
                // Move two codes from cur_max to one code at cur_max-1
                bl_count[cur_max] -= 2;
                bl_count[cur_max - 1] += 1;
                // If after move, still negative (odd), borrow from deeper levels
                if bl_count[cur_max] < 0 {
                    bl_count[cur_max] = 0;
                }
                continue;
            }
            cur_max -= 1;
        }
        // Ensure there are no counts below 1-bit negative
        for b in 1..=11 {
            if bl_count[b] < 0 {
                bl_count[b] = 0;
            }
        }
        // Fix total number of codes: sum(bl_count) must equal number of symbols
        let total_syms = present.len() as i32;
        let mut sum_counts: i32 = (1..=11).map(|b| bl_count[b]).sum();
        // If counts too small, promote some by splitting at longest available bucket
        while sum_counts < total_syms {
            // Find deepest non-empty bucket to split
            let mut d = 11;
            while d > 1 && bl_count[d] == 0 {
                d -= 1;
            }
            if bl_count[d] > 0 {
                bl_count[d] -= 1;
                bl_count[d + 1] += 2; // split one into two deeper codes
                sum_counts += 1; // net +1 symbol
                if d + 1 > 11 {
                    // need to pull back to the cap by iterative reduction
                    bl_count[d + 1] -= 2;
                    bl_count[d] += 1;
                    // fallback: put at cap
                    bl_count[11] += 1;
                }
            } else {
                break;
            }
        }
        // If counts too many, merge two deepest into one shorter
        while sum_counts > total_syms {
            let mut d = 2;
            while d <= 11 && bl_count[d] == 0 {
                d += 1;
            }
            if d <= 11 && bl_count[d] >= 2 {
                bl_count[d] -= 2;
                bl_count[d - 1] += 1;
                sum_counts -= 1;
            } else {
                break;
            }
        }
        // Assign lengths to symbols by frequency: shortest lengths to most frequent
        let mut eff_lengths = [0u8; 256];
        let mut idx = 0usize;
        for bits in 1..=11 {
            let cnt = bl_count[bits];
            for _ in 0..cnt {
                if idx < present.len() {
                    eff_lengths[present[idx].0] = bits as u8;
                    idx += 1;
                }
            }
        }
        // If any leftover symbols, assign them to the deepest allowed
        while idx < present.len() {
            eff_lengths[present[idx].0] = 11;
            idx += 1;
        }
        lengths = eff_lengths;
        max_bits = 11;
    }

    // Compute weights for 0..=last_sym (last one present but will not be emitted in header)
    let mut weights: Vec<u8> = vec![0u8; last_sym + 1];
    for i in 0..=last_sym {
        let l = lengths[i];
        // Zstd Direct Weights: weight = (HufLog + 1) - codeLen (0 for unused)
        weights[i] = if l == 0 {
            0
        } else {
            max_bits.saturating_add(1).saturating_sub(l)
        };
        // sanity: weights must be <= 15 (we're bounded by max_bits<=11)
        if weights[i] > 15 {
            return None;
        }
    }

    // Build Direct Weights header
    // Number_of_Symbols = last_sym (weights for 0..last_sym-1 are present); headerByte = 127 + Number_of_Symbols
    // In Direct Weights header, Number_of_Symbols = last_sym + 1 (present symbols count).
    // Emit weights for symbols 0..last_sym-1 (N-1 weights); last is deduced.
    let number_of_symbols = (last_sym as u8) + 1; // <= 128 ensured above
    let header_byte = 127u8 + number_of_symbols; // 128..255
    let mut hdr: Vec<u8> = Vec::with_capacity(1 + (number_of_symbols as usize).div_ceil(2));
    hdr.push(header_byte);
    // pack weights for symbols 0..(last_sym-1), 2 per byte: first takes high nibble
    let emit_weights = last_sym; // count = number_of_symbols - 1
    for i in (0..emit_weights).step_by(2) {
        let w0 = weights[i] & 0x0F;
        let w1 = if i + 1 < emit_weights {
            weights[i + 1] & 0x0F
        } else {
            0
        };
        hdr.push((w0 << 4) | w1);
    }

    // Reconstruct the last weight from leftover so it matches decoder's rule exactly.
    // total = 1 << (HufLog + 1); sum = Σ 1<<weight[i] for i=0..last_sym-1; leftover = total - sum;
    // Require leftover to be a power of two; then weight_last = log2(leftover).
    let mut total: u32 = 1u32 << (max_bits as u32 + 1);
    let mut sum: u32 = 0;
    for i in 0..last_sym {
        let w = weights[i] as u32;
        if w > 0 {
            sum = sum.saturating_add(1u32 << w);
        }
    }
    let mut leftover = total.saturating_sub(sum);
    // If leftover is not a power of two, try increasing HufLog (up to 11) which preserves lengths
    if leftover == 0 || (leftover & (leftover - 1)) != 0 {
        let mut raised = false;
        let mut try_bits = max_bits;
        while try_bits < 11 {
            try_bits += 1;
            total = 1u32 << (try_bits as u32 + 1);
            // increasing HufLog by 1 increases each non-zero weight by 1; zeros stay 0
            let mut sum2: u32 = 0;
            for i in 0..last_sym {
                let w = if weights[i] == 0 {
                    0
                } else {
                    (weights[i] as u32) + (try_bits as u32 - max_bits as u32)
                };
                if w > 0 {
                    sum2 = sum2.saturating_add(1u32 << w);
                }
            }
            leftover = total.saturating_sub(sum2);
            if leftover != 0 && (leftover & (leftover - 1)) == 0 {
                // accept this raised HufLog; update weights accordingly
                let delta = try_bits - max_bits;
                for i in 0..=last_sym {
                    if weights[i] > 0 {
                        weights[i] = weights[i].saturating_add(delta);
                    }
                }
                max_bits = try_bits;
                sum = sum2;
                raised = true;
                break;
            }
        }
        if !raised {
            // As a last resort, try to adjust a single non-zero weight to make leftover a power of two
            let mut found_fix = false;
            'outer: for i in 0..last_sym {
                let wi = weights[i];
                if wi == 0 {
                    continue;
                }
                // try new weight values 1..=15 with valid resulting length 1..=11
                for new_w in 1u8..=15u8 {
                    if new_w == wi {
                        continue;
                    }
                    let new_len = max_bits.saturating_add(1).saturating_sub(new_w);
                    if new_len == 0 || new_len > 11 {
                        continue;
                    }
                    let sum_prime = sum - (1u32 << (wi as u32)) + (1u32 << (new_w as u32));
                    let lo = total.saturating_sub(sum_prime);
                    if lo != 0 && (lo & (lo - 1)) == 0 {
                        weights[i] = new_w;
                        // sum = sum_prime; // Not needed since we break immediately
                        leftover = lo;
                        found_fix = true;
                        break 'outer;
                    }
                }
            }
            if !found_fix {
                return None;
            }
        }
    }
    let w_last: u8 = leftover.trailing_zeros() as u8;
    if w_last > 15 {
        return None;
    }

    // Build the effective weights including the reconstructed last weight
    let mut eff_weights = weights.clone();
    eff_weights[last_sym] = w_last;

    // Derive lengths from weights: len = 0 if weight==0 else (HufLog + 1) - weight
    let mut eff_lengths = [0u8; 256];
    for i in 0..=last_sym {
        let w = eff_weights[i];
        eff_lengths[i] = if w == 0 {
            0
        } else {
            max_bits.saturating_add(1).saturating_sub(w)
        };
    }

    // Build canonical codes from eff_lengths
    let mut bl_count = [0u16; 32];
    let mut eff_max_bits = 0u8;
    for i in 0..=last_sym {
        let l = eff_lengths[i];
        if l > 0 {
            bl_count[l as usize] += 1;
            if l > eff_max_bits {
                eff_max_bits = l;
            }
        }
    }
    let mut next_code = [0u16; 32];
    let mut code: u16 = 0;
    for bits in 1..=eff_max_bits {
        code = (code + bl_count[(bits - 1) as usize]) << 1;
        next_code[bits as usize] = code;
    }
    let mut codes_vec = vec![None; 256];
    for sym in 0..=last_sym {
        let len = eff_lengths[sym];
        if len > 0 {
            let c = next_code[len as usize];
            next_code[len as usize] = c + 1;
            codes_vec[sym] = Some((c, len));
        }
    }

    Some((
        HuffmanTable {
            codes: codes_vec,
            last_symbol: last_sym,
            weights: eff_weights,
        },
        hdr,
    ))
}

pub fn build_fse_compressed_weights_header(_table: &HuffmanTable) -> Option<Vec<u8>> {
    None
}

#[inline]
pub fn reverse_bits(mut value: u32, bits: u8) -> u32 {
    let mut out = 0u32;
    for i in 0..bits {
        out |= (value & 1) << (bits - 1 - i);
        value >>= 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_huffman_for_four_stream_text() {
        let test_text = b"Four streams should be chosen for larger blocks with Huffman coding. ";
        let mut payload = Vec::new();
        while payload.len() <= 5000 {
            payload.extend_from_slice(test_text);
        }
        let maxb = *payload.iter().max().unwrap();
        assert!(maxb <= 127, "payload contains non-ASCII byte: {maxb}");
        let res = build_literals_huffman(&payload);
        assert!(
            res.is_some(),
            "build_literals_huffman should succeed for ASCII payload <=127 symbols"
        );
        let (table, header) = res.unwrap();
        assert!(!table.codes.is_empty());
        assert!(!header.is_empty());
    }
}
