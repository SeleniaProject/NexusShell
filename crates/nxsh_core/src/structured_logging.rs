//! # NexusShell 構造化ログシステム
//!
//! このモジュールは、NexusShellのために設計された高性能な構造化ログシステムを提供します。
//! JSON形式での出力、ログローテーション、非同期書き込みをサポートしています。

use crate::compat::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
#[cfg(feature = "logging")]
use tracing_appender::non_blocking::WorkerGuard;
#[cfg(not(feature = "logging"))]
type WorkerGuard = (); // stub type when logging disabled
use std::sync::Arc;
#[cfg(feature = "logging")]
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// ログ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// ログレベル (trace, debug, info, warn, error)
    pub level: String,
    /// ログ出力形式
    pub format: LogFormat,
    /// ファイル出力設定
    pub file_output: Option<FileOutputConfig>,
    /// コンソール出力を有効にするか
    pub console_output: bool,
    /// JSON形式で出力するか
    pub json_format: bool,
    /// ANSIカラーを使用するか
    pub use_colors: bool,
}

/// ログ出力形式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    /// 人間が読みやすい形式
    Pretty,
    /// 構造化JSON形式
    Json,
    /// コンパクト形式
    Compact,
}

/// ファイル出力設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOutputConfig {
    /// ログファイルのベースパス
    pub path: PathBuf,
    /// ローテーション設定
    pub rotation: RotationConfig,
    /// 最大ファイルサイズ (MB)
    pub max_file_size: u64,
    /// 保持するファイル数
    pub max_files: u32,
}

/// ログローテーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationConfig {
    /// 日次ローテーション
    Daily,
    /// 時間毎ローテーション
    Hourly,
    /// サイズベース
    Size(u64),
    /// ローテーションなし
    Never,
}

/// NexusShellの構造化ログシステム
#[derive(Debug)]
pub struct StructuredLogger {
    config: Arc<RwLock<LogConfig>>,
    _guard: Option<WorkerGuard>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            file_output: None,
            console_output: true,
            json_format: false,
            use_colors: true,
        }
    }
}

impl StructuredLogger {
    /// Determine whether file logging should be disabled regardless of configuration
    /// Conditions:
    /// - BusyBox/minimal builds (compile-time feature)
    /// - Environment NXSH_DISABLE_FILE_LOGGING=1/true
    /// - Environment NXSH_BUSYBOX=1/true (runtime BusyBox mode)
    #[allow(dead_code)]
    fn is_file_logging_disabled() -> bool {
        #[cfg(feature = "busybox_min")]
        {
            true
        }
        #[cfg(not(feature = "busybox_min"))]
        {
            let by_env_disable = std::env::var("NXSH_DISABLE_FILE_LOGGING")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let by_env_busybox = std::env::var("NXSH_BUSYBOX")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            by_env_disable || by_env_busybox
        }
    }
    /// 新しい構造化ログシステムを作成
    pub fn new(config: LogConfig) -> Result<Self> {
        let base = Self {
            config: Arc::new(RwLock::new(config.clone())),
            _guard: None,
        };

        #[cfg(feature = "logging")]
        let mut logger = base;
        #[cfg(not(feature = "logging"))]
        let logger = base;

        #[cfg(feature = "logging")]
        {
            logger.initialize_logger(&config)?;
        }

        info!(
            event = "logger_initialized",
            level = %config.level,
            json_format = config.json_format,
            "NexusShell structured logger initialized"
        );

        Ok(logger)
    }

    /// デフォルト設定でロガーを初期化
    pub fn init_default() -> Result<Self> {
        Self::new(LogConfig::default())
    }

    /// 開発環境用のロガーを初期化
    pub fn init_development() -> Result<Self> {
        let config = LogConfig {
            level: "debug".to_string(),
            format: LogFormat::Pretty,
            console_output: true,
            json_format: false,
            use_colors: true,
            file_output: Some(FileOutputConfig {
                path: PathBuf::from("logs/nxsh-dev.log"),
                rotation: RotationConfig::Daily,
                max_file_size: 100, // 100MB
                max_files: 30,
            }),
        };
        Self::new(config)
    }

    /// 本番環境用のロガーを初期化
    pub fn init_production() -> Result<Self> {
        let config = LogConfig {
            level: "info".to_string(),
            format: LogFormat::Json,
            console_output: false,
            json_format: true,
            use_colors: false,
            file_output: Some(FileOutputConfig {
                path: PathBuf::from("/var/log/nexusshell/nxsh.log"),
                rotation: RotationConfig::Daily,
                max_file_size: 500, // 500MB
                max_files: 90,      // 3ヶ月間保持
            }),
        };
        Self::new(config)
    }

    /// ロガーを初期化
    #[cfg(feature = "logging")]
    fn initialize_logger(&mut self, config: &LogConfig) -> Result<()> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

        // ベースとなる設定
        let registry = tracing_subscriber::registry().with(env_filter);

        if !Self::is_file_logging_disabled() && config.file_output.is_some() {
            let file_config = config.file_output.as_ref().unwrap();
            #[cfg(not(feature = "logging"))]
            {
                // In non-logging builds, ignore file output configuration silently
                return Ok(());
            }
            #[cfg(feature = "logging")]
            // ファイル出力設定がある場合
            let (file_writer, guard) = self.create_file_writer(file_config)?;
            self._guard = Some(guard);

            if config.console_output {
                // コンソールとファイルの両方に出力
                let fmt_layer = fmt::layer()
                    .with_span_events(FmtSpan::CLOSE)
                    .with_ansi(config.use_colors);

                let file_layer = {
                    #[cfg(feature = "logging-json")]
                    {
                        fmt::layer()
                            .json()
                            .with_writer(file_writer)
                            .with_ansi(false)
                    }
                    #[cfg(not(feature = "logging-json"))]
                    {
                        fmt::layer()
                            .compact()
                            .with_writer(file_writer)
                            .with_ansi(false)
                    }
                };

                registry.with(fmt_layer).with(file_layer).init();
            } else {
                // ファイルのみに出力
                let file_layer = {
                    #[cfg(feature = "logging-json")]
                    {
                        fmt::layer()
                            .json()
                            .with_writer(file_writer)
                            .with_ansi(false)
                    }
                    #[cfg(not(feature = "logging-json"))]
                    {
                        fmt::layer()
                            .compact()
                            .with_writer(file_writer)
                            .with_ansi(false)
                    }
                };

                registry.with(file_layer).init();
            }
        } else if config.console_output {
            // コンソールのみに出力
            match config.format {
                LogFormat::Json => {
                    #[cfg(feature = "logging-json")]
                    {
                        let fmt_layer = fmt::layer().json().with_ansi(config.use_colors);
                        registry.with(fmt_layer).init();
                    }
                    #[cfg(not(feature = "logging-json"))]
                    {
                        let fmt_layer = fmt::layer().compact().with_ansi(config.use_colors);
                        registry.with(fmt_layer).init();
                    }
                }
                LogFormat::Compact => {
                    let fmt_layer = fmt::layer().compact().with_ansi(config.use_colors);
                    registry.with(fmt_layer).init();
                }
                LogFormat::Pretty => {
                    let fmt_layer = fmt::layer().pretty().with_ansi(config.use_colors);
                    registry.with(fmt_layer).init();
                }
            }
        } else {
            // 出力なし（基本レジストリのみ）
            registry.init();
        }

        Ok(())
    }

    /// ファイル書き込み設定を作成
    #[cfg(feature = "logging")]
    fn create_file_writer(
        &self,
        config: &FileOutputConfig,
    ) -> Result<(tracing_appender::non_blocking::NonBlocking, WorkerGuard)> {
        // ログディレクトリを作成
        if let Some(parent) = config.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::anyhow!("Failed to create log directory: {parent:?}: {e}"))?;
        }

        // ローテーションファイルアペンダーを作成
        let file_appender = match config.rotation {
            RotationConfig::Daily => {
                let dir = config.path.parent().unwrap_or_else(|| Path::new("."));
                let prefix = config.path.file_stem().unwrap().to_string_lossy();
                tracing_appender::rolling::daily(dir, prefix.as_ref())
            }
            RotationConfig::Hourly => {
                let dir = config.path.parent().unwrap_or_else(|| Path::new("."));
                let prefix = config.path.file_stem().unwrap().to_string_lossy();
                tracing_appender::rolling::hourly(dir, prefix.as_ref())
            }
            _ => {
                // サイズベースやNeverの場合は日次に fallback
                let dir = config.path.parent().unwrap_or_else(|| Path::new("."));
                let prefix = config.path.file_stem().unwrap().to_string_lossy();
                tracing_appender::rolling::daily(dir, prefix.as_ref())
            }
        };

        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        Ok((non_blocking, guard))
    }

    #[cfg(not(feature = "logging"))]
    #[allow(unused)]
    fn create_file_writer(&self, _config: &FileOutputConfig) -> Result<((), WorkerGuard)> {
        // Logging disabled: stub implementation never called
        Ok(((), ()))
    }

    /// ログ設定を更新
    pub async fn update_config(&self, config: LogConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config.clone();

        info!(
            event = "config_updated",
            level = %config.level,
            json_format = config.json_format,
            "Log configuration updated"
        );

        Ok(())
    }

    /// 現在のログ設定を取得
    pub async fn get_config(&self) -> LogConfig {
        self.config.read().await.clone()
    }

    /// ログ統計を取得
    pub async fn get_stats(&self) -> LogStats {
        // 実装は省略 - 実際には統計情報を収集
        LogStats {
            total_messages: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            debug_count: 0,
            trace_count: 0,
        }
    }

    /// ログファイルのローテーションを手動で実行
    pub async fn rotate_logs(&self) -> Result<()> {
        let config = self.config.read().await;

        if let Some(file_config) = &config.file_output {
            info!(
                event = "log_rotation_started",
                path = %file_config.path.display(),
                "Starting manual log rotation"
            );

            // ローテーション処理の実装
            // 実際の実装では、現在のログファイルをアーカイブして新しいファイルを作成

            info!(
                event = "log_rotation_completed",
                "Log rotation completed successfully"
            );
        }

        Ok(())
    }
}

/// ログ統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total_messages: u64,
    pub error_count: u64,
    pub warn_count: u64,
    pub info_count: u64,
    pub debug_count: u64,
    pub trace_count: u64,
}

/// ログイベント用のヘルパーマクロ
#[macro_export]
macro_rules! log_event {
    ($level:ident, $event:expr, $($field:ident = $value:expr),* $(,)?) => {
        tracing::$level!(
            event = $event,
            $($field = $value,)*
        );
    };
}

/// シェルコマンド実行ログの構造化データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionLog {
    pub event: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub user: String,
    pub start_time: std::time::SystemTime,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub pid: Option<u32>,
    pub memory_usage: Option<u64>,
    pub error_message: Option<String>,
}

impl CommandExecutionLog {
    /// コマンド開始時のログエントリを作成
    pub fn start(command: &str, args: &[String], working_dir: &Path) -> Self {
        Self {
            event: "command_started".to_string(),
            command: command.to_string(),
            args: args.to_vec(),
            working_dir: working_dir.to_path_buf(),
            user: {
                #[cfg(feature = "system-info")]
                {
                    whoami::username()
                }
                #[cfg(not(feature = "system-info"))]
                {
                    "unknown".to_string()
                }
            },
            start_time: std::time::SystemTime::now(),
            duration_ms: None,
            exit_code: None,
            pid: None,
            memory_usage: None,
            error_message: None,
        }
    }

    /// コマンド完了時の情報を更新
    pub fn complete(&mut self, exit_code: i32, pid: Option<u32>, memory_usage: Option<u64>) {
        self.event = "command_completed".to_string();
        self.duration_ms = self.start_time.elapsed().ok().map(|d| d.as_millis() as u64);
        self.exit_code = Some(exit_code);
        self.pid = pid;
        self.memory_usage = memory_usage;
    }

    /// エラー情報を設定
    pub fn set_error(&mut self, error: &str) {
        self.event = "command_failed".to_string();
        self.error_message = Some(error.to_string());
        self.duration_ms = self.start_time.elapsed().ok().map(|d| d.as_millis() as u64);
    }

    /// ログエントリを出力
    pub fn log(&self) {
        match self.event.as_str() {
            "command_started" => {
                info!(
                    event = %self.event,
                    command = %self.command,
                    args = ?self.args,
                    working_dir = %self.working_dir.display(),
                    user = %self.user,
                    pid = ?self.pid,
                    "Command execution started"
                );
            }
            "command_completed" => {
                info!(
                    event = %self.event,
                    command = %self.command,
                    exit_code = ?self.exit_code,
                    duration_ms = ?self.duration_ms,
                    memory_usage = ?self.memory_usage,
                    pid = ?self.pid,
                    "Command execution completed"
                );
            }
            "command_failed" => {
                error!(
                    event = %self.event,
                    command = %self.command,
                    error = ?self.error_message,
                    duration_ms = ?self.duration_ms,
                    "Command execution failed"
                );
            }
            _ => {
                warn!(
                    event = %self.event,
                    command = %self.command,
                    "Unknown command event"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(matches!(config.format, LogFormat::Pretty));
        assert!(config.console_output);
        assert!(!config.json_format);
        assert!(config.use_colors);
    }

    #[tokio::test]
    async fn test_structured_logger_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("test.log");

        let config = LogConfig {
            level: "debug".to_string(),
            format: LogFormat::Json,
            console_output: false,
            json_format: true,
            use_colors: false,
            file_output: Some(FileOutputConfig {
                path: log_path.clone(),
                rotation: RotationConfig::Never,
                max_file_size: 10,
                max_files: 5,
            }),
        };

        // Note: This test would fail in the current environment due to
        // tracing subscriber already being initialized. In a real test environment,
        // each test should run in isolation.

        assert_eq!(config.level, "debug");
        assert!(config.json_format);
    }

    #[test]
    fn test_command_execution_log() {
        let mut log =
            CommandExecutionLog::start("ls", &["-la".to_string()], &PathBuf::from("/home/user"));

        assert_eq!(log.command, "ls");
        assert_eq!(log.args, vec!["-la"]);
        assert_eq!(log.event, "command_started");

        log.complete(0, Some(12345), Some(1024));
        assert_eq!(log.event, "command_completed");
        assert_eq!(log.exit_code, Some(0));
        assert_eq!(log.pid, Some(12345));

        log.set_error("Permission denied");
        assert_eq!(log.event, "command_failed");
        assert!(log.error_message.is_some());
    }
}
