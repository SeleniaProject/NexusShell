use crate::zstd::zstd_impl::lz77::{find_matches, Match};

#[derive(Debug, Clone, Copy)]
pub struct Seq {
	// Code-space values (not the raw lengths). Additional bits are computed from *_extra fields when encoding.
	pub ll_code: u8,
	pub ll_extra: u32,
	pub ml_code: u8,
	pub ml_extra: u32,
	pub of_code: u8,
	pub of_extra: u32,
}

fn len_to_ll_code_and_extra(len: u32) -> (u8, u32) {
	// RFC 8878 3.1.1.3.2.1.1 Literals Length Codes
	// 0..15 -> length = code, 0 extra bits
	// 16 -> 16 + 1b
	// 17 -> 18 + 1b
	// 18 -> 20 + 1b
	// 19 -> 22 + 1b
	// 20 -> 24 + 2b
	// 21 -> 28 + 2b
	// 22 -> 32 + 3b
	// 23 -> 40 + 3b
	// 24 -> 48 + 4b
	// 25 -> 64 + 6b
	// 26 -> 128 + 7b
	// 27 -> 256 + 8b
	// 28 -> 512 + 9b
	// 29 -> 1024 + 10b
	// 30 -> 2048 + 11b
	// 31 -> 4096 + 12b
	// 32 -> 8192 + 13b
	// 33 -> 16384 + 14b
	// 34 -> 32768 + 15b
	// 35 -> 65536 + 16b
	let l = len as u64;
	if l <= 15 { return (l as u8, 0); }
	// Table of (code, base, nbits)
	const MAP: &[(u8,u64,u8)] = &[
		(16, 16, 1), (17, 18, 1), (18, 20, 1), (19, 22, 1),
		(20, 24, 2), (21, 28, 2), (22, 32, 3), (23, 40, 3),
		(24, 48, 4), (25, 64, 6), (26, 128, 7), (27, 256, 8),
		(28, 512, 9), (29, 1024, 10), (30, 2048, 11), (31, 4096, 12),
		(32, 8192, 13), (33, 16384, 14), (34, 32768, 15), (35, 65536, 16),
	];
	for &(code, base, nbits) in MAP {
		let maxv = base + ((1u64 << nbits) - 1);
		if l >= base && l <= maxv {
			return (code, (l - base) as u32);
		}
	}
	// Clamp very large -> 35
	let nbits = 16;
	let base = 65536u64;
	(35, (l.saturating_sub(base)) as u32 & ((1u32<<nbits)-1))
}

fn len_to_ml_code_and_extra(len: u32) -> (u8, u32) {
	// RFC 8878 3.1.1.3.2.1.1 Match Length Codes (lengths are code+3 for 0..31)
	let l = len as u64;
	if (3..=34).contains(&l) { return ((l as u8) - 3, 0); }
	// Table (code, base, nbits)
	const MAP: &[(u8,u64,u8)] = &[
		(32, 35, 1), (33, 37, 1), (34, 39, 1), (35, 41, 1),
		(36, 43, 2), (37, 47, 2), (38, 51, 3), (39, 59, 3),
		(40, 67, 4), (41, 83, 4), (42, 99, 5), (43, 131, 7),
		(44, 259, 8), (45, 515, 9), (46, 1027, 10), (47, 2051, 11),
		(48, 4099, 12), (49, 8195, 13), (50, 16387, 14), (51, 32771, 15), (52, 65539, 16),
	];
	for &(code, base, nbits) in MAP {
		let maxv = base + ((1u64 << nbits) - 1);
		if l >= base && l <= maxv {
			return (code, (l - base) as u32);
		}
	}
	// Clamp
	(52, (l.saturating_sub(65539)) as u32 & ((1u32<<16)-1))
}

fn dist_to_of_code_and_extra(dist: u32) -> Option<(u8, u32)> {
	// We only handle non-repeat offsets (>3). Encode Offset_Value = dist + 3
	if dist <= 3 { return None; }
	let val = (dist as u64) + 3;
	let oc = 63 - val.leading_zeros() as u8; // floor(log2(val))
	let base = 1u64 << oc;
	let extra = (val - base) as u32;
	Some((oc, extra))
}

/// Convert matches into a simplified sequence stream with leftover literals.
/// This greedy pass picks non-overlapping matches with dist > 3 to avoid repeat codes for now.
pub fn tokenize_sequences(input: &[u8]) -> (Vec<Seq>, Vec<u8>) {
	if input.is_empty() { return (Vec::new(), Vec::new()); }
	let matches = find_matches(input, 20, 3);
	let mut seqs = Vec::new();
	let mut lit_tail = Vec::new();
	let mut i = 0usize;
	let mut mpos = 0usize;
	while i < input.len() {
		// Advance to next non-overlapping match with dist>3 and len>=3
		while mpos < matches.len() && matches[mpos].0 < i { mpos += 1; }
		if mpos >= matches.len() { break; }
		let (pos, m) = matches[mpos];
		if pos < i { mpos += 1; continue; }
		if m.len < 3 || m.dist <= 3 { mpos += 1; continue; }
		// literals from i..pos
		let ll = (pos - i) as u32;
		// build sequence
		if let Some((of_code, of_extra)) = dist_to_of_code_and_extra(m.dist) {
			let (ll_code, ll_extra) = len_to_ll_code_and_extra(ll);
			let (ml_code, ml_extra) = len_to_ml_code_and_extra(m.len);
			seqs.push(Seq { ll_code, ll_extra, ml_code, ml_extra, of_code, of_extra });
			i = pos + (m.len as usize);
			mpos += 1;
		} else {
			mpos += 1;
		}
	}
	// leftover literals from i..end
	if i < input.len() { lit_tail.extend_from_slice(&input[i..]); }
	(seqs, lit_tail)
}
