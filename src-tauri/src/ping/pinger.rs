use crate::models::{PingResult, PingTarget};
use std::process::Command;

/// Cross-platform pinger that uses system ping command
/// This approach works without root/admin privileges on all platforms
pub struct Pinger {
    timeout_ms: u64,
}

impl Pinger {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    /// Ping a target and return the result
    pub fn ping(&self, target: &PingTarget, sequence: u32) -> PingResult {
        let result = self.execute_ping(&target.address);
        
        match result {
            Ok(latency) => PingResult::success(target, latency, sequence),
            Err(error) => PingResult::failure(target, error, sequence),
        }
    }

    /// Execute platform-specific ping command
    fn execute_ping(&self, address: &str) -> Result<f64, String> {
        let timeout_secs = (self.timeout_ms / 1000).max(1);
        
        #[cfg(target_os = "windows")]
        let output = Command::new("ping")
            .args(["-n", "1", "-w", &(self.timeout_ms).to_string(), address])
            .output();

        #[cfg(target_os = "macos")]
        let output = Command::new("ping")
            .args(["-c", "1", "-t", &timeout_secs.to_string(), address])
            .output();

        #[cfg(target_os = "linux")]
        let output = Command::new("ping")
            .args(["-c", "1", "-W", &timeout_secs.to_string(), address])
            .output();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let output = Command::new("ping")
            .args(["-c", "1", address])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    self.parse_latency(&stdout)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Ping failed: {}", stderr.trim()))
                }
            }
            Err(e) => Err(format!("Failed to execute ping: {}", e)),
        }
    }

    /// Parse latency from ping output
    fn parse_latency(&self, output: &str) -> Result<f64, String> {
        // Windows format: "Reply from x.x.x.x: bytes=32 time=12ms TTL=57"
        // macOS/Linux format: "64 bytes from x.x.x.x: icmp_seq=1 ttl=57 time=12.3 ms"
        
        // Try to find "time=" or "time<" pattern
        if let Some(time_idx) = output.find("time=") {
            let after_time = &output[time_idx + 5..];
            return self.extract_number(after_time);
        }
        
        if let Some(time_idx) = output.find("time<") {
            let after_time = &output[time_idx + 5..];
            return self.extract_number(after_time);
        }

        // Try to find "time " pattern (some systems use space)
        if let Some(time_idx) = output.find("time ") {
            let after_time = &output[time_idx + 5..];
            return self.extract_number(after_time);
        }

        Err("Could not parse latency from ping output".to_string())
    }

    /// Extract a floating point number from the beginning of a string
    fn extract_number(&self, s: &str) -> Result<f64, String> {
        let num_str: String = s
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        
        num_str
            .parse::<f64>()
            .map_err(|_| format!("Could not parse number from: {}", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_latency_macos() {
        let pinger = Pinger::new(5000);
        let output = "64 bytes from 1.1.1.1: icmp_seq=0 ttl=57 time=12.345 ms";
        assert_eq!(pinger.parse_latency(output).unwrap(), 12.345);
    }

    #[test]
    fn test_parse_latency_windows() {
        let pinger = Pinger::new(5000);
        let output = "Reply from 1.1.1.1: bytes=32 time=15ms TTL=57";
        assert_eq!(pinger.parse_latency(output).unwrap(), 15.0);
    }

    #[test]
    fn test_parse_latency_linux() {
        let pinger = Pinger::new(5000);
        let output = "64 bytes from 1.1.1.1: icmp_seq=1 ttl=57 time=8.92 ms";
        assert_eq!(pinger.parse_latency(output).unwrap(), 8.92);
    }
}
