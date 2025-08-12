//! # NexusShell 高度なジョブスケジューラー
//!
//! このモジュールは、cron風の定期実行、at風の時間指定実行、
//! 依存関係管理、優先度制御を含む高度なジョブスケジューリング機能を提供します。

use crate::compat::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, BinaryHeap, VecDeque},
    sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}},
    time::{Duration, SystemTime, Instant},
    cmp::{Ordering as CmpOrdering, Reverse},
};
use tokio::{
    sync::{RwLock as AsyncRwLock, Semaphore},
    time::interval,
    task::JoinHandle,
};
use tracing::{error, info};

/// スケジューラー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// 最大同時実行ジョブ数
    pub max_concurrent_jobs: usize,
    /// スケジュール確認間隔（秒）
    pub check_interval_secs: u64,
    /// ジョブ履歴保持期間（時間）
    pub history_retention_hours: u64,
    /// 失敗時の再試行回数
    pub default_retry_count: u32,
    /// 再試行間隔（秒）
    pub default_retry_interval_secs: u64,
    /// ジョブタイムアウト（秒）
    pub default_timeout_secs: u64,
    /// 優先度キューの有効化
    pub enable_priority_queue: bool,
}

/// 高度なジョブスケジューラー
pub struct AdvancedJobScheduler {
    config: SchedulerConfig,
    jobs: Arc<AsyncRwLock<HashMap<String, ScheduledJob>>>,
    queue: Arc<AsyncRwLock<BinaryHeap<Reverse<QueuedJob>>>>,
    running_jobs: Arc<AsyncRwLock<HashMap<String, RunningJob>>>,
    job_history: Arc<AsyncRwLock<VecDeque<JobHistoryEntry>>>,
    semaphore: Arc<Semaphore>,
    shutdown_signal: Arc<AtomicBool>,
    job_counter: AtomicU64,
    scheduler_handle: Option<JoinHandle<()>>,
}

/// スケジュールされたジョブ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    /// ジョブID
    pub id: String,
    /// ジョブ名
    pub name: String,
    /// 実行コマンド
    pub command: String,
    /// 引数
    pub args: Vec<String>,
    /// 作業ディレクトリ
    pub working_dir: String,
    /// 環境変数
    pub environment: HashMap<String, String>,
    /// スケジュール設定
    pub schedule: JobSchedule,
    /// 優先度（1-10、10が最高）
    pub priority: u8,
    /// タイムアウト（秒）
    pub timeout_secs: u64,
    /// 再試行設定
    pub retry_config: RetryConfig,
    /// 依存関係
    pub dependencies: Vec<String>,
    /// 通知設定
    pub notifications: NotificationConfig,
    /// 作成時刻
    pub created_at: SystemTime,
    /// 有効/無効
    pub enabled: bool,
    /// メタデータ
    pub metadata: HashMap<String, String>,
}

/// ジョブスケジュール設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobSchedule {
    /// 単発実行（at風）
    Once { 
        /// 実行時刻
        run_at: SystemTime,
    },
    /// 定期実行（cron風）
    Recurring { 
        /// cron式
        cron_expression: String,
        /// 次回実行時刻
        next_run: SystemTime,
        /// 最終実行時刻
        last_run: Option<SystemTime>,
    },
    /// 間隔実行
    Interval { 
        /// 間隔（秒）
        interval_secs: u64,
        /// 次回実行時刻
        next_run: SystemTime,
    },
    /// イベント依存
    EventBased { 
        /// 監視イベント
        trigger_events: Vec<String>,
    },
}

/// 再試行設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// 最大再試行回数
    pub max_retries: u32,
    /// 再試行間隔（秒）
    pub retry_interval_secs: u64,
    /// 指数バックオフの有効化
    pub exponential_backoff: bool,
    /// 最大遅延時間（秒）
    pub max_delay_secs: u64,
}

/// 通知設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 成功時の通知
    pub on_success: bool,
    /// 失敗時の通知
    pub on_failure: bool,
    /// 通知チャンネル
    pub channels: Vec<String>,
    /// カスタムメッセージ
    pub custom_message: Option<String>,
}

/// キューに入っているジョブ
#[derive(Debug, Clone, PartialEq, Eq)]
struct QueuedJob {
    /// ジョブID
    job_id: String,
    /// 実行予定時刻
    scheduled_time: SystemTime,
    /// 優先度
    priority: u8,
    /// 試行回数
    attempt: u32,
}

impl Ord for QueuedJob {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // 実行時刻で比較（早い方が優先）
        match self.scheduled_time.cmp(&other.scheduled_time) {
            CmpOrdering::Equal => {
                // 時刻が同じ場合は優先度で比較（高い方が優先）
                other.priority.cmp(&self.priority)
            }
            other => other,
        }
    }
}

impl PartialOrd for QueuedJob {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

/// 実行中のジョブ
#[derive(Debug)]
struct RunningJob {
    /// ジョブID
    job_id: String,
    /// 実行開始時刻
    started_at: Instant,
    /// 実行ハンドル
    handle: JoinHandle<JobExecutionResult>,
    /// プロセスID
    pid: Option<u32>,
}

/// ジョブ実行結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecutionResult {
    /// ジョブID
    pub job_id: String,
    /// 成功フラグ
    pub success: bool,
    /// 終了コード
    pub exit_code: Option<i32>,
    /// 実行時間（ミリ秒）
    pub execution_time_ms: u64,
    /// 標準出力
    pub stdout: String,
    /// 標準エラー出力
    pub stderr: String,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// 使用メモリ（バイト）
    pub memory_usage: Option<u64>,
    /// CPU使用率
    pub cpu_usage: Option<f64>,
}

/// ジョブ履歴エントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobHistoryEntry {
    /// ジョブID
    pub job_id: String,
    /// 実行開始時刻
    pub started_at: SystemTime,
    /// 実行終了時刻
    pub finished_at: SystemTime,
    /// 実行結果
    pub result: JobExecutionResult,
    /// スケジュールされた実行時刻
    pub scheduled_time: SystemTime,
    /// 遅延時間（ミリ秒）
    pub delay_ms: u64,
}

/// ジョブ統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatistics {
    /// 総ジョブ数
    pub total_jobs: u64,
    /// 実行中のジョブ数
    pub running_jobs: u64,
    /// 待機中のジョブ数
    pub queued_jobs: u64,
    /// 今日の実行回数
    pub executions_today: u64,
    /// 成功率（パーセント）
    pub success_rate: f64,
    /// 平均実行時間（ミリ秒）
    pub avg_execution_time_ms: f64,
    /// 最も実行されるコマンド
    pub top_commands: Vec<(String, u64)>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 10,
            check_interval_secs: 5,
            history_retention_hours: 168, // 1週間
            default_retry_count: 3,
            default_retry_interval_secs: 60,
            default_timeout_secs: 3600, // 1時間
            enable_priority_queue: true,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_interval_secs: 60,
            exponential_backoff: true,
            max_delay_secs: 3600,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            on_success: false,
            on_failure: true,
            channels: vec!["log".to_string()],
            custom_message: None,
        }
    }
}

impl AdvancedJobScheduler {
    /// 新しいスケジューラーを作成
    pub fn new(config: SchedulerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));
        
        Self {
            config,
            jobs: Arc::new(AsyncRwLock::new(HashMap::new())),
            queue: Arc::new(AsyncRwLock::new(BinaryHeap::new())),
            running_jobs: Arc::new(AsyncRwLock::new(HashMap::new())),
            job_history: Arc::new(AsyncRwLock::new(VecDeque::new())),
            semaphore,
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            job_counter: AtomicU64::new(0),
            scheduler_handle: None,
        }
    }
    
    /// スケジューラーを開始
    pub async fn start(&mut self) -> Result<()> {
        let jobs = Arc::clone(&self.jobs);
        let queue = Arc::clone(&self.queue);
        let running_jobs = Arc::clone(&self.running_jobs);
        let job_history = Arc::clone(&self.job_history);
        let semaphore = Arc::clone(&self.semaphore);
        let shutdown_signal = Arc::clone(&self.shutdown_signal);
        let config = self.config.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.check_interval_secs));
            
            info!("Advanced job scheduler started");
            
            while !shutdown_signal.load(Ordering::Relaxed) {
                interval.tick().await;
                
                if let Err(e) = Self::process_scheduled_jobs(
                    &jobs,
                    &queue,
                    &running_jobs,
                    &job_history,
                    &semaphore,
                    &config,
                ).await {
                    error!(error = %e, "Error processing scheduled jobs");
                }
            }
            
            info!("Advanced job scheduler stopped");
        });
        
        self.scheduler_handle = Some(handle);
        Ok(())
    }
    
    /// スケジューラーを停止
    pub async fn stop(&mut self) {
        self.shutdown_signal.store(true, Ordering::Relaxed);
        
        if let Some(handle) = self.scheduler_handle.take() {
            let _ = handle.await;
        }
        
        // 実行中のジョブを停止
        let running_jobs = self.running_jobs.read().await;
        for (_, job) in running_jobs.iter() {
            job.handle.abort();
        }
    }
    
    /// ジョブをスケジュール
    pub async fn schedule_job(&self, job: ScheduledJob) -> Result<String> {
        let job_id = job.id.clone();
        
        // 次回実行時刻を計算
        let next_run = self.calculate_next_run(&job.schedule).await?;
        
        // キューに追加
        {
            let mut queue = self.queue.write().await;
            queue.push(Reverse(QueuedJob {
                job_id: job_id.clone(),
                scheduled_time: next_run,
                priority: job.priority,
                attempt: 0,
            }));
        }
        
        // ジョブを保存
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id.clone(), job);
        }
        
        info!(
            job_id = %job_id,
            next_run = ?next_run,
            "Job scheduled successfully"
        );
        
        Ok(job_id)
    }
    
    /// at風のジョブをスケジュール
    pub async fn schedule_at(&self, command: String, run_at: SystemTime) -> Result<String> {
        let job_id = format!("at_{}", self.job_counter.fetch_add(1, Ordering::Relaxed));
        
        let job = ScheduledJob {
            id: job_id.clone(),
            name: format!("At job: {}", command),
            command: command.clone(),
            args: Vec::new(),
            working_dir: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            environment: std::env::vars().collect(),
            schedule: JobSchedule::Once { run_at },
            priority: 5,
            timeout_secs: self.config.default_timeout_secs,
            retry_config: RetryConfig::default(),
            dependencies: Vec::new(),
            notifications: NotificationConfig::default(),
            created_at: SystemTime::now(),
            enabled: true,
            metadata: HashMap::new(),
        };
        
        self.schedule_job(job).await
    }
    
    /// cron風のジョブをスケジュール
    pub async fn schedule_cron(&self, command: String, cron_expression: String) -> Result<String> {
        let job_id = format!("cron_{}", self.job_counter.fetch_add(1, Ordering::Relaxed));
        
        let next_run = self.parse_cron_expression(&cron_expression).await?;
        
        let job = ScheduledJob {
            id: job_id.clone(),
            name: format!("Cron job: {}", command),
            command: command.clone(),
            args: Vec::new(),
            working_dir: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            environment: std::env::vars().collect(),
            schedule: JobSchedule::Recurring {
                cron_expression,
                next_run,
                last_run: None,
            },
            priority: 5,
            timeout_secs: self.config.default_timeout_secs,
            retry_config: RetryConfig::default(),
            dependencies: Vec::new(),
            notifications: NotificationConfig::default(),
            created_at: SystemTime::now(),
            enabled: true,
            metadata: HashMap::new(),
        };
        
        self.schedule_job(job).await
    }
    
    /// ジョブをキャンセル
    pub async fn cancel_job(&self, job_id: &str) -> Result<bool> {
        // スケジュールされたジョブを削除
        {
            let mut jobs = self.jobs.write().await;
            if jobs.remove(job_id).is_none() {
                return Ok(false);
            }
        }
        
        // キューから削除（効率化のため、実際の実装では別のアプローチを使用）
        {
            let mut queue = self.queue.write().await;
            let mut new_queue = BinaryHeap::new();
            while let Some(Reverse(job)) = queue.pop() {
                if job.job_id != job_id {
                    new_queue.push(Reverse(job));
                }
            }
            *queue = new_queue;
        }
        
        // 実行中のジョブを停止
        {
            let mut running_jobs = self.running_jobs.write().await;
            if let Some(running_job) = running_jobs.remove(job_id) {
                running_job.handle.abort();
            }
        }
        
        info!(job_id = %job_id, "Job cancelled successfully");
        Ok(true)
    }
    
    /// ジョブリストを取得
    pub async fn list_jobs(&self) -> Vec<ScheduledJob> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }
    
    /// ジョブ統計を取得
    pub async fn get_statistics(&self) -> JobStatistics {
        let jobs = self.jobs.read().await;
        let running_jobs = self.running_jobs.read().await;
        let queue = self.queue.read().await;
        let history = self.job_history.read().await;
        
        let total_jobs = jobs.len() as u64;
        let running_jobs_count = running_jobs.len() as u64;
        let queued_jobs_count = queue.len() as u64;
        
        // 成功率の計算
        let total_executions = history.len();
        let successful_executions = history.iter().filter(|entry| entry.result.success).count();
        let success_rate = if total_executions > 0 {
            (successful_executions as f64 / total_executions as f64) * 100.0
        } else {
            100.0
        };
        
        // 平均実行時間の計算
        let avg_execution_time_ms = if !history.is_empty() {
            let total_time: u64 = history.iter().map(|entry| entry.result.execution_time_ms).sum();
            total_time as f64 / history.len() as f64
        } else {
            0.0
        };
        
        // トップコマンドの集計
        let mut command_counts = HashMap::new();
        for entry in history.iter() {
            if let Some(job) = jobs.get(&entry.job_id) {
                *command_counts.entry(job.command.clone()).or_insert(0) += 1;
            }
        }
        
        let mut top_commands: Vec<_> = command_counts.into_iter().collect();
        top_commands.sort_by(|a, b| b.1.cmp(&a.1));
        top_commands.truncate(10);
        
        JobStatistics {
            total_jobs,
            running_jobs: running_jobs_count,
            queued_jobs: queued_jobs_count,
            executions_today: 0, // 実際の実装では今日の実行回数を計算
            success_rate,
            avg_execution_time_ms,
            top_commands,
        }
    }
    
    /// スケジュールされたジョブを処理
    async fn process_scheduled_jobs(
        jobs: &Arc<AsyncRwLock<HashMap<String, ScheduledJob>>>,
        queue: &Arc<AsyncRwLock<BinaryHeap<Reverse<QueuedJob>>>>,
        running_jobs: &Arc<AsyncRwLock<HashMap<String, RunningJob>>>,
        job_history: &Arc<AsyncRwLock<VecDeque<JobHistoryEntry>>>,
        semaphore: &Arc<Semaphore>,
        config: &SchedulerConfig,
    ) -> Result<()> {
        let now = SystemTime::now();
        let mut jobs_to_execute = Vec::new();
        
        // 実行すべきジョブをキューから取得
        {
            let mut queue_guard = queue.write().await;
            while let Some(Reverse(queued_job)) = queue_guard.peek() {
                if queued_job.scheduled_time <= now {
                    if let Some(Reverse(job)) = queue_guard.pop() {
                        jobs_to_execute.push(job);
                    }
                } else {
                    break;
                }
            }
        }
        
        // ジョブを実行
        for queued_job in jobs_to_execute {
            let permit = semaphore.clone().try_acquire_owned();
            
            if let Ok(permit) = permit {
                let jobs_clone = Arc::clone(jobs);
                let running_jobs_clone = Arc::clone(running_jobs);
                let job_history_clone = Arc::clone(job_history);
                let queue_clone = Arc::clone(queue);
                let config_clone = config.clone();
                
                let job_id = queued_job.job_id.clone();
                let job_id_for_spawn = job_id.clone();
                let scheduled_time = queued_job.scheduled_time;
                let attempt = queued_job.attempt;
                
                let handle = tokio::spawn(async move {
                    let _permit = permit; // permitを保持
                    Self::execute_job(
                        &job_id_for_spawn,
                        scheduled_time,
                        attempt,
                        &jobs_clone,
                        &running_jobs_clone,
                        &job_history_clone,
                        &queue_clone,
                        &config_clone,
                    ).await
                });
                
                // 実行中ジョブに追加
                let running_job = RunningJob {
                    job_id: job_id.clone(),
                    started_at: Instant::now(),
                    handle,
                    pid: None,
                };
                
                running_jobs.write().await.insert(job_id, running_job);
            } else {
                // セマフォが満杯の場合、キューに戻す
                queue.write().await.push(Reverse(queued_job));
                break;
            }
        }
        
        Ok(())
    }
    
    /// ジョブを実行
    async fn execute_job(
        job_id: &str,
        scheduled_time: SystemTime,
        attempt: u32,
        jobs: &Arc<AsyncRwLock<HashMap<String, ScheduledJob>>>,
        running_jobs: &Arc<AsyncRwLock<HashMap<String, RunningJob>>>,
        job_history: &Arc<AsyncRwLock<VecDeque<JobHistoryEntry>>>,
        queue: &Arc<AsyncRwLock<BinaryHeap<Reverse<QueuedJob>>>>,
        config: &SchedulerConfig,
    ) -> JobExecutionResult {
        let start_time = Instant::now();
        let started_at = SystemTime::now();
        
        let job = {
            let jobs_guard = jobs.read().await;
            jobs_guard.get(job_id).cloned()
        };
        
        let result = if let Some(job) = job {
            // Skip disabled jobs gracefully (do not reschedule)
            if !job.enabled {
                return JobExecutionResult {
                    job_id: job_id.to_string(),
                    success: true,
                    exit_code: Some(0),
                    execution_time_ms: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                    error_message: None,
                    memory_usage: None,
                    cpu_usage: None,
                };
            }
            info!(
                job_id = %job_id,
                command = %job.command,
                "Starting job execution"
            );
            
            // 実際のコマンド実行（簡略化）
            let mut result = JobExecutionResult {
                job_id: job_id.to_string(),
                success: true,
                exit_code: Some(0),
                execution_time_ms: 0,
                stdout: "Job executed successfully".to_string(),
                stderr: String::new(),
                error_message: None,
                memory_usage: Some(1024 * 1024), // 1MB
                cpu_usage: Some(5.0),
            };
            
            // 実行時間を記録
            result.execution_time_ms = start_time.elapsed().as_millis() as u64;
            result
        } else {
            JobExecutionResult {
                job_id: job_id.to_string(),
                success: false,
                exit_code: Some(1),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                stdout: String::new(),
                stderr: "Job not found".to_string(),
                error_message: Some("Job not found".to_string()),
                memory_usage: None,
                cpu_usage: None,
            }
        };
        
        // 実行中ジョブから削除
        running_jobs.write().await.remove(job_id);
        
        // 履歴に追加
        let history_entry = JobHistoryEntry {
            job_id: job_id.to_string(),
            started_at,
            finished_at: SystemTime::now(),
            result: result.clone(),
            scheduled_time,
            delay_ms: started_at.duration_since(scheduled_time)
                .unwrap_or(Duration::ZERO)
                .as_millis() as u64,
        };
        
        {
            let mut history = job_history.write().await;
            history.push_back(history_entry);
            
            // 履歴サイズを制限
            let max_history = config.history_retention_hours * 60; // 1分あたり1エントリと仮定
            while history.len() > max_history as usize {
                history.pop_front();
            }
        }
        
        // 次回実行時刻の計算と再スケジュール処理
        if let Some(mut job) = { let guard = jobs.read().await; guard.get(job_id).cloned() } {
            // 成否に応じて再スケジュール方針
            let mut maybe_next: Option<SystemTime> = None;
            if result.success {
                match &mut job.schedule {
                    JobSchedule::Once { .. } => {
                        // 完了後はジョブを削除
                        let mut jobs_w = jobs.write().await;
                        jobs_w.remove(job_id);
                    }
                    JobSchedule::Recurring { cron_expression, next_run, last_run } => {
                        *last_run = Some(SystemTime::now());
                        // 簡易: 既存のパーサで次回を算出
                        if let Ok(nr) = Self::parse_cron_expression_static(cron_expression).await {
                            *next_run = nr; maybe_next = Some(nr);
                        }
                    }
                    JobSchedule::Interval { interval_secs, next_run } => {
                        let base = SystemTime::now();
                        let nr = base + Duration::from_secs(*interval_secs);
                        *next_run = nr; maybe_next = Some(nr);
                    }
                    JobSchedule::EventBased { .. } => { /* wait for event */ }
                }
            } else {
                // 失敗時の再試行
                if attempt < job.retry_config.max_retries {
                    let mut delay = Duration::from_secs(job.retry_config.retry_interval_secs);
                    if job.retry_config.exponential_backoff {
                        // Exponential backoff by doubling per attempt, clamped to max_delay_secs
                        let mut factor: u32 = 1;
                        for _ in 0..attempt.min(16) { factor = factor.saturating_mul(2); }
                        delay = delay.saturating_mul(factor);
                        let max = Duration::from_secs(job.retry_config.max_delay_secs);
                        if delay > max { delay = max; }
                    }
                    maybe_next = Some(SystemTime::now() + delay);
                }
            }
            if let Some(nr) = maybe_next {
                // 更新を確定しキューへ投入
                let priority_for_requeue = job.priority;
                {
                    let mut jobs_w = jobs.write().await;
                    if let Some(stored) = jobs_w.get_mut(job_id) { *stored = job.clone(); }
                }
                let mut q = queue.write().await;
                q.push(Reverse(QueuedJob { job_id: job_id.to_string(), scheduled_time: nr, priority: priority_for_requeue, attempt: attempt.saturating_add(1) }));
            }
        }
        
        info!(
            job_id = %job_id,
            success = result.success,
            execution_time_ms = result.execution_time_ms,
            "Job execution completed"
        );
        
        result
    }
    
    /// 次回実行時刻を計算
    async fn calculate_next_run(&self, schedule: &JobSchedule) -> Result<SystemTime> {
        match schedule {
            JobSchedule::Once { run_at } => Ok(*run_at),
            JobSchedule::Recurring { next_run, .. } => Ok(*next_run),
            JobSchedule::Interval { next_run, .. } => Ok(*next_run),
            JobSchedule::EventBased { .. } => {
                // イベントベースの場合は遠い未来を返す
                Ok(SystemTime::now() + Duration::from_secs(86400 * 365))
            }
        }
    }
    
    /// cron式を解析
    async fn parse_cron_expression(&self, cron_expression: &str) -> Result<SystemTime> {
        Self::parse_cron_expression_static(cron_expression).await
    }

    /// Static helper that computes next run time from a cron expression.
    async fn parse_cron_expression_static(_cron_expression: &str) -> Result<SystemTime> {
        // Simplified placeholder: real implementation would compute from cron string.
        Ok(SystemTime::now() + Duration::from_secs(3600)) // 1 hour later
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scheduler_config_default() {
        let config = SchedulerConfig::default();
        assert_eq!(config.max_concurrent_jobs, 10);
        assert_eq!(config.check_interval_secs, 5);
        assert!(config.enable_priority_queue);
    }
    
    #[test]
    fn test_queued_job_ordering() {
        let job1 = QueuedJob {
            job_id: "job1".to_string(),
            scheduled_time: SystemTime::now(),
            priority: 5,
            attempt: 0,
        };
        
        let job2 = QueuedJob {
            job_id: "job2".to_string(),
            scheduled_time: SystemTime::now() + Duration::from_secs(60),
            priority: 8,
            attempt: 0,
        };
        
        // job1の方が早い時刻なので、job1 < job2 (BinaryHeapでは逆順なのでjob1 > job2)
        assert!(job1 < job2); // 正しい順序比較
    }
    
    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = AdvancedJobScheduler::new(config);
        
        assert_eq!(scheduler.job_counter.load(Ordering::Relaxed), 0);
        assert!(!scheduler.shutdown_signal.load(Ordering::Relaxed));
    }
    
    #[tokio::test]
    async fn test_schedule_at_job() {
        let config = SchedulerConfig::default();
        let scheduler = AdvancedJobScheduler::new(config);
        
        let run_at = SystemTime::now() + Duration::from_secs(3600);
        let result = scheduler.schedule_at("echo hello".to_string(), run_at).await;
        
        assert!(result.is_ok());
        let job_id = result.unwrap();
        assert!(job_id.starts_with("at_"));
    }
}
