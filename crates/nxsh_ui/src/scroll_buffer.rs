use std::collections::VecDeque;

const MAX_LINES: usize = 100_000;

#[derive(Default)]
pub struct ScrollBuffer {
    lines: VecDeque<String>,
    offset: usize,
}

impl ScrollBuffer {
    /// Create a new buffer; size is currently advisory and ignored.
    pub fn new(_size: usize) -> Self {
        Self::default()
    }

    /// Push a new line into the buffer, evicting old lines when over capacity.
    pub fn push(&mut self, line: impl Into<String>) {
        if self.lines.len() == MAX_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(line.into());
    }

    /// Back-compat helper used by some UI paths.
    pub fn add_line(&mut self, line: impl Into<String>) {
        self.push(line);
    }

    /// Scroll by delta lines. Positive = down, negative = up.
    pub fn scroll(&mut self, delta: isize) {
        let len = self.lines.len();
        let new_offset = self.offset as isize + delta;
        self.offset = new_offset.clamp(0, (len as isize).saturating_sub(1)) as usize;
    }

    /// Return an iterator over visible lines starting at current offset.
    pub fn view(&self, height: usize) -> impl Iterator<Item = &str> {
        let start = self.offset.min(self.lines.len());
        let end = (start + height).min(self.lines.len());
        self.lines.iter().skip(start).take(end - start).map(String::as_str)
    }

    /// Collect visible lines into a Vec<String> for rendering layers that prefer owned strings.
    pub fn get_visible_lines(&self, height: usize) -> Vec<String> {
        self.view(height).map(|s| s.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_scroll() {
        let mut buf = ScrollBuffer::default();
        for i in 0..200 {
            buf.push(format!("line {}", i));
        }
        buf.scroll(-5);
        assert_eq!(buf.view(1).next().expect("line should exist"), "line 195");
    }
} 