# 🔐 USBGuard Authorizer

![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)
![Linux](https://img.shields.io/badge/Linux-supported-blue?logo=linux)
![License](https://img.shields.io/badge/license-MIT-green)
![Status](https://img.shields.io/badge/status-active-success)
![Security](https://img.shields.io/badge/USB%20Control-secure-red)

A **real-time USB authorization daemon for Linux** built with Rust + USBGuard.

It intercepts USB device connections and requires **user approval before access is granted**.

---

## ⚡ Why this exists

Linux USB security is usually:
- ❌ fully automatic
- ❌ hard to control per-device
- ❌ invisible to users

This project fixes that by introducing:

> 🧠 **interactive USB control at insertion time**

---

## ✨ Features

- 🔌 Real-time USB monitoring via `usbguard watch`
- 🪟 GUI prompt support:
  - KDE (`kdialog`)
  - GNOME (`zenity`)
  - terminal fallback
- 🔐 Allow / deny before device access
- 🧾 JSON logging of decisions
- 🚫 Filters noise (policy + removal events)
- ⚡ Async Rust backend (Tokio)
- 🧠 Device info extraction (name, VID, PID, serial)

---

## 📸 Example

```

A USB device has been detected:

Device ID : 42
Name      : Cruzer Blade
Vendor ID : 0781
Product ID: 5567
Serial    : 20060266931D78C11A76

Do you want to ALLOW this device?

````

---

## 🚀 Installation

```bash id="install1"
git clone https://github.com/YOUR_USERNAME/usbguard-authorizer
cd usbguard-authorizer
cargo build --release
````

---

## ▶️ Run

```bash id="run1"
cargo run
```

or:

```bash id="run2"
./target/release/usbguard-authorizer
```

---

## ⚙️ systemd service

```ini id="systemd1"
[Unit]
Description=USBGuard Authorizer
After=usbguard.service

[Service]
ExecStart=/usr/local/bin/usbguard-authorizer
Restart=always
User=root
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

---

## 📊 Logging

Stored at:

* `/var/log/usbguard-authorizer.log`
* fallback: `/tmp/usbguard-authorizer.log`

Includes:

* device metadata
* decision (ALLOW / DENY)
* timestamps

---

## 🧠 How it works

1. Listens to:

   ```bash
   usbguard watch
   ```
2. Parses USB events
3. Detects insertion events
4. Extracts device metadata
5. Prompts user
6. Applies decision:

   ```bash
   pkexec usbguard allow-device <id>
   ```

---

## 🛡 Security model

* Default: deny unknown devices
* User must explicitly approve access
* Policy events ignored automatically

---

## 🧩 Roadmap

* [ ] Trusted device whitelist
* [ ] Persistent device memory
* [ ] Device fingerprinting (hash-based)
* [ ] Tray UI (system integration)
* [ ] Wayland-native prompt system

---

## ⭐ Star History

If you like this project, give it a ⭐ — it helps a lot.

---

## 📜 License

MIT
