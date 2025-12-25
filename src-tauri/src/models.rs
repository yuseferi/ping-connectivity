use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration for a ping target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingTarget {
    pub id: String,
    pub address: String,
    pub label: String,
    pub enabled: bool,
}

impl PingTarget {
    pub fn new(address: String, label: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            address,
            label,
            enabled: true,
        }
    }

    /// Create default targets
    pub fn defaults() -> Vec<Self> {
        vec![
            Self::new("1.1.1.1".to_string(), "Cloudflare DNS".to_string()),
            Self::new("8.8.8.8".to_string(), "Google DNS".to_string()),
        ]
    }

    /// Preset targets for quick add
    pub fn presets() -> Vec<Self> {
        vec![
            Self::new("1.1.1.1".to_string(), "Cloudflare DNS".to_string()),
            Self::new("8.8.8.8".to_string(), "Google DNS".to_string()),
            Self::new("9.9.9.9".to_string(), "Quad9 DNS".to_string()),
            Self::new("208.67.222.222".to_string(), "OpenDNS".to_string()),
        ]
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub targets: Vec<PingTarget>,
    pub ping_interval_ms: u64,
    pub timeout_ms: u64,
    pub max_history_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            targets: PingTarget::defaults(),
            ping_interval_ms: 1000,
            timeout_ms: 5000,
            max_history_size: 100,
        }
    }
}

/// Result of a single ping operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub timestamp: DateTime<Utc>,
    pub target: String,
    pub target_label: String,
    pub latency_ms: Option<f64>,
    pub success: bool,
    pub sequence: u32,
    pub error: Option<String>,
}

impl PingResult {
    pub fn success(target: &PingTarget, latency_ms: f64, sequence: u32) -> Self {
        Self {
            timestamp: Utc::now(),
            target: target.address.clone(),
            target_label: target.label.clone(),
            latency_ms: Some(latency_ms),
            success: true,
            sequence,
            error: None,
        }
    }

    pub fn failure(target: &PingTarget, error: String, sequence: u32) -> Self {
        Self {
            timestamp: Utc::now(),
            target: target.address.clone(),
            target_label: target.label.clone(),
            latency_ms: None,
            success: false,
            sequence,
            error: Some(error),
        }
    }
}

/// Statistics for a specific target
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PingStatistics {
    pub target: String,
    pub target_label: String,
    pub total_pings: u64,
    pub successful_pings: u64,
    pub failed_pings: u64,
    pub packet_loss_percent: f64,
    pub min_latency_ms: Option<f64>,
    pub max_latency_ms: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub session_start: Option<DateTime<Utc>>,
    pub last_ping: Option<DateTime<Utc>>,
}

impl PingStatistics {
    pub fn new(target: &PingTarget) -> Self {
        Self {
            target: target.address.clone(),
            target_label: target.label.clone(),
            ..Default::default()
        }
    }
}

/// Event payload for ping results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResultEvent {
    pub result: PingResult,
}

/// Event payload for statistics updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsUpdateEvent {
    pub stats: Vec<PingStatistics>,
}

/// Application state for pinging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PingState {
    Stopped,
    Running,
    Paused,
}

impl Default for PingState {
    fn default() -> Self {
        Self::Stopped
    }
}
