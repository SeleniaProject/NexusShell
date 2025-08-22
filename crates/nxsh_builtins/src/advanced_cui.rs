// Temporary shim for advanced CUI used by beautiful UI

pub struct AdvancedCUI;

impl AdvancedCUI {
    pub fn new() -> Self { AdvancedCUI }
    pub fn render(&self, _content: &str) {
        // minimal rendering stub
        println!("[AdvancedCUI render]");
    }
}
