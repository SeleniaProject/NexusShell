//! NexusShell 起動時間測定ユーティリティ
use std::time::{Instant, Duration};

/// 起動時間測定用グローバル
static mut START_TIME: Option<Instant> = None;

/// 起動時に呼び出す
pub fn record_start_time() {
    unsafe {
        START_TIME = Some(Instant::now());
    }
}

/// 起動完了時に呼び出して経過時間を返す
pub fn get_startup_duration() -> Option<Duration> {
    unsafe {
        START_TIME.map(|t| t.elapsed())
    }
}

/// CLI表示用
pub fn print_startup_time() {
    if let Some(duration) = get_startup_duration() {
        println!("起動時間: {:.3}ms", duration.as_secs_f64() * 1000.0);
    }
}
