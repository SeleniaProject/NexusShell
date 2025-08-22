//! 補完デモンストレーション - NexusShell Completion Demo
//!
//! このモジュールは、NexusShellの補完システムのデモンストレーションと
//! テスト用の実装を提供します。開発者向けの学習教材として活用可能。

use std::collections::HashMap;
use std::io::{self, Write, stdout, stderr};
use std::time::{Duration, Instant};
use anyhow::{Result, Error};
use crossterm::{
    cursor, event, execute, queue, style, terminal,
    event::{Event, KeyCode, KeyEvent},
    style::{Color, Stylize},
    terminal::{ClearType, disable_raw_mode, enable_raw_mode},
};

use nxsh_ui::completion::{CompletionType, CompletionResult, NexusCompleter};
use nxsh_ui::config::UiConfig;
use nxsh_ui::themes::{Theme, get_theme};

/// 補完デモアプリケーション
pub struct CompletionDemo {
    completer: NexusCompleter,
    theme: Theme,
    input_buffer: String,
    cursor_position: usize,
    current_completions: Option<CompletionResult>,
    completion_index: usize,
    demo_scenarios: Vec<DemoScenario>,
    current_scenario: usize,
    is_running: bool,
}

/// デモシナリオ
#[derive(Debug, Clone)]
pub struct DemoScenario {
    pub name: String,
    pub description: String,
    pub initial_input: String,
    pub expected_completions: Vec<String>,
    pub demonstration_text: Vec<String>,
}

impl CompletionDemo {
    /// 新しい補完デモを作成
    pub fn new() -> Result<Self> {
        let config = UiConfig::default();
        let completer = NexusCompleter::new();
        let theme = get_theme(&config.theme_name)?;

        let demo_scenarios = Self::create_demo_scenarios();

        Ok(Self {
            completer,
            theme,
            input_buffer: String::new(),
            cursor_position: 0,
            current_completions: None,
            completion_index: 0,
            demo_scenarios,
            current_scenario: 0,
            is_running: false,
        })
    }

    /// デモシナリオを作成
    fn create_demo_scenarios() -> Vec<DemoScenario> {
        vec![
            DemoScenario {
                name: "基本コマンド補完".to_string(),
                description: "基本的なシェルコマンドの補完を実演します".to_string(),
                initial_input: "l".to_string(),
                expected_completions: vec!["ls".to_string(), "less".to_string(), "ln".to_string()],
                demonstration_text: vec![
                    "デモ1: 基本コマンド補完".to_string(),
                    "「l」と入力してTabキーを押すと、lで始まるコマンドが補完候補として表示されます。".to_string(),
                    "補完候補: ls, less, ln, locate など".to_string(),
                ],
            },
            DemoScenario {
                name: "ファイルパス補完".to_string(),
                description: "ファイルとディレクトリの補完を実演します".to_string(),
                initial_input: "./".to_string(),
                expected_completions: vec!["./src/".to_string(), "./target/".to_string(), "./Cargo.toml".to_string()],
                demonstration_text: vec![
                    "デモ2: ファイルパス補完".to_string(),
                    "「./」と入力してTabキーを押すと、現在のディレクトリの内容が表示されます。".to_string(),
                    "ディレクトリは「/」で終わり、ファイルはそのまま表示されます。".to_string(),
                ],
            },
            DemoScenario {
                name: "オプション補完".to_string(),
                description: "コマンドオプションの補完を実演します".to_string(),
                initial_input: "ls --".to_string(),
                expected_completions: vec!["--all".to_string(), "--long".to_string(), "--help".to_string()],
                demonstration_text: vec![
                    "デモ3: オプション補完".to_string(),
                    "「ls --」と入力してTabキーを押すと、lsコマンドのオプションが表示されます。".to_string(),
                    "利用可能なオプション: --all, --long, --help, --human-readable など".to_string(),
                ],
            },
            DemoScenario {
                name: "履歴補完".to_string(),
                description: "コマンド履歴からの補完を実演します".to_string(),
                initial_input: "git st".to_string(),
                expected_completions: vec!["git status".to_string(), "git stash".to_string()],
                demonstration_text: vec![
                    "デモ4: 履歴補完".to_string(),
                    "「git st」と入力してTabキーを押すと、履歴から関連するコマンドが提案されます。".to_string(),
                    "頻繁に使用されるコマンドほど上位に表示されます。".to_string(),
                ],
            },
            DemoScenario {
                name: "スマート補完".to_string(),
                description: "文脈を理解したスマート補完を実演します".to_string(),
                initial_input: "cd ".to_string(),
                expected_completions: vec!["src/".to_string(), "target/".to_string(), "../".to_string()],
                demonstration_text: vec![
                    "デモ5: スマート補完".to_string(),
                    "「cd 」と入力してTabキーを押すと、ディレクトリのみが補完候補として表示されます。".to_string(),
                    "コマンドの性質を理解して、適切な補完を提供します。".to_string(),
                ],
            },
        ]
    }

    /// デモを実行
    pub async fn run_demo(&mut self) -> Result<()> {
        self.initialize_terminal()?;
        self.is_running = true;

        // スプラッシュ画面を表示
        self.show_splash_screen()?;
        
        // 各シナリオを実行
        while self.is_running && self.current_scenario < self.demo_scenarios.len() {
            self.run_scenario(self.current_scenario).await?;
            self.current_scenario += 1;
        }

        // 終了画面を表示
        self.show_conclusion()?;
        
        self.cleanup_terminal()?;
        Ok(())
    }

    /// ターミナルを初期化
    fn initialize_terminal(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            terminal::Clear(ClearType::All),
            cursor::Hide
        )?;
        Ok(())
    }

    /// ターミナルをクリーンアップ
    fn cleanup_terminal(&mut self) -> Result<()> {
        execute!(
            stdout(),
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        disable_raw_mode()?;
        Ok(())
    }

    /// スプラッシュ画面を表示
    fn show_splash_screen(&self) -> Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        let splash_text = vec![
            "╔══════════════════════════════════════════════════════════════════╗",
            "║                                                                  ║",
            "║                    NexusShell 補完システム                        ║",
            "║                      Completion Demo                             ║",
            "║                                                                  ║",
            "║   このデモでは、NexusShellの強力な補完システムを実演します。      ║",
            "║   各シナリオで異なる補完機能を体験できます。                      ║",
            "║                                                                  ║",
            "║   操作方法:                                                      ║",
            "║   - Tab: 補完を実行                                              ║",
            "║   - ↑/↓: 補完候補を選択                                          ║",
            "║   - Enter: 選択した補完を適用                                     ║",
            "║   - Esc: 補完をキャンセル                                        ║",
            "║   - Ctrl+C: デモを終了                                           ║",
            "║                                                                  ║",
            "║               Enterキーを押して開始してください                   ║",
            "║                                                                  ║",
            "╚══════════════════════════════════════════════════════════════════╝",
        ];

        for (i, line) in splash_text.iter().enumerate() {
            execute!(stdout(), cursor::MoveTo(5, 3 + i as u16))?;
            print!("{}", line.with(self.theme.colors.primary));
        }

        stdout().flush()?;
        self.wait_for_enter()?;
        Ok(())
    }

    /// Enterキーを待機
    fn wait_for_enter(&self) -> Result<()> {
        loop {
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => break,
                    Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers, .. }) 
                        if modifiers.contains(event::KeyModifiers::CONTROL) => {
                        return Err(Error::msg("User cancelled"));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// シナリオを実行
    async fn run_scenario(&mut self, scenario_index: usize) -> Result<()> {
        let scenario = if let Some(scenario) = self.demo_scenarios.get(scenario_index) {
            scenario.clone()
        } else {
            return Ok(());
        };

        execute!(stdout(), terminal::Clear(ClearType::All))?;

        // シナリオ情報を表示
        self.display_scenario_info(&scenario)?;

        // 初期入力を設定
        self.input_buffer = scenario.initial_input.clone();
        self.cursor_position = self.input_buffer.len();

        // インタラクティブセッションを開始
        self.run_interactive_session(&scenario).await?;

        // 次のシナリオへの移行を待機
        self.wait_for_next_scenario()?;

        Ok(())
    }

    /// シナリオ情報を表示
    fn display_scenario_info(&self, scenario: &DemoScenario) -> Result<()> {
        execute!(stdout(), cursor::MoveTo(0, 0))?;

        // ヘッダー
        let header = format!("═══ {} ═══", scenario.name);
        println!("{}", header.with(self.theme.colors.secondary));
        println!();

        // 説明
        println!("{}", scenario.description.with(self.theme.colors.text));
        println!();

        // デモンストレーションテキスト
        for line in &scenario.demonstration_text {
            println!("{}", line.with(self.theme.colors.comment));
        }
        println!();

        println!("{}", "─".repeat(70).with(self.theme.colors.border));
        println!();

        stdout().flush()?;
        Ok(())
    }

    /// インタラクティブセッションを実行
    async fn run_interactive_session(&mut self, scenario: &DemoScenario) -> Result<()> {
        let mut session_active = true;

        while session_active {
            // プロンプトと入力を表示
            self.display_prompt_and_input()?;

            // ユーザー入力を処理
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key_event) => {
                        match self.handle_key_event(key_event).await? {
                            DemoAction::Continue => {}
                            DemoAction::NextScenario => {
                                session_active = false;
                            }
                            DemoAction::Exit => {
                                self.is_running = false;
                                session_active = false;
                            }
                            DemoAction::TriggerCompletion => {
                                self.trigger_completion().await?;
                            }
                            DemoAction::ApplyCompletion => {
                                self.apply_completion();
                            }
                            DemoAction::CancelCompletion => {
                                self.cancel_completion();
                            }
                        }
                    }
                    _ => {}
                }
            }

            // 補完パネルを表示
            if let Some(completions) = &self.current_completions {
                self.display_completion_panel(completions)?;
            }
        }

        Ok(())
    }

    /// プロンプトと入力を表示
    fn display_prompt_and_input(&self) -> Result<()> {
        execute!(stdout(), cursor::MoveTo(0, 10))?;
        print!("{}", "$ ".with(self.theme.colors.prompt));
        print!("{}", self.input_buffer);

        // カーソル位置を設定
        execute!(stdout(), cursor::MoveTo((2 + self.cursor_position) as u16, 10))?;
        stdout().flush()?;
        Ok(())
    }

    /// キーイベントを処理
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<DemoAction> {
        match key_event.code {
            KeyCode::Tab => {
                Ok(DemoAction::TriggerCompletion)
            }
            KeyCode::Enter => {
                if self.current_completions.is_some() {
                    Ok(DemoAction::ApplyCompletion)
                } else {
                    Ok(DemoAction::NextScenario)
                }
            }
            KeyCode::Esc => {
                if self.current_completions.is_some() {
                    Ok(DemoAction::CancelCompletion)
                } else {
                    Ok(DemoAction::NextScenario)
                }
            }
            KeyCode::Up => {
                self.select_previous_completion();
                Ok(DemoAction::Continue)
            }
            KeyCode::Down => {
                self.select_next_completion();
                Ok(DemoAction::Continue)
            }
            KeyCode::Char('c') if key_event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                Ok(DemoAction::Exit)
            }
            KeyCode::Char(c) => {
                self.insert_char(c);
                Ok(DemoAction::Continue)
            }
            KeyCode::Backspace => {
                self.delete_backward();
                Ok(DemoAction::Continue)
            }
            _ => Ok(DemoAction::Continue),
        }
    }

    /// 補完を実行
    async fn trigger_completion(&mut self) -> Result<()> {
        let completions = self.completion_system.get_intelligent_completions(
            &self.input_buffer,
            self.cursor_position,
        )?;

        if !completions.items.is_empty() {
            self.current_completions = Some(completions);
            self.completion_index = 0;
        }

        Ok(())
    }

    /// 補完パネルを表示
    fn display_completion_panel(&self, completions: &CompletionResult) -> Result<()> {
        let start_row = 12;
        let max_items = 10;

        execute!(stdout(), cursor::MoveTo(0, start_row))?;
        println!("{}", "補完候補:".with(self.theme.colors.secondary));

        for (i, item) in completions.items.iter().take(max_items).enumerate() {
            let row = start_row + 1 + i as u16;
            execute!(stdout(), cursor::MoveTo(2, row))?;

            let prefix = if i == self.completion_index { "► " } else { "  " };
            let type_indicator = match item.completion_type {
                CompletionType::Command => "⚡",
                CompletionType::File => "📄",
                CompletionType::Directory => "📁",
                CompletionType::Variable => "🔧",
                CompletionType::Option => "⚙️",
                _ => "•",
            };

            let line = format!(
                "{}{} {} - {}",
                prefix,
                type_indicator,
                item.text,
                item.description.as_deref().unwrap_or("No description")
            );

            if i == self.completion_index {
                print!("{}", line.with(self.theme.colors.completion_highlight));
            } else {
                print!("{}", line.with(self.theme.colors.text));
            }
        }

        if completions.items.len() > max_items {
            let row = start_row + 1 + max_items as u16;
            execute!(stdout(), cursor::MoveTo(2, row))?;
            print!("{}", format!("... and {} more", completions.items.len() - max_items)
                .with(self.theme.colors.comment));
        }

        stdout().flush()?;
        Ok(())
    }

    /// 補完候補選択
    fn select_next_completion(&mut self) {
        if let Some(completions) = &self.current_completions {
            if !completions.items.is_empty() {
                self.completion_index = (self.completion_index + 1) % completions.items.len();
            }
        }
    }

    fn select_previous_completion(&mut self) {
        if let Some(completions) = &self.current_completions {
            if !completions.items.is_empty() {
                self.completion_index = if self.completion_index > 0 {
                    self.completion_index - 1
                } else {
                    completions.items.len() - 1
                };
            }
        }
    }

    /// 補完を適用
    fn apply_completion(&mut self) {
        if let Some(completions) = &self.current_completions {
            if let Some(item) = completions.items.get(self.completion_index) {
                // 現在の単語を補完で置換
                let prefix_len = completions.prefix.len();
                let start_pos = self.cursor_position.saturating_sub(prefix_len);
                
                self.input_buffer.drain(start_pos..self.cursor_position);
                self.input_buffer.insert_str(start_pos, &item.text);
                self.cursor_position = start_pos + item.text.len();
            }
        }
        self.cancel_completion();
    }

    /// 補完をキャンセル
    fn cancel_completion(&mut self) {
        self.current_completions = None;
        self.completion_index = 0;
    }

    /// 文字挿入
    fn insert_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.cancel_completion(); // 入力が変更されたら補完をクリア
    }

    /// 後方削除
    fn delete_backward(&mut self) {
        if self.cursor_position > 0 {
            self.input_buffer.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.cancel_completion();
        }
    }

    /// 次のシナリオへの移行を待機
    fn wait_for_next_scenario(&self) -> Result<()> {
        execute!(stdout(), cursor::MoveTo(0, 25))?;
        println!("{}", "次のデモに進むにはEnterキーを押してください（Ctrl+Cで終了）"
            .with(self.theme.colors.prompt));
        stdout().flush()?;

        self.wait_for_enter()?;
        Ok(())
    }

    /// 終了画面を表示
    fn show_conclusion(&self) -> Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        let conclusion_text = vec![
            "╔══════════════════════════════════════════════════════════════════╗",
            "║                                                                  ║",
            "║                        デモ完了！                                ║",
            "║                                                                  ║",
            "║   NexusShellの補完システムの機能を体験していただきありがとう      ║",
            "║   ございました。                                                  ║",
            "║                                                                  ║",
            "║   主な機能:                                                      ║",
            "║   ✓ インテリジェントなコマンド補完                               ║",
            "║   ✓ ファイルシステムナビゲーション                               ║",
            "║   ✓ コンテキスト認識オプション補完                               ║",
            "║   ✓ 履歴ベース補完                                              ║",
            "║   ✓ ファジーマッチング                                          ║",
            "║                                                                  ║",
            "║   詳細については、ドキュメントをご参照ください。                  ║",
            "║                                                                  ║",
            "║               Enterキーを押して終了してください                   ║",
            "║                                                                  ║",
            "╚══════════════════════════════════════════════════════════════════╝",
        ];

        for (i, line) in conclusion_text.iter().enumerate() {
            execute!(stdout(), cursor::MoveTo(5, 3 + i as u16))?;
            print!("{}", line.with(self.theme.colors.success));
        }

        stdout().flush()?;
        self.wait_for_enter()?;
        Ok(())
    }
}

/// デモアクション
#[derive(Debug, Clone, PartialEq)]
enum DemoAction {
    Continue,
    NextScenario,
    Exit,
    TriggerCompletion,
    ApplyCompletion,
    CancelCompletion,
}

impl Default for CompletionDemo {
    fn default() -> Self {
        Self::new().expect("Failed to create completion demo")
    }
}

/// デモを実行する関数
pub async fn run_completion_demo() -> Result<()> {
    let mut demo = CompletionDemo::new()?;
    demo.run_demo().await
}

/// コマンドライン引数でデモを実行
pub async fn run_demo_from_args(args: &[String]) -> Result<()> {
    if args.len() > 1 && args[1] == "completion-demo" {
        run_completion_demo().await
    } else {
        println!("使用方法: nxsh completion-demo");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_creation() {
        let demo = CompletionDemo::new();
        assert!(demo.is_ok());
    }

    #[test]
    fn test_scenario_creation() {
        let scenarios = CompletionDemo::create_demo_scenarios();
        assert!(!scenarios.is_empty());
        assert_eq!(scenarios.len(), 5);
    }

    #[test]
    fn test_input_handling() {
        let mut demo = CompletionDemo::new().unwrap();
        demo.insert_char('h');
        demo.insert_char('e');
        demo.insert_char('l');
        demo.insert_char('l');
        demo.insert_char('o');

        assert_eq!(demo.input_buffer, "hello");
        assert_eq!(demo.cursor_position, 5);
    }

    #[test]
    fn test_completion_selection() {
        let mut demo = CompletionDemo::new().unwrap();
        
        // Mock completion result
        let completions = CompletionResult::new(
            vec![
                CompletionItem::new("ls".to_string(), CompletionType::Command),
                CompletionItem::new("less".to_string(), CompletionType::Command),
                CompletionItem::new("ln".to_string(), CompletionType::Command),
            ],
            "l".to_string(),
        );
        
        demo.current_completions = Some(completions);
        assert_eq!(demo.completion_index, 0);

        demo.select_next_completion();
        assert_eq!(demo.completion_index, 1);

        demo.select_next_completion();
        assert_eq!(demo.completion_index, 2);

        demo.select_next_completion(); // Should wrap around
        assert_eq!(demo.completion_index, 0);

        demo.select_previous_completion(); // Should wrap to end
        assert_eq!(demo.completion_index, 2);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("NexusShell Completion Demo");
    println!("========================");
    
    let mut demo = CompletionDemo::new()?;
    demo.run().await?;
    
    Ok(())
}
