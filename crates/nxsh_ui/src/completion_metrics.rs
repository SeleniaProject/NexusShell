//! 補完レイチE��シ測定ユーチE��リチE��
use std::time::Instant;

pub struct CompletionTimer {
    start: Instant,
}

impl CompletionTimer {
    pub fn start() -> Self {
        CompletionTimer { start: Instant::now() }
    }
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

/// 補完�E琁E�E侁E
pub fn measure_completion<F, T>(f: F) -> (T, f64)
where F: FnOnce() -> T {
    let timer = CompletionTimer::start();
    let result = f();
    let ms = timer.elapsed_ms();
    (result, ms)
}

