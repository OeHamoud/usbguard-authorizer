use anyhow::{anyhow, Result};
use regex::Regex;
use std::fmt;

/// USB event types
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    DeviceInserted,
    DeviceBlocked,
    DeviceAllowed,
    DeviceRemoved,
    Unknown(String),
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceInserted => write!(f, "DeviceInserted"),
            Self::DeviceBlocked => write!(f, "DeviceBlocked"),
            Self::DeviceAllowed => write!(f, "DeviceAllowed"),
            Self::DeviceRemoved => write!(f, "DeviceRemoved"),
            Self::Unknown(s) => write!(f, "Unknown({})", s),
        }
    }
}

/// Parsed USB event
#[derive(Debug, Clone)]
pub struct UsbEvent {
    pub event_type: EventType,
    pub device_id: u32,
    pub name: String,
    pub vendor_id: String,
    pub product_id: String,
    pub port: String,
    pub serial: String,
    pub raw_line: String,
}

impl fmt::Display for UsbEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] id={} name=\"{}\" vendor={} product={} port={}",
            self.event_type,
            self.device_id,
            self.name,
            self.vendor_id,
            self.product_id,
            self.port,
        )
    }
}

/// MAIN PARSER
pub fn parse_event(line: &str) -> Result<UsbEvent> {
    let event_type = detect_event_type(line);

    let device_id = extract_id(line)
        .ok_or_else(|| anyhow!("No device id found: {}", line))?;

    Ok(UsbEvent {
        event_type,
        device_id,
        name: extract_quoted(line, "name").unwrap_or_else(|| "Unknown USB Device".into()),
        vendor_id: extract_vendor_product(line, 0),
        product_id: extract_vendor_product(line, 1),
        port: extract_quoted(line, "via-port").unwrap_or_default(),
        serial: extract_quoted(line, "serial").unwrap_or_default(),
        raw_line: line.to_string(),
    })
}

/// EVENT TYPE DETECTION (FIXED)
fn detect_event_type(line: &str) -> EventType {
    let lower = line.to_lowercase();

    // ❌ IGNORE ALL POLICY NOISE FIRST
    if lower.contains("policychanged")
        || lower.contains("policyapplied")
        || lower.contains("rule_id")
    {
        return EventType::Unknown("policy_noise".into());
    }

    // ❌ REAL REMOVAL detection
    if lower.contains("remove")
        || lower.contains("unbind")
        || lower.contains("disappear")
    {
        return EventType::DeviceRemoved;
    }

    // ✅ REAL INSERT ONLY if it is not noise
    if lower.contains("presencechanged") && lower.contains("id=") {
        return EventType::DeviceInserted;
    }

    if lower.contains("target=block") {
        return EventType::DeviceBlocked;
    }

    if lower.contains("target=allow") {
        return EventType::DeviceAllowed;
    }

    EventType::Unknown(line.chars().take(40).collect())
}

/// Extract device id (supports id= and device.id=)
fn extract_id(line: &str) -> Option<u32> {
    let re = Regex::new(r"(?:device\.)?id=(\d+)").ok()?;
    re.captures(line)?
        .get(1)?
        .as_str()
        .parse()
        .ok()
}

/// Extract vendor/product from "id 0781:5567"
fn extract_vendor_product(line: &str, index: usize) -> String {
    let re = Regex::new(r"(\b[0-9a-fA-F]{4}:[0-9a-fA-F]{4}\b)").ok();

    if let Some(re) = re {
        if let Some(cap) = re.captures(line) {
            if let Some(pair) = cap.get(1) {
                let parts: Vec<&str> = pair.as_str().split(':').collect();
                if parts.len() == 2 {
                    return parts[index].to_string();
                }
            }
        }
    }

    String::new()
}

/// Extract quoted fields
fn extract_quoted(line: &str, key: &str) -> Option<String> {
    let pattern = format!(r#"{}=['"](.*?)['"]"#, regex::escape(key));
    let re = Regex::new(&pattern).ok()?;
    Some(re.captures(line)?.get(1)?.as_str().to_string())
}