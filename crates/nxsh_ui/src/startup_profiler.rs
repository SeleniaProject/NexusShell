use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Global startup profiler for measuring early UI milestones.
/// Enabled when environment variable `NXSH_MEASURE_STARTUP=1` is set
/// or the CLI passed `--measure-startup` (which sets the env var).
#[derive(Default)]
pub struct StartupProfiler {
    /// Process start time provided by the CLI entry.
    start_cli: Option<Instant>,
    /// Time when CUI initialization completed.
    cui_init_done: Option<Instant>,
    /// Time when the first splash/banner frame was flushed to the terminal.
    first_frame_flushed: Option<Instant>,
    /// Time when the first prompt was flushed to the terminal.
    first_prompt_flushed: Option<Instant>,
}

// Default is derived

static PROFILER: OnceLock<Mutex<StartupProfiler>> = OnceLock::new();

fn get_profiler() -> &'static Mutex<StartupProfiler> {
    PROFILER.get_or_init(|| Mutex::new(StartupProfiler::default()))
}

/// Returns true if startup measurement is enabled via environment.
pub fn is_enabled() -> bool {
    std::env::var("NXSH_MEASURE_STARTUP")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Initialize the profiler with the CLI start instant.
pub fn init_with_cli_start(start: Instant) {
    if !is_enabled() {
        return;
    }
    let mut p = get_profiler().lock().expect("startup profiler lock");
    p.start_cli = Some(start);
}

/// Mark that CUI initialization finished.
pub fn mark_cui_init_done(now: Instant) {
    if !is_enabled() {
        return;
    }
    let mut p = get_profiler().lock().expect("startup profiler lock");
    if p.cui_init_done.is_none() {
        p.cui_init_done = Some(now);
    }
}

/// Mark that the first frame (splash/banner) has been flushed.
pub fn mark_first_frame_flushed(now: Instant) {
    if !is_enabled() {
        return;
    }
    let mut p = get_profiler().lock().expect("startup profiler lock");
    if p.first_frame_flushed.is_none() {
        p.first_frame_flushed = Some(now);
        // When first frame is flushed and we have the CLI start, print an immediate summary.
        drop(p);
        print_partial_summary();
    }
}

/// Mark that the first prompt has been flushed.
pub fn mark_first_prompt_flushed(now: Instant) {
    if !is_enabled() {
        return;
    }
    let mut p = get_profiler().lock().expect("startup profiler lock");
    if p.first_prompt_flushed.is_none() {
        p.first_prompt_flushed = Some(now);
        // Print a complete summary once prompt is ready.
        drop(p);
        print_complete_summary();
    }
}

fn compute_ms(from: Option<Instant>, to: Option<Instant>) -> Option<u128> {
    match (from, to) {
        (Some(a), Some(b)) => Some(b.duration_since(a).as_millis()),
        _ => None,
    }
}

/// Minimal bilingual output helper.
fn choose<'a>(en: &'a str, ja: &'a str) -> &'a str {
    let lang = std::env::var("LANG").unwrap_or_default().to_ascii_lowercase();
    if lang.starts_with("ja") { ja } else { en }
}

/// Print a summary up to first-frame milestone.
fn print_partial_summary() {
    if !is_enabled() { return; }
    let p = get_profiler().lock().expect("startup profiler lock");
    let t_frame = compute_ms(p.start_cli, p.first_frame_flushed);
    let threshold_ms: u128 = 16;
    if let Some(ms) = t_frame {
        let pass = if ms <= threshold_ms { "PASS" } else { "FAIL" };
        let label = choose(
            "Startup(first-frame)",
            "起動(初回フレーム)"
        );
        let note = choose(
            "threshold 16ms",
            "しきい値 16ms"
        );
        eprintln!(
            "{label}: {ms}ms [{pass}] ({note})"
        );
    }
}

/// Print a complete summary including init and prompt milestones.
fn print_complete_summary() {
    if !is_enabled() { return; }
    let p = get_profiler().lock().expect("startup profiler lock");
    let t_init = compute_ms(p.start_cli, p.cui_init_done);
    let t_frame = compute_ms(p.start_cli, p.first_frame_flushed);
    let t_prompt = compute_ms(p.start_cli, p.first_prompt_flushed);
    let threshold_ms: u128 = 16;
    let (label_total, label_init, label_frame, label_prompt, label_note) = (
        choose("Startup total", "起動 合計"),
        choose("Init", "初期化"),
        choose("First frame", "初回フレーム"),
        choose("First prompt", "初回プロンプト"),
        choose("threshold 16ms", "しきい値 16ms"),
    );

    // Pick the latest known milestone as current total.
    let total_ms = p.first_prompt_flushed
        .or(p.first_frame_flushed)
        .or(p.cui_init_done)
        .and_then(|t| p.start_cli.map(|s| t.duration_since(s).as_millis()));

    if let Some(total) = total_ms {
        let pass = if total <= threshold_ms { "PASS" } else { "FAIL" };
        eprintln!("{label_total}: {total}ms [{pass}] ({label_note})");
    }
    if let Some(v) = t_init { eprintln!("  {label_init}:  {v}ms"); }
    if let Some(v) = t_frame { eprintln!("  {label_frame}: {v}ms"); }
    if let Some(v) = t_prompt { eprintln!("  {label_prompt}: {v}ms"); }
}


