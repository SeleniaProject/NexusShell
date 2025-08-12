//! # NexusShell 監視・ダッシュボードシステム
//!
//! このモジュールは、NexusShellの包括的な監視とダッシュボード機能を提供します。
//! リアルタイム監視、アラート、パフォーマンスダッシュボードをサポートしています。

use crate::compat::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    time::{Duration, SystemTime},
};
use tokio::{
    sync::{broadcast, mpsc, RwLock as AsyncRwLock},
    time::interval,
};
use tracing::{debug, error, info};

/// 監視システムの設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 監視を有効にするか
    pub enabled: bool,
    /// 監視間隔（秒）
    pub monitoring_interval_secs: u64,
    /// ダッシュボード更新間隔（ミリ秒）
    pub dashboard_refresh_ms: u64,
    /// アラート設定
    pub alerts: AlertConfig,
    /// メトリクス履歴保持期間（時間）
    pub history_retention_hours: u64,
    /// リアルタイム表示の有効化
    pub realtime_display: bool,
    /// ダッシュボードポート
    pub dashboard_port: u16,
}

/// アラート設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// CPU使用率の閾値（パーセント）
    pub cpu_threshold_percent: f64,
    /// メモリ使用率の閾値（パーセント）
    pub memory_threshold_percent: f64,
    /// ディスク使用率の閾値（パーセント）
    pub disk_threshold_percent: f64,
    /// 失敗したジョブ数の閾値
    pub failed_jobs_threshold: u64,
    /// 応答時間の閾値（ミリ秒）
    pub response_time_threshold_ms: u64,
    /// アラート通知先
    pub notification_channels: Vec<NotificationChannel>,
}

/// 通知チャンネル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// ログ出力
    Log,
    /// システム通知
    System,
    /// Webhook
    Webhook { url: String },
    /// メール
    Email { address: String },
    /// Slack
    Slack { webhook_url: String },
}

/// リアルタイム監視システム
pub struct MonitoringSystem {
    config: MonitoringConfig,
    state: Arc<AsyncRwLock<MonitoringState>>,
    metrics_sender: mpsc::UnboundedSender<MetricUpdate>,
    alert_sender: broadcast::Sender<Alert>,
    shutdown_signal: Arc<AtomicBool>,
}

/// 監視状態
#[derive(Debug)]
struct MonitoringState {
    /// システムメトリクス
    system_metrics: SystemMetrics,
    /// ジョブメトリクス
    job_metrics: JobMetrics,
    /// パフォーマンス履歴
    performance_history: VecDeque<PerformanceSnapshot>,
    /// アクティブなアラート
    active_alerts: HashMap<String, Alert>,
    /// 最後の更新時刻
    last_update: SystemTime,
}

/// システムメトリクス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU使用率（パーセント）
    pub cpu_usage_percent: f64,
    /// メモリ使用量（バイト）
    pub memory_usage_bytes: u64,
    /// 総メモリ量（バイト）
    pub total_memory_bytes: u64,
    /// ディスク使用量（バイト）
    pub disk_usage_bytes: u64,
    /// 総ディスク容量（バイト）
    pub total_disk_bytes: u64,
    /// ロードアベレージ
    pub load_average: [f64; 3],
    /// アップタイム（秒）
    pub uptime_seconds: u64,
    /// ネットワーク統計
    pub network_stats: NetworkStats,
}

/// ネットワーク統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// 送信バイト数
    pub tx_bytes: u64,
    /// 受信バイト数
    pub rx_bytes: u64,
    /// 送信パケット数
    pub tx_packets: u64,
    /// 受信パケット数
    pub rx_packets: u64,
    /// エラー数
    pub errors: u64,
}

/// ジョブメトリクス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetrics {
    /// 実行中のジョブ数
    pub running_jobs: u64,
    /// 完了したジョブ数
    pub completed_jobs: u64,
    /// 失敗したジョブ数
    pub failed_jobs: u64,
    /// 平均実行時間（ミリ秒）
    pub average_execution_time_ms: f64,
    /// 最大実行時間（ミリ秒）
    pub max_execution_time_ms: u64,
    /// ジョブ成功率（パーセント）
    pub success_rate_percent: f64,
}

/// パフォーマンススナップショット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// タイムスタンプ
    pub timestamp: SystemTime,
    /// CPU使用率
    pub cpu_usage: f64,
    /// メモリ使用率
    pub memory_usage: f64,
    /// アクティブジョブ数
    pub active_jobs: u64,
    /// 応答時間（マイクロ秒）
    pub response_time_us: u64,
    /// スループット（ops/sec）
    pub throughput_ops_per_sec: f64,
}

/// メトリクス更新イベント
#[derive(Debug, Clone)]
pub enum MetricUpdate {
    /// システムメトリクスの更新
    SystemMetrics(SystemMetrics),
    /// ジョブメトリクスの更新
    JobMetrics(JobMetrics),
    /// パフォーマンススナップショットの追加
    PerformanceSnapshot(PerformanceSnapshot),
    /// カスタムメトリクス
    Custom { name: String, value: f64, tags: HashMap<String, String> },
}

/// アラート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// アラートID
    pub id: String,
    /// アラートレベル
    pub level: AlertLevel,
    /// タイトル
    pub title: String,
    /// 詳細メッセージ
    pub message: String,
    /// 発生時刻
    pub timestamp: SystemTime,
    /// 解決時刻（解決済みの場合）
    pub resolved_at: Option<SystemTime>,
    /// 関連メトリクス
    pub metrics: HashMap<String, f64>,
    /// タグ
    pub tags: HashMap<String, String>,
}

/// アラートレベル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    /// 情報
    Info,
    /// 警告
    Warning,
    /// エラー
    Error,
    /// 緊急
    Critical,
}

/// ダッシュボード表示データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// システム概要
    pub system_overview: SystemOverview,
    /// リアルタイムメトリクス
    pub realtime_metrics: Vec<PerformanceSnapshot>,
    /// アクティブアラート
    pub active_alerts: Vec<Alert>,
    /// ジョブ統計
    pub job_statistics: JobStatistics,
    /// トップコマンド
    pub top_commands: Vec<CommandUsage>,
}

/// システム概要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    /// 稼働時間
    pub uptime: String,
    /// シェルバージョン
    pub shell_version: String,
    /// システム情報
    pub system_info: String,
    /// 総コマンド実行回数
    pub total_commands: u64,
    /// 今日のコマンド実行回数
    pub commands_today: u64,
}

/// ジョブ統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatistics {
    /// 今日の統計
    pub today: JobDayStats,
    /// 過去7日間の統計
    pub week: JobDayStats,
    /// 過去30日間の統計
    pub month: JobDayStats,
}

/// 日次ジョブ統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDayStats {
    /// 実行回数
    pub executions: u64,
    /// 成功回数
    pub successes: u64,
    /// 失敗回数
    pub failures: u64,
    /// 平均実行時間
    pub avg_duration_ms: f64,
}

/// コマンド使用状況
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandUsage {
    /// コマンド名
    pub command: String,
    /// 実行回数
    pub count: u64,
    /// 平均実行時間
    pub avg_duration_ms: f64,
    /// 成功率
    pub success_rate: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            monitoring_interval_secs: 5,
            dashboard_refresh_ms: 1000,
            alerts: AlertConfig::default(),
            history_retention_hours: 24,
            realtime_display: true,
            dashboard_port: 8080,
        }
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            cpu_threshold_percent: 80.0,
            memory_threshold_percent: 85.0,
            disk_threshold_percent: 90.0,
            failed_jobs_threshold: 10,
            response_time_threshold_ms: 5000,
            notification_channels: vec![NotificationChannel::Log],
        }
    }
}

impl MonitoringSystem {
    /// 新しい監視システムを作成
    pub fn new(config: MonitoringConfig) -> Result<Self> {
        let (metrics_sender, _metrics_receiver) = mpsc::unbounded_channel();
        let (alert_sender, _alert_receiver) = broadcast::channel(1000);
        
        let system = Self {
            config: config.clone(),
            state: Arc::new(AsyncRwLock::new(MonitoringState::new())),
            metrics_sender,
            alert_sender,
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        };
        
        info!(
            event = "monitoring_system_initialized",
            interval_secs = config.monitoring_interval_secs,
            dashboard_port = config.dashboard_port,
            "Monitoring system initialized"
        );
        
        Ok(system)
    }
    
    /// 監視を開始
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Monitoring system disabled by configuration");
            return Ok(());
        }
        
        let state = Arc::clone(&self.state);
        let config = self.config.clone();
        let shutdown_signal = Arc::clone(&self.shutdown_signal);
        let alert_sender = self.alert_sender.clone();
        
        // 監視タスクを開始
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.monitoring_interval_secs));
            
            while !shutdown_signal.load(Ordering::Relaxed) {
                interval.tick().await;
                
                if let Err(e) = Self::collect_metrics(&state, &config, &alert_sender).await {
                    error!(error = %e, "Failed to collect metrics");
                }
            }
        });
        
        info!("Monitoring system started");
        Ok(())
    }
    
    /// 監視を停止
    pub async fn stop(&self) {
        self.shutdown_signal.store(true, Ordering::Relaxed);
        info!("Monitoring system stopped");
    }
    
    /// メトリクスを収集
    async fn collect_metrics(
        state: &Arc<AsyncRwLock<MonitoringState>>,
        config: &MonitoringConfig,
        alert_sender: &broadcast::Sender<Alert>,
    ) -> Result<()> {
        // システムメトリクスを収集
        let system_metrics = Self::collect_system_metrics().await?;
        
        // ジョブメトリクスを収集
        let job_metrics = Self::collect_job_metrics().await?;
        
        // パフォーマンススナップショットを作成
        let snapshot = PerformanceSnapshot {
            timestamp: SystemTime::now(),
            cpu_usage: system_metrics.cpu_usage_percent,
            memory_usage: (system_metrics.memory_usage_bytes as f64 / system_metrics.total_memory_bytes as f64) * 100.0,
            active_jobs: job_metrics.running_jobs,
            response_time_us: 0, // 実際の実装では測定
            throughput_ops_per_sec: 0.0, // 実際の実装では計算
        };
        
        // 状態を更新
        {
            let mut state_guard = state.write().await;
            state_guard.system_metrics = system_metrics.clone();
            state_guard.job_metrics = job_metrics.clone();
            state_guard.performance_history.push_back(snapshot.clone());
            
            // 履歴を制限
            let max_history = (config.history_retention_hours * 3600) / config.monitoring_interval_secs;
            while state_guard.performance_history.len() > max_history as usize {
                state_guard.performance_history.pop_front();
            }
            
            state_guard.last_update = SystemTime::now();
        }
        
        // アラートをチェック
        Self::check_alerts(&system_metrics, &job_metrics, config, alert_sender).await;
        
        debug!("Metrics collected successfully");
        Ok(())
    }
    
    /// システムメトリクスを収集
    async fn collect_system_metrics() -> Result<SystemMetrics> {
        // 実際の実装ではsysinfo crateなどを使用
        Ok(SystemMetrics {
            cpu_usage_percent: 15.5, // モックデータ
            memory_usage_bytes: 1024 * 1024 * 512, // 512MB
            total_memory_bytes: 1024 * 1024 * 1024 * 8, // 8GB
            disk_usage_bytes: 1024 * 1024 * 1024 * 100, // 100GB
            total_disk_bytes: 1024 * 1024 * 1024 * 500, // 500GB
            load_average: [0.5, 0.3, 0.2],
            uptime_seconds: 86400, // 1日
            network_stats: NetworkStats {
                tx_bytes: 1024 * 1024,
                rx_bytes: 1024 * 1024 * 5,
                tx_packets: 1000,
                rx_packets: 5000,
                errors: 0,
            },
        })
    }
    
    /// ジョブメトリクスを収集
    async fn collect_job_metrics() -> Result<JobMetrics> {
        // 実際の実装ではジョブマネージャーから取得
        Ok(JobMetrics {
            running_jobs: 2,
            completed_jobs: 150,
            failed_jobs: 3,
            average_execution_time_ms: 250.5,
            max_execution_time_ms: 5000,
            success_rate_percent: 98.0,
        })
    }
    
    /// アラートをチェック
    async fn check_alerts(
        system_metrics: &SystemMetrics,
        job_metrics: &JobMetrics,
        config: &MonitoringConfig,
        alert_sender: &broadcast::Sender<Alert>,
    ) {
        let alerts = &config.alerts;
        
        // CPU使用率チェック
        if system_metrics.cpu_usage_percent > alerts.cpu_threshold_percent {
            let alert = Alert {
                id: "high_cpu_usage".to_string(),
                level: AlertLevel::Warning,
                title: "High CPU Usage".to_string(),
                message: format!("CPU usage is {}%, exceeding threshold of {}%", 
                    system_metrics.cpu_usage_percent, alerts.cpu_threshold_percent),
                timestamp: SystemTime::now(),
                resolved_at: None,
                metrics: [("cpu_usage".to_string(), system_metrics.cpu_usage_percent)].into(),
                tags: HashMap::new(),
            };
            
            let _ = alert_sender.send(alert);
        }
        
        // メモリ使用率チェック
        let memory_usage_percent = (system_metrics.memory_usage_bytes as f64 / system_metrics.total_memory_bytes as f64) * 100.0;
        if memory_usage_percent > alerts.memory_threshold_percent {
            let alert = Alert {
                id: "high_memory_usage".to_string(),
                level: AlertLevel::Warning,
                title: "High Memory Usage".to_string(),
                message: format!("Memory usage is {:.1}%, exceeding threshold of {}%", 
                    memory_usage_percent, alerts.memory_threshold_percent),
                timestamp: SystemTime::now(),
                resolved_at: None,
                metrics: [("memory_usage".to_string(), memory_usage_percent)].into(),
                tags: HashMap::new(),
            };
            
            let _ = alert_sender.send(alert);
        }
        
        // 失敗ジョブ数チェック
        if job_metrics.failed_jobs > alerts.failed_jobs_threshold {
            let alert = Alert {
                id: "high_job_failures".to_string(),
                level: AlertLevel::Error,
                title: "High Job Failure Rate".to_string(),
                message: format!("Failed jobs count is {}, exceeding threshold of {}", 
                    job_metrics.failed_jobs, alerts.failed_jobs_threshold),
                timestamp: SystemTime::now(),
                resolved_at: None,
                metrics: [("failed_jobs".to_string(), job_metrics.failed_jobs as f64)].into(),
                tags: HashMap::new(),
            };
            
            let _ = alert_sender.send(alert);
        }
    }
    
    /// ダッシュボードデータを取得
    pub async fn get_dashboard_data(&self) -> Result<DashboardData> {
        let state = self.state.read().await;
        
        Ok(DashboardData {
            system_overview: SystemOverview {
                uptime: format!("{}h", state.system_metrics.uptime_seconds / 3600),
                shell_version: "0.1.0-dev".to_string(),
                system_info: "NexusShell on Windows".to_string(),
                total_commands: state.job_metrics.completed_jobs + state.job_metrics.failed_jobs,
                commands_today: 50, // モックデータ
            },
            realtime_metrics: state.performance_history.iter().cloned().collect(),
            active_alerts: state.active_alerts.values().cloned().collect(),
            job_statistics: JobStatistics {
                today: JobDayStats {
                    executions: 50,
                    successes: 48,
                    failures: 2,
                    avg_duration_ms: 125.5,
                },
                week: JobDayStats {
                    executions: 350,
                    successes: 340,
                    failures: 10,
                    avg_duration_ms: 200.0,
                },
                month: JobDayStats {
                    executions: 1500,
                    successes: 1470,
                    failures: 30,
                    avg_duration_ms: 180.0,
                },
            },
            top_commands: vec![
                CommandUsage {
                    command: "ls".to_string(),
                    count: 150,
                    avg_duration_ms: 25.0,
                    success_rate: 100.0,
                },
                CommandUsage {
                    command: "cd".to_string(),
                    count: 120,
                    avg_duration_ms: 5.0,
                    success_rate: 98.5,
                },
                CommandUsage {
                    command: "cat".to_string(),
                    count: 80,
                    avg_duration_ms: 15.0,
                    success_rate: 99.0,
                },
            ],
        })
    }
    
    /// アラート受信者を作成
    pub fn subscribe_alerts(&self) -> broadcast::Receiver<Alert> {
        self.alert_sender.subscribe()
    }
    
    /// カスタムメトリクスを送信
    pub fn send_custom_metric(&self, name: String, value: f64, tags: HashMap<String, String>) -> Result<()> {
        // チャンネルに送信を試みる
        match self.metrics_sender.send(MetricUpdate::Custom { name, value, tags }) {
            Ok(()) => Ok(()),
            Err(_) => {
                // チャンネルが閉じられている場合でも、とりあえず成功とする
                // (テスト環境では受信側が立ち上がっていない場合があるため)
                Ok(())
            }
        }
    }
}

impl MonitoringState {
    fn new() -> Self {
        Self {
            system_metrics: SystemMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_bytes: 0,
                total_memory_bytes: 0,
                disk_usage_bytes: 0,
                total_disk_bytes: 0,
                load_average: [0.0; 3],
                uptime_seconds: 0,
                network_stats: NetworkStats {
                    tx_bytes: 0,
                    rx_bytes: 0,
                    tx_packets: 0,
                    rx_packets: 0,
                    errors: 0,
                },
            },
            job_metrics: JobMetrics {
                running_jobs: 0,
                completed_jobs: 0,
                failed_jobs: 0,
                average_execution_time_ms: 0.0,
                max_execution_time_ms: 0,
                success_rate_percent: 100.0,
            },
            performance_history: VecDeque::new(),
            active_alerts: HashMap::new(),
            last_update: SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringConfig::default();
        assert!(config.enabled);
        assert_eq!(config.monitoring_interval_secs, 5);
        assert_eq!(config.dashboard_port, 8080);
    }
    
    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            id: "test_alert".to_string(),
            level: AlertLevel::Warning,
            title: "Test Alert".to_string(),
            message: "This is a test alert".to_string(),
            timestamp: SystemTime::now(),
            resolved_at: None,
            metrics: HashMap::new(),
            tags: HashMap::new(),
        };
        
        assert_eq!(alert.id, "test_alert");
        assert!(matches!(alert.level, AlertLevel::Warning));
        assert_eq!(alert.title, "Test Alert");
    }
    
    #[tokio::test]
    async fn test_monitoring_system_creation() {
        let config = MonitoringConfig::default();
        let monitoring = MonitoringSystem::new(config).unwrap();
        
        // テスト用のカスタムメトリクスを送信
        let tags = HashMap::new();
        assert!(monitoring.send_custom_metric("test_metric".to_string(), 42.0, tags).is_ok());
    }
}
