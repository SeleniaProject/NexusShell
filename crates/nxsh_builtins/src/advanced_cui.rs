// Temporary shim for advanced CUI used by beautiful UI

pub struct AdvancedCUI;

impl Default for AdvancedCUI {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvancedCUI {
    pub fn new() -> Self {
        AdvancedCUI
    }
    pub fn render(&self, _content: &str) {
        // minimal rendering stub
        println!("[AdvancedCUI render]");
    }
}
