use crate::models::{AppConfig, PingResult, PingState, PingStatistics, PingTarget};
use crate::state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::broadcast;

/// Start continuous ping monitoring
#[tauri::command]
pub async fn start_pinging(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let current_state = state.get_ping_state();
    if current_state == PingState::Running {
        return Err("Pinging is already running".to_string());
    }

    // Create stop signal channel
    let (tx, _) = broadcast::channel::<()>(1);
    *state.stop_signal.write() = Some(tx.clone());

    // Reset stats if starting fresh
    if current_state == PingState::Stopped {
        state.reset_stats();
    }

    state.set_ping_state(PingState::Running);

    // Clone what we need for the async task
    let state_clone = Arc::clone(&state);
    let app_clone = app.clone();

    // Spawn the ping loop
    tokio::spawn(async move {
        let mut rx = tx.subscribe();
        
        loop {
            // Check for stop signal
            if rx.try_recv().is_ok() {
                break;
            }

            // Check if still running
            if state_clone.get_ping_state() != PingState::Running {
                break;
            }

            // Get enabled targets
            let targets = state_clone.get_enabled_targets();
            if targets.is_empty() {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            // Create pinger
            let pinger = state_clone.create_pinger();
            let sequence = state_clone.next_sequence();

            // Ping all enabled targets
            for target in targets {
                // Execute ping synchronously (it's already fast)
                let result = pinger.ping(&target, sequence);
                
                // Add result to state
                state_clone.add_result(result.clone());
                
                // Emit event to frontend
                let _ = app_clone.emit("ping-result", &result);
            }

            // Emit stats update
            let stats = state_clone.get_all_stats();
            let _ = app_clone.emit("stats-update", &stats);

            // Wait for next interval
            let interval_ms = state_clone.get_ping_interval();
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
        }

        // Update state when loop ends
        if state_clone.get_ping_state() == PingState::Running {
            state_clone.set_ping_state(PingState::Stopped);
        }
    });

    Ok(())
}

/// Stop ping monitoring
#[tauri::command]
pub async fn stop_pinging(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.set_ping_state(PingState::Stopped);
    
    // Send stop signal
    if let Some(tx) = state.stop_signal.read().as_ref() {
        let _ = tx.send(());
    }
    
    Ok(())
}

/// Pause ping monitoring
#[tauri::command]
pub async fn pause_pinging(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    if state.get_ping_state() == PingState::Running {
        state.set_ping_state(PingState::Paused);
    }
    Ok(())
}

/// Resume ping monitoring
#[tauri::command]
pub async fn resume_pinging(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    if state.get_ping_state() == PingState::Paused {
        start_pinging(app, state).await?;
    }
    Ok(())
}

/// Get current ping state
#[tauri::command]
pub fn get_ping_state(state: State<'_, Arc<AppState>>) -> PingState {
    state.get_ping_state()
}

/// Get statistics for all targets
#[tauri::command]
pub fn get_statistics(state: State<'_, Arc<AppState>>) -> Vec<PingStatistics> {
    state.get_all_stats()
}

/// Get statistics for a specific target
#[tauri::command]
pub fn get_statistics_for_target(
    target: String,
    state: State<'_, Arc<AppState>>,
) -> Option<PingStatistics> {
    state.get_stats_for_target(&target)
}

/// Get recent ping results
#[tauri::command]
pub fn get_recent_pings(
    count: Option<usize>,
    state: State<'_, Arc<AppState>>,
) -> Vec<PingResult> {
    state.get_recent_results(count)
}

/// Get log directory path
#[tauri::command]
pub fn get_log_path(state: State<'_, Arc<AppState>>) -> String {
    state.get_log_path().to_string_lossy().to_string()
}

/// Set ping interval
#[tauri::command]
pub fn set_ping_interval(interval_ms: u64, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    if interval_ms < 100 {
        return Err("Interval must be at least 100ms".to_string());
    }
    state.set_ping_interval(interval_ms);
    Ok(())
}

/// Get all configured targets
#[tauri::command]
pub fn get_targets(state: State<'_, Arc<AppState>>) -> Vec<PingTarget> {
    state.get_targets()
}

/// Add a new ping target
#[tauri::command]
pub fn add_target(
    address: String,
    label: String,
    state: State<'_, Arc<AppState>>,
) -> Result<PingTarget, String> {
    if address.is_empty() {
        return Err("Address cannot be empty".to_string());
    }
    
    let target = PingTarget::new(address, label);
    Ok(state.add_target(target))
}

/// Remove a ping target
#[tauri::command]
pub fn remove_target(id: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    if state.remove_target(&id) {
        Ok(())
    } else {
        Err("Target not found".to_string())
    }
}

/// Update a ping target
#[tauri::command]
pub fn update_target(
    id: String,
    address: String,
    label: String,
    state: State<'_, Arc<AppState>>,
) -> Result<PingTarget, String> {
    state
        .update_target(&id, address, label)
        .ok_or_else(|| "Target not found".to_string())
}

/// Toggle a target's enabled state
#[tauri::command]
pub fn toggle_target(id: String, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    state
        .toggle_target(&id)
        .ok_or_else(|| "Target not found".to_string())
}

/// Get current configuration
#[tauri::command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> AppConfig {
    state.get_config()
}

/// Update configuration
#[tauri::command]
pub fn save_config(config: AppConfig, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.update_config(config);
    Ok(())
}

/// Get preset targets
#[tauri::command]
pub fn get_preset_targets() -> Vec<PingTarget> {
    PingTarget::presets()
}

/// Reset all statistics
#[tauri::command]
pub fn reset_statistics(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.reset_stats();
    Ok(())
}

/// Open log directory in file explorer
#[tauri::command]
pub async fn open_log_directory(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let path = state.get_log_path();
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
