use crate::models::{PingResult, PingStatistics, PingTarget};
use chrono::Utc;
use std::collections::HashMap;

/// Statistics calculator for ping results
pub struct StatsCalculator {
    /// Statistics per target (keyed by target address)
    stats: HashMap<String, TargetStats>,
}

/// Internal statistics tracking for a single target
struct TargetStats {
    target: String,
    target_label: String,
    total_pings: u64,
    successful_pings: u64,
    failed_pings: u64,
    latencies: Vec<f64>,
    session_start: Option<chrono::DateTime<Utc>>,
    last_ping: Option<chrono::DateTime<Utc>>,
}

impl TargetStats {
    fn new(target: &PingTarget) -> Self {
        Self {
            target: target.address.clone(),
            target_label: target.label.clone(),
            total_pings: 0,
            successful_pings: 0,
            failed_pings: 0,
            latencies: Vec::new(),
            session_start: None,
            last_ping: None,
        }
    }

    fn update(&mut self, result: &PingResult) {
        self.total_pings += 1;
        self.last_ping = Some(result.timestamp);
        
        if self.session_start.is_none() {
            self.session_start = Some(result.timestamp);
        }
        
        if result.success {
            self.successful_pings += 1;
            if let Some(latency) = result.latency_ms {
                self.latencies.push(latency);
            }
        } else {
            self.failed_pings += 1;
        }
    }

    fn to_statistics(&self) -> PingStatistics {
        let packet_loss_percent = if self.total_pings > 0 {
            (self.failed_pings as f64 / self.total_pings as f64) * 100.0
        } else {
            0.0
        };

        let (min_latency_ms, max_latency_ms, avg_latency_ms, jitter_ms) = 
            if !self.latencies.is_empty() {
                let min = self.latencies.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = self.latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let sum: f64 = self.latencies.iter().sum();
                let avg = sum / self.latencies.len() as f64;
                
                // Calculate jitter (average deviation from mean)
                let jitter = if self.latencies.len() > 1 {
                    let variance: f64 = self.latencies
                        .iter()
                        .map(|&x| (x - avg).powi(2))
                        .sum::<f64>() / (self.latencies.len() - 1) as f64;
                    variance.sqrt()
                } else {
                    0.0
                };
                
                (Some(min), Some(max), Some(avg), Some(jitter))
            } else {
                (None, None, None, None)
            };

        PingStatistics {
            target: self.target.clone(),
            target_label: self.target_label.clone(),
            total_pings: self.total_pings,
            successful_pings: self.successful_pings,
            failed_pings: self.failed_pings,
            packet_loss_percent,
            min_latency_ms,
            max_latency_ms,
            avg_latency_ms,
            jitter_ms,
            session_start: self.session_start,
            last_ping: self.last_ping,
        }
    }

    fn reset(&mut self) {
        self.total_pings = 0;
        self.successful_pings = 0;
        self.failed_pings = 0;
        self.latencies.clear();
        self.session_start = None;
        self.last_ping = None;
    }
}

impl StatsCalculator {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    /// Initialize statistics for a target
    pub fn init_target(&mut self, target: &PingTarget) {
        if !self.stats.contains_key(&target.address) {
            self.stats.insert(target.address.clone(), TargetStats::new(target));
        }
    }

    /// Update statistics with a new ping result
    pub fn update(&mut self, result: &PingResult) {
        if let Some(stats) = self.stats.get_mut(&result.target) {
            stats.update(result);
        }
    }

    /// Get statistics for a specific target
    pub fn get_stats(&self, target_address: &str) -> Option<PingStatistics> {
        self.stats.get(target_address).map(|s| s.to_statistics())
    }

    /// Get statistics for all targets
    pub fn get_all_stats(&self) -> Vec<PingStatistics> {
        self.stats.values().map(|s| s.to_statistics()).collect()
    }

    /// Reset statistics for a specific target
    pub fn reset_target(&mut self, target_address: &str) {
        if let Some(stats) = self.stats.get_mut(target_address) {
            stats.reset();
        }
    }

    /// Reset all statistics
    pub fn reset_all(&mut self) {
        for stats in self.stats.values_mut() {
            stats.reset();
        }
    }

    /// Remove a target from statistics
    pub fn remove_target(&mut self, target_address: &str) {
        self.stats.remove(target_address);
    }
}

impl Default for StatsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_calculation() {
        let mut calc = StatsCalculator::new();
        let target = PingTarget::new("1.1.1.1".to_string(), "Test".to_string());
        
        calc.init_target(&target);
        
        // Add some successful pings
        calc.update(&PingResult::success(&target, 10.0, 1));
        calc.update(&PingResult::success(&target, 20.0, 2));
        calc.update(&PingResult::success(&target, 15.0, 3));
        
        // Add a failed ping
        calc.update(&PingResult::failure(&target, "Timeout".to_string(), 4));
        
        let stats = calc.get_stats("1.1.1.1").unwrap();
        
        assert_eq!(stats.total_pings, 4);
        assert_eq!(stats.successful_pings, 3);
        assert_eq!(stats.failed_pings, 1);
        assert_eq!(stats.packet_loss_percent, 25.0);
        assert_eq!(stats.min_latency_ms, Some(10.0));
        assert_eq!(stats.max_latency_ms, Some(20.0));
        assert_eq!(stats.avg_latency_ms, Some(15.0));
    }
}
