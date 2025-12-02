//! Performance monitoring and metrics collection for Super IDE

use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f32,
    pub memory_usage_mb: f32,
    pub active_documents: usize,
    pub ai_requests_per_minute: u32,
    pub response_time_ms: VecDeque<f32>,
    pub file_operations_per_second: u32,
    pub network_requests: u32,
    pub error_count: u32,
    pub uptime_seconds: u64,
}

/// Performance counters
#[derive(Debug, Default)]
pub struct PerformanceCounters {
    pub ai_requests: u64,
    pub file_operations: u64,
    pub network_requests: u64,
    pub errors: u64,
    pub memory_allocations: u64,
    pub cpu_cycles: u64,
}

/// Performance monitor
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    counters: Arc<Mutex<PerformanceCounters>>,
    start_time: Instant,
    sampling_interval: Duration,
    cpu_monitor: Option<CpuMonitor>,
    memory_monitor: Option<MemoryMonitor>,
}

/// CPU usage monitor
struct CpuMonitor {
    last_cpu_time: Option<u64>, // Simplified: just store raw counter
}

/// Memory usage monitor
struct MemoryMonitor {
    last_memory_info: Option<u64>, // Simplified: just store raw counter
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        let initial_metrics = PerformanceMetrics {
            cpu_usage: 0.0,
            memory_usage_mb: 0.0,
            active_documents: 0,
            ai_requests_per_minute: 0,
            response_time_ms: VecDeque::new(),
            file_operations_per_second: 0,
            network_requests: 0,
            error_count: 0,
            uptime_seconds: 0,
        };
        
        Self {
            metrics: Arc::new(RwLock::new(initial_metrics)),
            counters: Arc::new(Mutex::new(PerformanceCounters::default())),
            start_time: Instant::now(),
            sampling_interval: Duration::from_secs(1),
            cpu_monitor: None,
            memory_monitor: None,
        }
    }
    
    /// Start monitoring
    pub async fn start(&mut self) {
        // Initialize system monitors
        self.initialize_system_monitors();
        
        // Start sampling task
        let metrics_clone = self.metrics.clone();
        let counters_clone = self.counters.clone();
        let interval = self.sampling_interval;
        let start_time_clone = self.start_time;
        let cpu_monitor_clone = self.cpu_monitor.clone();
        let memory_monitor_clone = self.memory_monitor.clone();
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Update metrics
                let mut metrics = metrics_clone.write().await;
                let counters = counters_clone.lock().await;
                
                // Update uptime
                metrics.uptime_seconds = start_time_clone.elapsed().as_secs();
                
                // Update system metrics
                if let Some(cpu_monitor) = &cpu_monitor_clone {
                    metrics.cpu_usage = cpu_monitor.get_cpu_usage();
                }
                
                if let Some(memory_monitor) = &memory_monitor_clone {
                    metrics.memory_usage_mb = memory_monitor.get_memory_usage();
                }
                
                // Update operational metrics
                metrics.ai_requests_per_minute = (counters.ai_requests * 60) as u32;
                metrics.file_operations_per_second = counters.file_operations as u32;
                metrics.network_requests = counters.network_requests;
                metrics.error_count = counters.errors;
                
                // Calculate rolling average response time
                if !metrics.response_time_ms.is_empty() {
                    let sum: f32 = metrics.response_time_ms.iter().sum();
                    let avg_response_time = sum / metrics.response_time_ms.len() as f32;
                    // Store average or keep rolling window
                }
            }
        });
    }
    
    /// Record an AI request
    pub async fn record_ai_request(&self, duration: Duration) {
        let mut counters = self.counters.lock().await;
        counters.ai_requests += 1;
        
        // Record response time
        let mut metrics = self.metrics.write().await;
        metrics.response_time_ms.push_back(duration.as_secs_f32() * 1000.0);
        
        // Keep only last 100 response times
        while metrics.response_time_ms.len() > 100 {
            metrics.response_time_ms.pop_front();
        }
    }
    
    /// Record a file operation
    pub async fn record_file_operation(&self) {
        let mut counters = self.counters.lock().await;
        counters.file_operations += 1;
    }
    
    /// Record a network request
    pub async fn record_network_request(&self) {
        let mut counters = self.counters.lock().await;
        counters.network_requests += 1;
    }
    
    /// Record an error
    pub async fn record_error(&self) {
        let mut counters = self.counters.lock().await;
        counters.errors += 1;
    }
    
    /// Update active document count
    pub async fn set_active_documents(&self, count: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.active_documents = count;
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Get performance report
    pub async fn get_performance_report(&self) -> PerformanceReport {
        let metrics = self.get_metrics().await;
        let counters = self.counters.lock().await;
        
        PerformanceReport {
            system_metrics: SystemMetrics {
                cpu_usage: metrics.cpu_usage,
                memory_usage_mb: metrics.memory_usage_mb,
                uptime_seconds: metrics.uptime_seconds,
            },
            operational_metrics: OperationalMetrics {
                ai_requests_total: counters.ai_requests,
                file_operations_total: counters.file_operations,
                network_requests_total: counters.network_requests,
                errors_total: counters.errors,
                ai_requests_per_minute: metrics.ai_requests_per_minute,
                file_operations_per_second: metrics.file_operations_per_second,
            },
            performance_indicators: PerformanceIndicators {
                avg_response_time_ms: if metrics.response_time_ms.is_empty() {
                    0.0
                } else {
                    metrics.response_time_ms.iter().sum::<f32>() / metrics.response_time_ms.len() as f32
                },
                error_rate: if counters.ai_requests > 0 {
                    (counters.errors as f32 / counters.ai_requests as f32) * 100.0
                } else {
                    0.0
                },
                efficiency_score: self.calculate_efficiency_score(&metrics, &counters),
            },
            recommendations: self.generate_recommendations(&metrics, &counters).await,
        }
    }
    
    /// Initialize system monitoring
    fn initialize_system_monitors(&mut self) {
        // Initialize CPU monitor
        #[cfg(target_os = "linux")]
        {
            self.cpu_monitor = Some(CpuMonitor::new());
        }
        
        // Initialize memory monitor
        #[cfg(feature = "sysinfo")]
        {
            self.memory_monitor = Some(MemoryMonitor::new());
        }
    }
    
    /// Calculate efficiency score
    fn calculate_efficiency_score(&self, metrics: &PerformanceMetrics, counters: &PerformanceCounters) -> f32 {
        let mut score = 100.0;
        
        // Deduct points for high CPU usage
        if metrics.cpu_usage > 80.0 {
            score -= (metrics.cpu_usage - 80.0) * 0.5;
        }
        
        // Deduct points for high memory usage
        if metrics.memory_usage_mb > 1000.0 {
            score -= (metrics.memory_usage_mb - 1000.0) / 100.0;
        }
        
        // Deduct points for high error rate
        if counters.ai_requests > 0 {
            let error_rate = (counters.errors as f32 / counters.ai_requests as f32) * 100.0;
            score -= error_rate;
        }
        
        score.max(0.0).min(100.0)
    }
    
    /// Generate performance recommendations
    async fn generate_recommendations(&self, metrics: &PerformanceMetrics, counters: &PerformanceCounters) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if metrics.cpu_usage > 80.0 {
            recommendations.push("High CPU usage detected. Consider reducing the number of active documents or disabling some AI features.".to_string());
        }
        
        if metrics.memory_usage_mb > 1000.0 {
            recommendations.push("High memory usage detected. Consider closing unused documents or increasing system memory.".to_string());
        }
        
        if counters.errors > counters.ai_requests / 10 {
            recommendations.push("High error rate detected. Check AI configuration and network connectivity.".to_string());
        }
        
        if metrics.response_time_ms.len() > 10 {
            let recent_avg = metrics.response_time_ms.iter().rev().take(10).sum::<f32>() / 10.0;
            if recent_avg > 1000.0 {
                recommendations.push("Slow AI response times detected. Consider using a local model or optimizing prompts.".to_string());
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("System performance is optimal.".to_string());
        }
        
        recommendations
    }
    
    /// Start monitoring a specific operation
    pub async fn start_operation_monitoring(&self) -> OperationMonitor {
        OperationMonitor {
            start_time: Instant::now(),
            metrics: self.metrics.clone(),
        }
    }
}

/// Operation monitor for timing specific operations
pub struct OperationMonitor {
    start_time: Instant,
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl OperationMonitor {
    /// Finish monitoring and record the operation
    pub async fn finish(self, operation_type: OperationType) {
        let duration = self.start_time.elapsed();
        
        match operation_type {
            OperationType::AiRequest => {
                let mut counters = self.metrics.write().await;
                counters.ai_requests += 1;
            }
            OperationType::FileOperation => {
                let mut counters = self.metrics.write().await;
                counters.file_operations += 1;
            }
            OperationType::NetworkRequest => {
                let mut counters = self.metrics.write().await;
                counters.network_requests += 1;
            }
        }
    }
}

/// Types of operations being monitored
#[derive(Debug, Clone)]
pub enum OperationType {
    AiRequest,
    FileOperation,
    NetworkRequest,
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub system_metrics: SystemMetrics,
    pub operational_metrics: OperationalMetrics,
    pub performance_indicators: PerformanceIndicators,
    pub recommendations: Vec<String>,
}

/// System-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage_mb: f32,
    pub uptime_seconds: u64,
}

/// Operational metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalMetrics {
    pub ai_requests_total: u64,
    pub file_operations_total: u64,
    pub network_requests_total: u64,
    pub errors_total: u64,
    pub ai_requests_per_minute: u32,
    pub file_operations_per_second: u32,
}

/// Performance indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIndicators {
    pub avg_response_time_ms: f32,
    pub error_rate: f32,
    pub efficiency_score: f32,
}

// CPU Monitor Implementation
impl CpuMonitor {
    fn new() -> Self {
        Self {
            last_cpu_time: None,
        }
    }
    
    fn get_cpu_usage(&mut self) -> f32 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(cpu_times) = std::fs::read_to_string("/proc/stat") {
                let lines: Vec<&str> = cpu_times.lines().collect();
                if !lines.is_empty() {
                    let parts: Vec<u64> = lines[0]
                        .split_whitespace()
                        .skip(1)
                        .take(5)
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    
                    if parts.len() >= 5 {
                        let total = parts.iter().sum::<u64>() as f32;
                        let idle = parts[3] as f32;
                        let usage = ((total - idle) / total) * 100.0;
                        return usage.max(0.0).min(100.0);
                    }
                }
            }
        }
        
        // Fallback for non-Linux systems
        0.0
    }
}

// Memory Monitor Implementation
impl MemoryMonitor {
    fn new() -> Self {
        Self {
            last_memory_info: None,
        }
    }
    
    fn get_memory_usage(&mut self) -> f32 {
        #[cfg(feature = "sysinfo")]
        {
            use sysinfo::{System, SystemExt};
            
            let mut sys = System::new_all();
            sys.refresh_all();
            
            sys.used_memory() as f32 / (1024.0 * 1024.0) // Convert to MB
        }
        
        #[cfg(not(feature = "sysinfo"))]
        {
            // Fallback implementation
            0.0
        }
    }
}

// Global performance monitor instance
static GLOBAL_PERFORMANCE_MONITOR: Lazy<PerformanceMonitor> = Lazy::new(|| {
    PerformanceMonitor::new()
});

/// Get the global performance monitor instance
pub fn global_performance_monitor() -> &'static PerformanceMonitor {
    &GLOBAL_PERFORMANCE_MONITOR
}