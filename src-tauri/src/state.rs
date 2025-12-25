use crate::logging::JsonLogger;
use crate::models::{AppConfig, PingResult, PingState, PingStatistics, PingTarget};
use crate::ping::Pinger;
use crate::stats::StatsCalculator;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::broadcast;

/// Application state shared across the application
pub struct AppState {
    /// Current configuration
    pub config: RwLock<AppConfig>,
    /// Statistics calculator
    pub stats: RwLock<StatsCalculator>,
    /// JSON logger
    pub logger: RwLock<Option<JsonLogger>>,
    /// Current ping state
    pub ping_state: RwLock<PingState>,
    /// Recent ping results (for chart display)
    pub recent_results: RwLock<VecDeque<PingResult>>,
    /// Sequence counter for pings
    pub sequence: AtomicU32,
    /// Channel to signal stop
    pub stop_signal: RwLock<Option<broadcast::Sender<()>>>,
}

impl AppState {
    pub fn new() -> Self {
        let config = AppConfig::default();
        let mut stats = StatsCalculator::new();
        
        // Initialize stats for default targets
        for target in &config.targets {
            stats.init_target(target);
        }
        
        // Initialize logger
        let logger = JsonLogger::new(JsonLogger::default_log_dir())
            .map_err(|e| log::error!("Failed to create logger: {}", e))
            .ok();
        
        Self {
            config: RwLock::new(config),
            stats: RwLock::new(stats),
            logger: RwLock::new(logger),
            ping_state: RwLock::new(PingState::Stopped),
            recent_results: RwLock::new(VecDeque::new()),
            sequence: AtomicU32::new(0),
            stop_signal: RwLock::new(None),
        }
    }

    /// Get the next sequence number
    pub fn next_sequence(&self) -> u32 {
        self.sequence.fetch_add(1, Ordering::SeqCst)
    }

    /// Reset sequence counter
    pub fn reset_sequence(&self) {
        self.sequence.store(0, Ordering::SeqCst);
    }

    /// Add a ping result
    pub fn add_result(&self, result: PingResult) {
        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.update(&result);
        }
        
        // Log the result
        {
            let logger = self.logger.read();
            if let Some(ref logger) = *logger {
                if let Err(e) = logger.log(&result) {
                    log::error!("Failed to log ping result: {}", e);
                }
            }
        }
        
        // Add to recent results
        {
            let config = self.config.read();
            let max_size = config.max_history_size;
            drop(config);
            
            let mut recent = self.recent_results.write();
            recent.push_back(result);
            while recent.len() > max_size {
                recent.pop_front();
            }
        }
    }

    /// Get recent ping results
    pub fn get_recent_results(&self, count: Option<usize>) -> Vec<PingResult> {
        let recent = self.recent_results.read();
        let count = count.unwrap_or(recent.len());
        recent.iter().rev().take(count).cloned().collect()
    }

    /// Get all statistics
    pub fn get_all_stats(&self) -> Vec<PingStatistics> {
        self.stats.read().get_all_stats()
    }

    /// Get statistics for a specific target
    pub fn get_stats_for_target(&self, target: &str) -> Option<PingStatistics> {
        self.stats.read().get_stats(target)
    }

    /// Get current configuration
    pub fn get_config(&self) -> AppConfig {
        self.config.read().clone()
    }

    /// Update configuration
    pub fn update_config(&self, config: AppConfig) {
        // Update stats calculator with new targets
        {
            let mut stats = self.stats.write();
            for target in &config.targets {
                stats.init_target(target);
            }
        }
        
        *self.config.write() = config;
    }

    /// Get all targets
    pub fn get_targets(&self) -> Vec<PingTarget> {
        self.config.read().targets.clone()
    }

    /// Get enabled targets
    pub fn get_enabled_targets(&self) -> Vec<PingTarget> {
        self.config.read()
            .targets
            .iter()
            .filter(|t| t.enabled)
            .cloned()
            .collect()
    }

    /// Add a new target
    pub fn add_target(&self, target: PingTarget) -> PingTarget {
        let mut config = self.config.write();
        let target_clone = target.clone();
        config.targets.push(target);
        
        // Initialize stats for the new target
        self.stats.write().init_target(&target_clone);
        
        target_clone
    }

    /// Remove a target by ID
    pub fn remove_target(&self, id: &str) -> bool {
        let mut config = self.config.write();
        let initial_len = config.targets.len();
        
        // Find the target address before removing
        let target_address = config.targets
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.address.clone());
        
        config.targets.retain(|t| t.id != id);
        
        // Remove from stats
        if let Some(address) = target_address {
            self.stats.write().remove_target(&address);
        }
        
        config.targets.len() < initial_len
    }

    /// Toggle a target's enabled state
    pub fn toggle_target(&self, id: &str) -> Option<bool> {
        let mut config = self.config.write();
        if let Some(target) = config.targets.iter_mut().find(|t| t.id == id) {
            target.enabled = !target.enabled;
            Some(target.enabled)
        } else {
            None
        }
    }

    /// Update a target
    pub fn update_target(&self, id: &str, address: String, label: String) -> Option<PingTarget> {
        let mut config = self.config.write();
        if let Some(target) = config.targets.iter_mut().find(|t| t.id == id) {
            target.address = address;
            target.label = label;
            Some(target.clone())
        } else {
            None
        }
    }

    /// Get ping state
    pub fn get_ping_state(&self) -> PingState {
        *self.ping_state.read()
    }

    /// Set ping state
    pub fn set_ping_state(&self, state: PingState) {
        *self.ping_state.write() = state;
    }

    /// Get log directory path
    pub fn get_log_path(&self) -> PathBuf {
        let logger = self.logger.read();
        if let Some(ref logger) = *logger {
            logger.log_dir().clone()
        } else {
            JsonLogger::default_log_dir()
        }
    }

    /// Reset all statistics
    pub fn reset_stats(&self) {
        self.stats.write().reset_all();
        self.recent_results.write().clear();
        self.reset_sequence();
    }

    /// Create a pinger with current timeout settings
    pub fn create_pinger(&self) -> Pinger {
        let config = self.config.read();
        Pinger::new(config.timeout_ms)
    }

    /// Get ping interval
    pub fn get_ping_interval(&self) -> u64 {
        self.config.read().ping_interval_ms
    }

    /// Set ping interval
    pub fn set_ping_interval(&self, interval_ms: u64) {
        self.config.write().ping_interval_ms = interval_ms;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
