use crate::event::UsbEvent;
use anyhow::Result;
use tokio::process::Command;
use tracing::{debug, warn};

#[derive(Debug, PartialEq)]
pub enum PromptResult {
    Approved,
    Denied,
}

/// MAIN ENTRY
pub async fn ask_user(event: &UsbEvent) -> Result<PromptResult> {
    let title = "USB Device Authorization";

    debug!("Prompt request for device_id={}", event.device_id);

    if event.device_id == 0 {
        warn!("Skipping invalid device_id=0");
        return Ok(PromptResult::Denied);
    }

    let message = build_message(event);

    // KDE FIRST
    match run_kdialog(&message, title).await {
        Ok(r) => return Ok(r),
        Err(e) => warn!("kdialog failed: {}", e),
    }

    // GNOME fallback
    match run_zenity(&message, title).await {
        Ok(r) => return Ok(r),
        Err(e) => warn!("zenity failed: {}", e),
    }

    // terminal fallback
    run_terminal_prompt(event).await
}

/// CLEAN MESSAGE BUILDER
fn build_message(event: &UsbEvent) -> String {
    format!(
        "A USB device has been detected:\n\n\
         Device ID : {}\n\
         Name      : {}\n\
         Vendor ID : {}\n\
         Product ID: {}\n\
         Serial    : {}\n\n\
         Do you want to ALLOW this device?",
        event.device_id,
        if event.name.is_empty() { "Unknown" } else { &event.name },
        event.vendor_id,
        event.product_id,
        if event.serial.is_empty() { "unknown" } else { &event.serial },
    )
}

/// KDE DIALOG (FIXED EXIT HANDLING)
async fn run_kdialog(message: &str, title: &str) -> Result<PromptResult> {
    debug!("Launching kdialog");

    let status = Command::new("kdialog")
        .args([
            "--title", title,
            "--yesno", message,
            "--yes-label", "Allow",
            "--no-label", "Deny",
        ])
        .status()
        .await?;

    let code = status.code().unwrap_or(-1);
    debug!("kdialog finished");

    Ok(match code {
        0 => PromptResult::Approved,
        1 => PromptResult::Denied,
        _ => PromptResult::Denied,
    })
}

/// ZENITY FALLBACK
async fn run_zenity(message: &str, title: &str) -> Result<PromptResult> {
    let status = Command::new("zenity")
        .args([
            "--question",
            "--title", title,
            "--text", message,
            "--ok-label", "Allow",
            "--cancel-label", "Deny",
            "--width=400",
        ])
        .status()
        .await?;

    Ok(if status.success() {
        PromptResult::Approved
    } else {
        PromptResult::Denied
    })
}

/// TERMINAL FALLBACK
async fn run_terminal_prompt(event: &UsbEvent) -> Result<PromptResult> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let mut stdout = tokio::io::stdout();

    stdout.write_all(
        format!(
            "\n[USB Authorization Required]\n\
             Device ID : {}\n\
             Name      : {}\n\
             Vendor ID : {}\n\
             Product ID: {}\n\
             Serial    : {}\n\n\
             Allow? [y/N]: ",
            event.device_id,
            if event.name.is_empty() { "Unknown" } else { &event.name },
            event.vendor_id,
            event.product_id,
            event.serial,
        )
        .as_bytes(),
    ).await?;

    stdout.flush().await?;

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    Ok(if line.trim().eq_ignore_ascii_case("y") {
        PromptResult::Approved
    } else {
        PromptResult::Denied
    })
}