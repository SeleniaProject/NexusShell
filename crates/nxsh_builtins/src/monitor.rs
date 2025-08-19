//! `monitor` command - Advanced system monitoring dashboard
//! Real-time system monitoring with beautiful visualizations

use anyhow::Result;
use crate::ui_design::{
    TableFormatter, Colorize, Animation, ProgressBar, create_advanced_table,
    TableOptions, BorderStyle, Alignment, ItemStatus, StatusItem, StatusDashboard, DashboardSection, SectionStyle
};
use std::time::{Duration, Instant};
use std::thread;

#[derive(Debug, Clone)]
pub struct SystemMonitor {
    pub update_interval: Duration,
    pub auto_refresh: bool,
    pub show_graphs: bool,
    pub compact_mode: bool,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub load_average: (f64, f64, f64),
    pub process_count: u32,
    pub uptime: Duration,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            update_interval: Duration::from_secs(1),
            auto_refresh: false,
            show_graphs: true,
            compact_mode: false,
        }
    }
    
    pub fn run_dashboard(&self) -> Result<()> {
        println!("{}", "🖥️  Starting System Monitor Dashboard...".primary());
        let _animation = Animation::spinner();
        println!("Initializing monitoring systems...");
        
        let mut iteration = 0;
        let start_time = Instant::now();
        
        loop {
            // Clear screen for refresh
            if iteration > 0 {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
            }
            
            let metrics = self.collect_metrics()?;
            self.render_dashboard(&metrics, iteration, start_time.elapsed())?;
            
            if !self.auto_refresh {
                println!("\n{} Press Ctrl+C to exit, Enter to refresh manually...", "💡".info());
                
                // Non-blocking input check
                use std::io::{self, Read};
                let mut buffer = [0; 1];
                match std::io::stdin().read(&mut buffer) {
                    Ok(_) => {
                        if buffer[0] == 3 { // Ctrl+C
                            break;
                        }
                    },
                    Err(_) => break,
                }
            } else {
                thread::sleep(self.update_interval);
            }
            
            iteration += 1;
            
            // Auto-exit after 100 iterations to prevent infinite loop
            if iteration > 100 {
                break;
            }
        }
        
        println!("\n{}", "👋 System monitoring stopped.".success());
        Ok(())
    }
    
    fn collect_metrics(&self) -> Result<SystemMetrics> {
        // Simulate collecting real system metrics
        // In a real implementation, this would use system APIs
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        Ok(SystemMetrics {
            cpu_usage: rng.gen_range(10.0..90.0),
            memory_usage: rng.gen_range(30.0..80.0),
            disk_usage: rng.gen_range(40.0..95.0),
            network_rx: rng.gen_range(1000..1000000),
            network_tx: rng.gen_range(500..500000),
            load_average: (
                rng.gen_range(0.1..2.0),
                rng.gen_range(0.1..2.0),
                rng.gen_range(0.1..2.0),
            ),
            process_count: rng.gen_range(150..300),
            uptime: Duration::from_secs(rng.gen_range(3600..86400)),
        })
    }
    
    fn render_dashboard(&self, metrics: &SystemMetrics, iteration: u32, elapsed: Duration) -> Result<()> {
        let mut dashboard = StatusDashboard::new("System Monitor Dashboard".to_string());
        
        // System Overview Section
        let mut overview_section = DashboardSection {
            title: "🖥️  System Overview".to_string(),
            style: SectionStyle::Boxed,
            items: Vec::new(),
        };
        
        overview_section.items.push(StatusItem {
            name: "cpu_usage".to_string(),
            label: "CPU Usage".to_string(),
            value: format!("{:.1}%", metrics.cpu_usage),
            status: self.get_cpu_status(metrics.cpu_usage),
            icon: "🔥".to_string(),
        });
        
        overview_section.items.push(StatusItem {
            name: "memory_usage".to_string(),
            label: "Memory Usage".to_string(),
            value: format!("{:.1}%", metrics.memory_usage),
            status: self.get_memory_status(metrics.memory_usage),
            icon: "🧠".to_string(),
        });
        
        overview_section.items.push(StatusItem {
            name: "disk_usage".to_string(),
            label: "Disk Usage".to_string(),
            value: format!("{:.1}%", metrics.disk_usage),
            status: self.get_disk_status(metrics.disk_usage),
            icon: "💾".to_string(),
        });
        
        overview_section.items.push(StatusItem {
            name: "process_count".to_string(),
            label: "Process Count".to_string(),
            value: metrics.process_count.to_string(),
            status: ItemStatus::Info,
            icon: "⚡".to_string(),
        });
        
        dashboard.add_section(overview_section);
        
        // Performance Section
        let mut performance_section = DashboardSection {
            title: "📊 Performance Metrics".to_string(),
            style: SectionStyle::Highlighted,
            items: Vec::new(),
        };
        
        performance_section.items.push(StatusItem {
            name: "load_average".to_string(),
            label: "Load Average".to_string(),
            value: format!("{:.2}, {:.2}, {:.2}", 
                metrics.load_average.0,
                metrics.load_average.1,
                metrics.load_average.2
            ),
            status: self.get_load_status(metrics.load_average.0),
            icon: "⚖️".to_string(),
        });
        
        performance_section.items.push(StatusItem {
            name: "uptime".to_string(),
            label: "Uptime".to_string(),
            value: self.format_uptime(metrics.uptime),
            status: ItemStatus::Good,
            icon: "⏰".to_string(),
        });
        
        dashboard.add_section(performance_section);
        
        // Network Section
        let mut network_section = DashboardSection {
            title: "🌐 Network Activity".to_string(),
            style: SectionStyle::Simple,
            items: Vec::new(),
        };
        
        network_section.items.push(StatusItem {
            name: "network_rx".to_string(),
            label: "Received".to_string(),
            value: bytesize::ByteSize::b(metrics.network_rx).to_string(),
            status: ItemStatus::Info,
            icon: "⬇️".to_string(),
        });
        
        network_section.items.push(StatusItem {
            name: "network_tx".to_string(),
            label: "Transmitted".to_string(),
            value: bytesize::ByteSize::b(metrics.network_tx).to_string(),
            status: ItemStatus::Info,
            icon: "⬆️".to_string(),
        });
        
        dashboard.add_section(network_section);
        
        // Render the dashboard
        println!("{}", dashboard.render());
        
        // Show visual usage bars if enabled
        if self.show_graphs {
            self.render_usage_bars(metrics)?;
        }
        
        // Show monitoring info
        println!("\n{}", "📈 Monitoring Status".info());
        println!("   • Update #{}: {:.1}s elapsed", 
            iteration.to_string().primary(),
            elapsed.as_secs_f32().to_string().info()
        );
        println!("   • Refresh Rate: {}s", self.update_interval.as_secs().to_string().info());
        println!("   • Auto Refresh: {}", 
            if self.auto_refresh { "Enabled".success() } else { "Disabled".dim() }
        );
        
        Ok(())
    }
    
    fn render_usage_bars(&self, metrics: &SystemMetrics) -> Result<()> {
        println!("\n{}", "📊 Resource Utilization".primary());
        println!("{}", "─".repeat(50).dim());
        
        self.render_usage_bar("CPU", metrics.cpu_usage, 100.0)?;
        self.render_usage_bar("Memory", metrics.memory_usage, 100.0)?;
        self.render_usage_bar("Disk", metrics.disk_usage, 100.0)?;
        
        Ok(())
    }
    
    fn render_usage_bar(&self, label: &str, value: f64, max_value: f64) -> Result<()> {
        let bar_width = 30;
        let percentage = (value / max_value * 100.0).min(100.0);
        let filled = (percentage / 100.0 * bar_width as f64) as usize;
        let empty = bar_width - filled;
        
        let bar = match percentage {
            p if p > 90.0 => format!("[{}{}]", "█".repeat(filled).error(), "░".repeat(empty).dim()),
            p if p > 70.0 => format!("[{}{}]", "█".repeat(filled).warning(), "░".repeat(empty).dim()),
            _ => format!("[{}{}]", "█".repeat(filled).success(), "░".repeat(empty).dim()),
        };
        
        println!("{:>8}: {} {:.1}%", 
            label.info(),
            bar,
            percentage.to_string().primary()
        );
        
        Ok(())
    }
    
    fn get_cpu_status(&self, usage: f64) -> ItemStatus {
        match usage {
            u if u > 90.0 => ItemStatus::Error,
            u if u > 70.0 => ItemStatus::Warning,
            _ => ItemStatus::Good,
        }
    }
    
    fn get_memory_status(&self, usage: f64) -> ItemStatus {
        match usage {
            u if u > 85.0 => ItemStatus::Error,
            u if u > 70.0 => ItemStatus::Warning,
            _ => ItemStatus::Good,
        }
    }
    
    fn get_disk_status(&self, usage: f64) -> ItemStatus {
        match usage {
            u if u > 95.0 => ItemStatus::Error,
            u if u > 80.0 => ItemStatus::Warning,
            _ => ItemStatus::Good,
        }
    }
    
    fn get_load_status(&self, load: f64) -> ItemStatus {
        match load {
            l if l > 2.0 => ItemStatus::Warning,
            l if l > 4.0 => ItemStatus::Error,
            _ => ItemStatus::Good,
        }
    }
    
    fn format_uptime(&self, uptime: Duration) -> String {
        let total_seconds = uptime.as_secs();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        
        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
    
    pub fn show_process_table(&self) -> Result<()> {
        println!("\n{}", "⚡ Top Processes".primary());
        println!("{}", "═".repeat(60).dim());
        
        // Simulate process data
        let headers = vec!["PID", "Name", "CPU%", "Memory%", "Status"];
        let mut rows = vec![
            vec!["1234".primary(), "firefox".info(), "15.2%".warning(), "8.5%".info(), "Running".success()],
            vec!["5678".primary(), "code".info(), "12.1%".warning(), "12.3%".warning(), "Running".success()],
            vec!["9012".primary(), "cargo".info(), "8.9%".info(), "4.2%".info(), "Running".success()],
            vec!["3456".primary(), "systemd".info(), "0.1%".success(), "0.5%".success(), "Sleeping".dim()],
            vec!["7890".primary(), "bash".info(), "0.3%".success(), "1.2%".success(), "Running".success()],
        ];
        
        let options = TableOptions {
            show_borders: true,
            zebra_striping: false,
            compact_mode: false,
            max_width: None,
            show_header: true,
            alternating_rows: true,
            align_columns: true,
            compact: false,
            border_style: BorderStyle::Rounded,
            header_alignment: Alignment::Center,
        };
        
        println!("{}", create_advanced_table(&headers, &rows, options));
        
        Ok(())
    }
}

pub fn monitor_cli(args: &[String]) -> Result<()> {
    let mut monitor = SystemMonitor::new();
    
    // Parse arguments
    for arg in args {
        match arg.as_str() {
            "--auto" | "-a" => monitor.auto_refresh = true,
            "--no-graphs" | "-n" => monitor.show_graphs = false,
            "--compact" | "-c" => monitor.compact_mode = true,
            "--interval" | "-i" => {
                // In a real implementation, parse the next argument as interval
                monitor.update_interval = Duration::from_secs(2);
            },
            "processes" | "proc" => {
                monitor.show_process_table()?;
                return Ok(());
            },
            "help" | "-h" | "--help" => {
                show_monitor_help()?;
                return Ok(());
            },
            _ => {}
        }
    }
    
    // Run the monitoring dashboard
    monitor.run_dashboard()?;
    
    Ok(())
}

fn show_monitor_help() -> Result<()> {
    println!("\n{}", "🖥️  System Monitor Help".primary());
    println!("{}", "═".repeat(50).dim());
    
    println!("\n{}", "Usage:".info());
    println!("  monitor [OPTIONS] [COMMAND]");
    
    println!("\n{}", "Options:".info());
    println!("  -a, --auto        Auto-refresh mode");
    println!("  -n, --no-graphs   Disable usage graphs");
    println!("  -c, --compact     Compact display mode");
    println!("  -i, --interval    Update interval in seconds");
    println!("  -h, --help        Show this help message");
    
    println!("\n{}", "Commands:".info());
    println!("  processes, proc   Show process table only");
    
    println!("\n{}", "Interactive Controls:".info());
    println!("  Ctrl+C           Exit monitoring");
    println!("  Enter            Manual refresh (non-auto mode)");
    
    println!("\n{}", "Examples:".info());
    println!("  monitor");
    println!("  monitor --auto");
    println!("  monitor processes");
    println!("  monitor --compact --no-graphs");
    
    Ok(())
}
