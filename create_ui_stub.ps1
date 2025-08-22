# 最小限の動作するui_designスタブを作成

$uiFile = "crates\nxsh_builtins\src\ui_design_stub.rs"
$content = @"
// Temporary stub for ui_design.rs to avoid encoding issues

use std::io::{self, Write};

/// Basic string colorization trait  
pub trait Colorize {
    fn primary(self) -> String;
    fn success(self) -> String;
    fn warning(self) -> String;
    fn error(self) -> String;
    fn info(self) -> String;
    fn bright_cyan(self) -> String;
    fn bright_green(self) -> String;
}

impl<T: AsRef<str>> Colorize for T {
    fn primary(self) -> String { self.as_ref().to_string() }
    fn success(self) -> String { self.as_ref().to_string() }
    fn warning(self) -> String { self.as_ref().to_string() }
    fn error(self) -> String { self.as_ref().to_string() }
    fn info(self) -> String { self.as_ref().to_string() }
    fn bright_cyan(self) -> String { self.as_ref().to_string() }
    fn bright_green(self) -> String { self.as_ref().to_string() }
}

pub struct ColorPalette;
impl ColorPalette {
    pub const INFO: &'static str = "";
}

pub struct Icons;
impl Icons {
    pub fn new() -> Self { Self }
}

pub struct ItemStatus;
pub struct StatusItem;
pub struct StatusDashboard;
pub struct DashboardSection;
pub struct SectionStyle;
pub struct CommandWizard;
pub struct WizardStep;
pub struct InputType;

pub fn create_advanced_table() {}

pub struct TableOptions;
pub struct BorderStyle;
pub struct Alignment;
"@

Set-Content -Path $uiFile -Value $content

# lib.rsでui_designをスタブに置き換え
$libFile = "crates\nxsh_builtins\src\lib.rs"
$content = Get-Content $libFile -Raw
$content = $content -replace "pub mod ui_design;", "pub mod ui_design_stub as ui_design;"
Set-Content -Path $libFile -Value $content

Write-Host "Created ui_design stub to avoid encoding issues"
