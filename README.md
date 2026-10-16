# 🚀 Uninstall Apps (Fast Search & Clean Tool)

A blazingly fast, multi-threaded command-line tool built in **Rust** to scan, search, copy, or purge specific file footprints on macOS. It utilizes native parallel processors to handle heavy file structures in seconds.

## ✨ Features
* **Parallel Core Scan:** Floods your storage engine pipeline using all available CPU threads.
* **Live Progress Spinner:** Tracks indexed files/folders in real time without freezing.
* **Interactive Multi-Select Checkboxes:** Use `Spacebar` to highlight exactly what you want to touch.
* **Smart Actions:** Bulk-copy targeted paths to your Mac Clipboard or execute safe, native deletions.
* **Error-Safe:** Gracefully skips restricted macOS directories without crashing or requiring `sudo` configurations.

---

## 💻 Compatibility (Intel and Apple Silicon Macs M1, M2...)

This tool compiles cleanly down to native machine code on **all macOS architectures**:

* **Intel Macs (x86_64):** Runs natively. If a binary is built on an Intel Mac, it runs out-of-the-box on other Intel Macs.
* **Apple Silicon Macs (M1/M2...):** Runs natively. If you download an Intel binary on an Apple Silicon Mac, it will run automatically using Apple's built-in **Rosetta 2** translation layer. For absolute maximum execution speed on an ARM processor, compile directly on the host Apple Silicon device.

---

## 🛠️ Installation Steps

### OPTION A: Download the executable and run it
1. Go to [Releases](/grbltl/uninstall-apps/releases)

2. Download uninstall_apps

3. Run it

### OPTION B: Build from Source
This approach guarantees that the binary is explicitly optimized for your exact processor family (Intel or M-Series Apple Silicon).

1. **Install the Rust toolchain:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh -s -- -y
   source "$HOME/.cargo/env"
   ```
2. **Clone and Enter Repository:**
   ```bash
   git clone https://github.com/grbltl/uninstall-apps.git
   cd uninstall-apps
   ```
3. **Compile the High-Speed Standalone Application:**
   ```bash
   cargo build --release
   ```

### Make it a Global System Command
To launch the tool from **any folder** on your Mac by typing a single keyword:
```bash
sudo cp target/release/uninstall-apps /usr/local/bin/uninstall-apps
```

---

## 🎮 How to Use

1. Launch your application:
   ```bash
   uninstall-apps
   ```
   *(Or `./target/release/uninstall-apps` if running straight out of your local project folder)*
2. Provide the folder directory path and your target keyword lookup.
3. Use the **Up/Down Arrow Keys** to navigate, tap **Spacebar** to check items, and hit **Enter** to pass choices to the action menu!
