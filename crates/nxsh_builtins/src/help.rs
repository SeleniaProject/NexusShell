use tabled::{Table, Tabled};
use ansi_term::Colour;
use pulldown_cmark::{Parser, Options, html};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Tabled)]
struct HelpEntry {
    command: &'static str,
    description_ja: &'static str,
    description_en: &'static str,
}

static HELP_ENTRIES: &[HelpEntry] = &[
    HelpEntry { command: "cd", description_ja: "ディレクトリを変更", description_en: "Change directory" },
    HelpEntry { command: "history", description_ja: "コマンド履歴を表示", description_en: "Show command history" },
    HelpEntry { command: "help", description_ja: "ヘルプを表示", description_en: "Display help" },
    HelpEntry { command: "fg", description_ja: "ジョブをフォアグラウンド", description_en: "Bring job to foreground" },
    HelpEntry { command: "bg", description_ja: "ジョブをバックグラウンド", description_en: "Resume job in background" },
];

pub fn help_cli(args: &[String]) -> Result<()> {
    if args.is_empty() {
        print_general_help();
    } else if args[0] == "--lang" {
        // example: help --lang ja
        if args.len() < 2 { anyhow::bail!("--lang requires parameter"); }
        let lang = &args[1];
        print_general_help_lang(lang);
    } else {
        let cmd = &args[0];
        print_command_help(cmd);
    }
    Ok(())
}

fn print_general_help() {
    let lang = detect_lang();
    print_general_help_lang(&lang);
}

fn print_general_help_lang(lang: &str) {
    let rows: Vec<_> = HELP_ENTRIES.iter().map(|e| {
        if lang.starts_with("ja") {
            (e.command, e.description_ja)
        } else {
            (e.command, e.description_en)
        }
    }).collect();

    let table = Table::new(rows).to_string();
    println!("{}", table);
}

fn print_command_help(cmd: &str) {
    if let Some(entry) = HELP_ENTRIES.iter().find(|e| e.command == cmd) {
        let lang = detect_lang();
        let desc = if lang.starts_with("ja") { entry.description_ja } else { entry.description_en };
        println!("{}: {}", Colour::Green.paint(cmd), desc);
        // Attempt to render markdown manpage from docs/man/{cmd}.md
        let path = format!("docs/man/{}.md", cmd);
        if let Ok(markdown) = std::fs::read_to_string(&path) {
            let parser = Parser::new_ext(&markdown, Options::all());
            let mut html_buf = String::new();
            html::push_html(&mut html_buf, parser);
            // Strip HTML tags for ANSI output (simple)
            let text = html2text::from_read(html_buf.as_bytes(), 80);
            println!("{}", text);
        }
    } else {
        println!("No help for command {}", cmd);
    }
}

fn detect_lang() -> String {
    std::env::var("LANG").unwrap_or_else(|_| "en".into())
} 