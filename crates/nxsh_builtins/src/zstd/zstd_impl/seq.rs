use crate::zstd::zstd_impl::lz77::{find_matches, find_matches_with_dict};

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

/// Number of additional bits for a given Literals_Length code (RFC 3.1.1.3.2.1.1)
pub fn ll_code_num_extra_bits(code: u8) -> u8 {
	match code {
		0..=15 => 0,
		16..=19 => 1,
		20..=21 => 2,
		22..=23 => 3,
		24 => 4,
		25 => 6,
		26 => 7,
		27 => 8,
		28 => 9,
		29 => 10,
		30 => 11,
		31 => 12,
		32 => 13,
		33 => 14,
		34 => 15,
		_ => 16,
	}
}

/// Number of additional bits for a given Match_Length code (RFC 3.1.1.3.2.1.1)
pub fn ml_code_num_extra_bits(code: u8) -> u8 {
	match code {
		0..=31 => 0,
		32..=35 => 1,
		36..=37 => 2,
		38..=39 => 3,
		40..=41 => 4,
		42 => 5,
		43 => 7,
		44 => 8,
		45 => 9,
		46 => 10,
		47 => 11,
		48 => 12,
		49 => 13,
		50 => 14,
		51 => 15,
		_ => 16,
	}
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

/// Decode a Literals_Length code back to absolute length.
fn ll_code_to_len(code: u8, extra: u32) -> u32 {
	let c = code as u32;
	if c <= 15 { return c; }
	match code {
		16 => 16 + (extra & 0x1),
		17 => 18 + (extra & 0x1),
		18 => 20 + (extra & 0x1),
		19 => 22 + (extra & 0x1),
		20 => 24 + (extra & 0x3),
		21 => 28 + (extra & 0x3),
		22 => 32 + (extra & 0x7),
		23 => 40 + (extra & 0x7),
		24 => 48 + (extra & 0xF),
		25 => 64 + (extra & 0x3F),
		26 => 128 + (extra & 0x7F),
		27 => 256 + (extra & 0xFF),
		28 => 512 + (extra & 0x1FF),
		29 => 1024 + (extra & 0x3FF),
		30 => 2048 + (extra & 0x7FF),
		31 => 4096 + (extra & 0xFFF),
		32 => 8192 + (extra & 0x1FFF),
		33 => 16384 + (extra & 0x3FFF),
		34 => 32768 + (extra & 0x7FFF),
		_ => 65536 + (extra & 0xFFFF),
	}
}

/// Decode a Match_Length code back to absolute length.
fn ml_code_to_len(code: u8, extra: u32) -> u32 {
	let c = code as u32;
	if c <= 31 { return c + 3; }
	match code {
		32 => 35 + (extra & 0x1),
		33 => 37 + (extra & 0x1),
		34 => 39 + (extra & 0x1),
		35 => 41 + (extra & 0x1),
		36 => 43 + (extra & 0x3),
		37 => 47 + (extra & 0x3),
		38 => 51 + (extra & 0x7),
		39 => 59 + (extra & 0x7),
		40 => 67 + (extra & 0xF),
		41 => 83 + (extra & 0xF),
		42 => 99 + (extra & 0x1F),
		43 => 131 + (extra & 0x7F),
		44 => 259 + (extra & 0xFF),
		45 => 515 + (extra & 0x1FF),
		46 => 1027 + (extra & 0x3FF),
		47 => 2051 + (extra & 0x7FF),
		48 => 4099 + (extra & 0xFFF),
		49 => 8195 + (extra & 0x1FFF),
		50 => 16387 + (extra & 0x3FFF),
		51 => 32771 + (extra & 0x7FFF),
		_ => 65539 + (extra & 0xFFFF),
	}
}

/// Decode an Offset_Code back to match distance (non-repeat only here).
fn of_code_to_dist(of_code: u8, extra: u32) -> u32 {
	let base = 1u32 << of_code;
	let val = base + (extra & (base - 1));
	val.saturating_sub(3)
}

/// Tokenize into sequences and produce the concatenated literal stream used by sequences.
/// This is preparatory work for the Sequences section writer.
pub fn tokenize_full(input: &[u8]) -> (Vec<Seq>, Vec<u8>) {
    tokenize_full_with_dict(input, None)
}

/// Enhanced tokenize with dictionary support
pub fn tokenize_full_with_dict(input: &[u8], dict: Option<&[u8]>) -> (Vec<Seq>, Vec<u8>) {
    if input.is_empty() { return (Vec::new(), Vec::new()); }
    let matches = find_matches_with_dict(input, dict, 20, 3);
    let mut seqs = Vec::new();
    let mut literals = Vec::with_capacity(input.len());
    let mut i = 0usize;
    let mut mpos = 0usize;
    while i < input.len() {
        while mpos < matches.len() && matches[mpos].0 < i { mpos += 1; }
        if mpos >= matches.len() { break; }
        let (pos, m) = matches[mpos];
        if pos < i || m.len < 3 || m.dist <= 3 { mpos += 1; continue; }
        // Append literals preceding the match
        if pos > i { literals.extend_from_slice(&input[i..pos]); }
        let ll = (pos - i) as u32;
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
	// Tail literals after the last match
	if i < input.len() { literals.extend_from_slice(&input[i..]); }
	(seqs, literals)
}

/// Tokenize only the first eligible sequence (dist>3, len>=3) and return it with the
/// literals stream expected by Sequences section (prefix literals + tail literals after the match).
/// Returns None if no eligible sequence exists.
pub fn tokenize_first(input: &[u8]) -> Option<(Seq, Vec<u8>)> {
	if input.is_empty() { return None; }
	let matches = find_matches(input, 20, 3);
	// find first non-overlapping (trivial) match with dist>3 and len>=3
	for (pos, m) in matches.into_iter() {
		if m.len >= 3 && m.dist > 3 {
			// Build sequence
			if let Some((of_code, of_extra)) = dist_to_of_code_and_extra(m.dist) {
				let (ll_code, ll_extra) = len_to_ll_code_and_extra(pos as u32);
				let (ml_code, ml_extra) = len_to_ml_code_and_extra(m.len);
				let seq = Seq { ll_code, ll_extra, ml_code, ml_extra, of_code, of_extra };
				// literals stream = prefix literals + tail after matched region
				let mut literals = Vec::with_capacity(input.len());
				if pos > 0 { literals.extend_from_slice(&input[..pos]); }
				let after = pos + (m.len as usize);
				if after < input.len() { literals.extend_from_slice(&input[after..]); }
				return Some((seq, literals));
			}
		}
	}
	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ll_code_mapping_small() {
		for l in 0u32..=15 {
			let (code, extra) = super::len_to_ll_code_and_extra(l);
			assert_eq!(code as u32, l);
			assert_eq!(extra, 0);
		}
	}

	#[test]
	fn test_ml_code_mapping_base() {
		// lengths 3..=34 map to codes 0..=31 with 0 extra bits
		for l in 3u32..=34 {
			let (code, extra) = super::len_to_ml_code_and_extra(l);
			assert_eq!(code as u32, l - 3);
			assert_eq!(extra, 0);
		}
	}

	#[test]
	fn test_of_code_non_repeat() {
		// distance > 3 encodes as Offset_Value = dist + 3
		let dist = 5u32;
		let (code, extra) = super::dist_to_of_code_and_extra(dist).expect("non-repeat");
		// Compute expected
		let val = (dist + 3) as u64;
		let oc = 63 - val.leading_zeros() as u8;
		let base = 1u64 << oc;
		let exp_extra = (val - base) as u32;
		assert_eq!(code, oc);
		assert_eq!(extra, exp_extra);
	}

	#[test]
	fn test_tokenize_sequences_basic_non_repeat() {
		// Construct input where a non-repeat offset (>3) occurs: "abcdXabcdY"
		let input = b"abcdXabcdY"; // second "abcd" at dist=5, len=4
		let (seqs, tail) = tokenize_sequences(input);
		// We expect at least one sequence and some literal tail (the trailing 'Y')
		assert!(!seqs.is_empty());
		assert_eq!(tail.last().copied(), Some(b'Y'));
	// First sequence should have LL code for literals before match (pos 0..5 => 5)
		let s0 = seqs[0];
	assert_eq!(s0.ll_code, 5);
	// Match length code for 4 should be 1 (since 3->0, 4->1, ...)
		assert_eq!(s0.ml_code, 1);
	// Offset code for dist=5 => val=8 => code=3, extra=0
	assert_eq!(s0.of_code, 3);
	assert_eq!(s0.of_extra, 0);
	}

	#[test]
	fn test_tokenize_full_and_reconstruct_roundtrip() {
		// Pattern with a clear non-repeat match and mixed literals
		let input = b"hello-hello_world-hello!";
		let (seqs, literals) = tokenize_full(input);
		// if no sequences found, fall back trivially
		if seqs.is_empty() {
			assert_eq!(literals.as_slice(), input);
			return;
		}
		// Reconstruct using decoded lengths and offsets
		let mut out = Vec::with_capacity(input.len());
		let mut lit_pos = 0usize;
		for s in &seqs {
			let ll = ll_code_to_len(s.ll_code, s.ll_extra) as usize;
			if ll > 0 {
				out.extend_from_slice(&literals[lit_pos..lit_pos + ll]);
				lit_pos += ll;
			}
			let ml = ml_code_to_len(s.ml_code, s.ml_extra) as usize;
			let dist = of_code_to_dist(s.of_code, s.of_extra) as usize;
			// copy match from already produced output
			assert!(dist > 0 && dist <= out.len());
			let start = out.len() - dist;
			// matches can overlap; copy byte by byte
			for i in 0..ml { let b = out[start + i]; out.push(b); }
		}
		// Copy remaining last literals
		if lit_pos < literals.len() { out.extend_from_slice(&literals[lit_pos..]); }
		assert_eq!(out.as_slice(), input);
	}

	#[test]
	fn test_tokenize_first_single_seq_and_literals() {
		let input = b"abcdXabcdY"; // one clear match at dist=5 len=4 starting at pos=5
		if let Some((s, lits)) = tokenize_first(input) {
			// LL should equal 5 (prefix 'abcdX')
			assert_eq!(ll_code_to_len(s.ll_code, s.ll_extra), 5);
			// ML should be 4
			assert_eq!(ml_code_to_len(s.ml_code, s.ml_extra), 4);
			// Dist should be 5
			assert_eq!(of_code_to_dist(s.of_code, s.of_extra), 5);
			// Literals stream should be prefix + tail 'Y'
			let mut expected = Vec::new();
			expected.extend_from_slice(b"abcdX");
			expected.extend_from_slice(b"Y");
			assert_eq!(lits, expected);
		} else {
			panic!("expected one sequence");
		}
	}
}
