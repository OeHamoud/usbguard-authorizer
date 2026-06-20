use crate::{event, logger, prompt};
use crate::prompt::PromptResult;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Entry point
pub async fn run() -> Result<()> {
    let mut child = Command::new("usbguard")
        .args(["watch"])
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("failed stdout");

    let mut lines = BufReader::new(stdout).lines();

    info!("usbguard watch started");

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        debug!("RAW: {}", line);

        match event::parse_event(&line) {
            Ok(ev) => handle_event(ev).await,
            Err(_) => {}
        }
    }

    Ok(())
}

/// MAIN EVENT ROUTER
async fn handle_event(ev: crate::event::UsbEvent) {
    use crate::event::EventType;

    info!("USB EVENT: {}", ev);

    match ev.event_type {
    EventType::DeviceInserted => {
        // 🚨 EXTRA SAFETY CHECK
        if ev.device_id == 4294967295 || ev.device_id == 0 {
            info!("Ignoring invalid insert event");
            return;
        }

        info!("REAL USB INSERT → prompting user");

        if let Some(full) = resolve_device(ev.device_id).await {
            ask_and_authorize(full).await;
        } else {
            warn!("Could not resolve device → skipping prompt");
        }
    }

    EventType::DeviceRemoved => {
        info!("USB removed → NO PROMPT (safe)");
    }

    EventType::DeviceBlocked | EventType::DeviceAllowed => {
        info!("Policy event → ignored");
    }

    EventType::Unknown(_) => {
        // 🔥 IMPORTANT: never prompt from unknown
        info!("Noise event ignored");
    }
}
}

/// 🔥 FIX: Resolve proper USB name from usbguard database
async fn resolve_device(id: u32) -> Option<crate::event::UsbEvent> {
    let output = Command::new("usbguard")
        .args(["list-devices"])
        .output()
        .await
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if !line.starts_with(&format!("{}:", id)) {
            continue;
        }

        let name = extract_field(line, "name");
        let serial = extract_field(line, "serial");
        let port = extract_field(line, "via-port");

        let ids = extract_field(line, "id");
        let (vendor_id, product_id) = if let Some((v, p)) = ids.split_once(':') {
            (v.to_string(), p.to_string())
        } else {
            ("".to_string(), "".to_string())
        };

        return Some(crate::event::UsbEvent {
            device_id: id,
            name,
            vendor_id,
            product_id,
            port,
            serial,
            raw_line: line.to_string(),
            event_type: crate::event::EventType::Unknown("resolved".into()),
        });
    }

    None
}

/// Extract key="value" from usbguard output
fn extract_field(line: &str, key: &str) -> String {
    let pattern = format!("{} \"", key);

    if let Some(start) = line.find(&pattern) {
        let start = start + pattern.len();

        if let Some(end) = line[start..].find('"') {
            return line[start..start + end].to_string();
        }
    }

    String::new()
}

/// Prompt user + apply decision
async fn ask_and_authorize(ev: crate::event::UsbEvent) {
    let device_id = ev.device_id;

    let decision = match prompt::ask_user(&ev).await {
        Ok(d) => d,
        Err(e) => {
            error!("prompt failed: {}", e);
            PromptResult::Denied
        }
    };

    logger::log_decision(&ev, &decision);

    if decision == PromptResult::Approved {
        info!("APPROVED {}", device_id);
        allow_device(device_id).await;
    } else {
        info!("DENIED {}", device_id);
    }
}

/// Allow USB device via usbguard
async fn allow_device(device_id: u32) {
    let id = device_id.to_string();

    let _ = Command::new("pkexec")
        .args(["usbguard", "allow-device", &id])
        .status()
        .await;
}