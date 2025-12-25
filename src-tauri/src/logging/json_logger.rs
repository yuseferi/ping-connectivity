use crate::models::PingResult;
use chrono::{Local, NaiveDate};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;

/// JSON logger with daily file rotation
pub struct JsonLogger {
    log_dir: PathBuf,
    current_date: Mutex<Option<NaiveDate>>,
    writer: Mutex<Option<BufWriter<File>>>,
}

impl JsonLogger {
    /// Create a new JSON logger
    pub fn new(log_dir: PathBuf) -> Result<Self, std::io::Error> {
        // Create log directory if it doesn't exist
        fs::create_dir_all(&log_dir)?;
        
        Ok(Self {
            log_dir,
            current_date: Mutex::new(None),
            writer: Mutex::new(None),
        })
    }

    /// Get the default log directory for the application
    pub fn default_log_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ping-connectivity")
            .join("logs")
    }

    /// Get the log directory path
    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }

    /// Log a ping result
    pub fn log(&self, result: &PingResult) -> Result<(), std::io::Error> {
        let today = Local::now().date_naive();
        
        // Check if we need to rotate the log file
        {
            let mut current_date = self.current_date.lock().unwrap();
            let mut writer = self.writer.lock().unwrap();
            
            if current_date.map(|d| d != today).unwrap_or(true) {
                // Close existing writer
                if let Some(ref mut w) = *writer {
                    w.flush()?;
                }
                
                // Open new file for today
                let file_path = self.log_file_path(today);
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)?;
                
                *writer = Some(BufWriter::new(file));
                *current_date = Some(today);
                
                log::info!("Opened log file: {:?}", file_path);
            }
        }
        
        // Write the log entry
        let mut writer = self.writer.lock().unwrap();
        if let Some(ref mut w) = *writer {
            let json = serde_json::to_string(result)?;
            writeln!(w, "{}", json)?;
            w.flush()?;
        }
        
        Ok(())
    }

    /// Get the log file path for a specific date
    fn log_file_path(&self, date: NaiveDate) -> PathBuf {
        self.log_dir.join(format!("ping-{}.jsonl", date.format("%Y-%m-%d")))
    }

    /// Get list of available log files
    pub fn list_log_files(&self) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut files = Vec::new();
        
        if self.log_dir.exists() {
            for entry in fs::read_dir(&self.log_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                    files.push(path);
                }
            }
        }
        
        files.sort();
        Ok(files)
    }

    /// Read ping results from a log file
    pub fn read_log_file(&self, path: &PathBuf) -> Result<Vec<PingResult>, std::io::Error> {
        let content = fs::read_to_string(path)?;
        let mut results = Vec::new();
        
        for line in content.lines() {
            if let Ok(result) = serde_json::from_str::<PingResult>(line) {
                results.push(result);
            }
        }
        
        Ok(results)
    }
}

impl Drop for JsonLogger {
    fn drop(&mut self) {
        // Flush and close the writer
        if let Ok(mut writer) = self.writer.lock() {
            if let Some(ref mut w) = *writer {
                let _ = w.flush();
            }
        }
    }
}
