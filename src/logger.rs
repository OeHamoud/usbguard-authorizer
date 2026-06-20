use crate::event::UsbEvent;
use crate::prompt::PromptResult;
use chrono::Local;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use tracing::{info, warn};

/// Decide log location:
/// - /var/log if running with sudo/root
/// - /tmp otherwise (safe fallback)
fn log_path() -> &'static str {
    if std::env::var("SUDO_UID").is_ok() || nix_uid_is_root() {
        "/var/log/usbguard-authorizer.log"
    } else {
        "/tmp/usbguard-authorizer.log"
    }
}

/// safer root check (prevents false sudo detection)
fn nix_uid_is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

#[derive(Serialize)]
struct LogEntry {
    timestamp: String,
    event_type: String,
    device_id: u32,
    name: String,
    vendor_id: String,
    product_id: String,
    port: String,
    serial: String,
    decision: String,
}

/// Log decision (ALLOW / DENY) for a USB event
pub fn log_decision(event: &UsbEvent, decision: &PromptResult) {
    let decision_str = match decision {
        PromptResult::Approved => "ALLOWED",
        PromptResult::Denied => "DENIED",
    };

    // Structured console log (journald / stdout)
    info!(
        device_id = event.device_id,
        name = %event.name,
        vendor_id = %event.vendor_id,
        product_id = %event.product_id,
        port = %event.port,
        decision = decision_str,
        "USB device decision"
    );

    let entry = LogEntry {
        timestamp: Local::now().to_rfc3339(),
        event_type: event.event_type.to_string(),
        device_id: event.device_id,
        name: event.name.clone(),
        vendor_id: event.vendor_id.clone(),
        product_id: event.product_id.clone(),
        port: event.port.clone(),
        serial: event.serial.clone(),
        decision: decision_str.to_string(),
    };

    // Serialize safely
    let json = match serde_json::to_string(&entry) {
        Ok(j) => j,
        Err(e) => {
            warn!("Failed to serialize USB log entry: {}", e);
            return;
        }
    };

    // Write log file
    let path = log_path();

    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{json}") {
                warn!("Failed writing to log file: {}", e);
            }
        }
        Err(e) => {
            warn!("Cannot open log file {}: {}", path, e);
        }
    }
}