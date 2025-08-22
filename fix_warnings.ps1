# スクリプトで未使用インポートの警告を修正

$fixes = @{
    "crates\nxsh_builtins\src\uniq.rs" = @{
        "use crate::ui_design::{TableFormatter, ColorPalette, Icons, Colorize};" = "use crate::ui_design::{ColorPalette};"
    }
    "crates\nxsh_builtins\src\head.rs" = @{
        "    TableFormatter, ColorPalette, Icons, Colorize, ProgressBar, Animation," = "    ColorPalette, Icons,"
        "    TableOptions, BorderStyle, Alignment, Notification, NotificationType" = ""
        "use std::time::{Duration, Instant};" = ""
        "use std::thread;" = ""
    }
    "crates\nxsh_builtins\src\tail.rs" = @{
        "    TableFormatter, ColorPalette, Icons, Colorize, ProgressBar, Animation," = "    ColorPalette, Icons,"
        "    TableOptions, BorderStyle, Alignment, Notification, NotificationType" = ""
        "use std::time::Instant;" = ""
    }
    "crates\nxsh_builtins\src\wc.rs" = @{
        "    TableFormatter, Colorize, ProgressBar, Animation, TableOptions, BorderStyle," = "    Colorize,"
        "    Alignment, Notification, NotificationType" = ""
        "use std::time::{Duration, Instant};" = ""
        "use std::thread;" = ""
        "use std::io::{self, Read, Write};" = "use std::io::{self, Read};"
    }
    "crates\nxsh_builtins\src\cut.rs" = @{
        "use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};" = "use super::ui_design::{ColorPalette};"
    }
    "crates\nxsh_builtins\src\sort.rs" = @{
        "use crate::ui_design::{TableFormatter, ColorPalette, Icons, Colorize};" = "use crate::ui_design::{ColorPalette, Icons, Colorize};"
    }
    "crates\nxsh_builtins\src\awk.rs" = @{
        "use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};" = ""
    }
    "crates\nxsh_builtins\src\sed.rs" = @{
        "use super::ui_design::{Colorize, TableFormatter, ColorPalette, Icons};" = ""
    }
    "crates\nxsh_builtins\src\tr.rs" = @{
        "use crate::ui_design::{TableFormatter, ColorPalette, Icons, Colorize};" = "use crate::ui_design::{ColorPalette, Colorize};"
    }
}

foreach ($file in $fixes.Keys) {
    if (Test-Path $file) {
        $content = Get-Content $file -Raw
        foreach ($search in $fixes[$file].Keys) {
            $replace = $fixes[$file][$search]
            $content = $content -replace [regex]::Escape($search), $replace
        }
        Set-Content -Path $file -Value $content
        Write-Host "Fixed $file"
    }
}
