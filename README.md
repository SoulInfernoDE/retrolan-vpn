<div align="center">
  <img src="assets/logo.svg" alt="RetroLAN Logo" width="220" />
  <h1>🌐 RetroLAN-VPN</h1>
  <p><b>Zero-Config Layer 2/3 LAN & Steam Relay VPN Engine for Classic & Modern Gaming</b></p>
  <p><i>[ <a href="README_DE.md">🇩🇪 Auf Deutsch lesen / Read in German</a> ]</i></p>
</div>

---

## 🚀 About The Project
RetroLAN-VPN bridges the gap between retro LAN gaming and modern network security. Built with **Rust, WireGuard (`boringtun`/`Wintun`), Tauri v2, Vite 8, and Steamworks**, it combines high-speed User-Space Layer 3 encryption with an automated Layer 2 UDP broadcast reflector.

### ✨ Key Features
* **Zero-Config NAT & IPv6 Traversal:** Automatically routes via IPv6 P2P, IPv4 STUN hole-punching, or seamlessly falls back to Valve's global **Steam Relay Network (SDR)** to bypass DS-Lite and CGNAT.
* **IPX-to-UDP Wrapping:** Intercepts retro IPX/SPX packets via a lightweight `wsock32.dll` hook and tunnels them over modern IPv4.
* **Linux-First & AVX2/NTSYNC Proton Manager:** Auto-injects `WINE_BIND_IP` bindings, scans CPU hardware capabilities (`x86-64-v3`), and dynamically downloads compatibility tools like **Proton-CachyOS v3** or **GE-Proton**.
* **Clean OS Footprint:** Strict Subnet-Only Routing (Split-Tunneling) keeps your regular browser and Discord traffic untouched. Auto-cleans virtual adapters on exit.
* **Offline LAN Mode:** Physical local discovery via mDNS (`mdns-sd v0.20+`) for true basement LAN parties without internet or Steam.
* **Modern Cyberpunk GUI:** Built with Tauri v2, Vite 8, TypeScript, and Tailwind CSS for an ultra-fast, lightweight dashboard experience compatible with NSolid and Node.js LTS.

### 🎥 Quickstart & UI Preview
*The animated interactive quickstart guide showing lobby creation, protocol switching, and Proton auto-detection will be displayed below upon GUI release:*

<div align="center">
  <!-- PLACEHOLDER FOR FINAL ANIMATED DEMO (demo.webp / demo.gif) -->
  <img src="https://raw.githubusercontent.com/SoulInfernoDE/retrolan-vpn/main/assets/logo.svg" alt="RetroLAN App Demo Preview" width="600" style="opacity: 0.7; border-radius: 10px;" />
  <p><i>▲ Interactive RetroLAN UI Demo (Lobby Host, IPX Hooking & Steam Relay in Action) ▲</i></p>
</div>

---

## 📦 Installation & Usage

### 1. System Requirements & Dependencies
* **Linux (Recommended):** Modern distribution (Arch/CachyOS, Ubuntu 22.04+, Fedora). Uses native user-space `TUN/TAP` (requires standard `polkit` permissions for adapter creation).
* **Windows:** Windows 10/11 (64-Bit). Uses the bundled official **Wintun** virtual network adapter (will trigger a one-time UAC administrator prompt on first launch).
* **Steam Client (Optional but Recommended):** Required for automatic Steam Relay Network (SDR) fallback routing and P2P lobby invites.

### 2. Pre-Compiled Installation
* **Linux:** Download the latest `.AppImage`, `.deb`, or `.rpm` package from the [Releases Page](https://github.com/SoulInfernoDE/retrolan-vpn/releases).
* **Windows:** Download and run `RetroLAN-Setup.exe`.
* **Portable Mode:** Simply extract the archive and run the executable directly; all configs are read from the local directory.

### 3. How to Use (Step-by-Step)
1. **Launch RetroLAN-VPN:** The app automatically scans your CPU capabilities (AVX2/FMA), checks for `/dev/ntsync`, and verifies your network status.
2. **Host or Join a Lobby:**
   * **Online (Steam):** Click **"Steam SDR Lobby Hosten"** to invite Steam friends. If direct P2P fails due to strict CGNAT/DS-Lite, the app instantly switches to Valve's global SDR relay servers.
   * **Offline (Physical LAN):** Select **"Offline-LAN Beitreten"** to discover friends on the same physical switch via local mDNS beacons.
3. **Launch Your Game:** Select your game from the community database (`games.toml`). RetroLAN automatically applies the required profile (e.g., binding interfaces, applying Proton-CachyOS v3, or injecting the IPX wrapper).
4. **Play:** Enjoy classic LAN gaming with zero latency and zero manual router configuration!

### 4. Building from Source (Compilation Guide)
If you wish to compile RetroLAN-VPN from scratch, follow these instructions using the native Rust Tauri CLI (fully compatible with Node.js and NSolid environments):

#### A. Install Prerequisites
* **Rust & Cargo:** Install via [rustup.rs](https://rustup.rs/) (Rust 1.78+ required).
* **Native Tauri v2 CLI:** Install globally via Cargo to prevent CLI execution conflicts in hardened Node/NSolid runtimes:
  ```bash
  cargo install tauri-cli --version "^2.0.0" --locked
  ```
* **Node.js / NSolid:** Install Node.js v20+ or NSolid LTS.
* **System Build Dependencies:**
  * **Debian / Ubuntu:** `sudo apt update && sudo apt install -y build-essential curl wget file libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  * **Arch Linux / CachyOS:** `sudo pacman -S --needed base-devel curl wget openssl gtk3 webkit2gtk appindicator-gtk3 librsvg patchelf`
  * **Windows:** Install the *Microsoft Visual C++ Build Tools* and *WebView2 SDK*.

#### B. Compile & Run
```bash
# 1. Clone the repository
git clone https://github.com/SoulInfernoDE/retrolan-vpn.git
cd retrolan-vpn

# 2. Install Tauri frontend web dependencies (Vite 8 & Tailwind)
npm install

# 3. Build and launch in development mode using native Rust CLI
cargo tauri dev

# 4. Create an optimized release binary and installer bundle
cargo tauri build
```
The compiled production executables and installers will be generated inside `target/release/bundle/`.

---

## ⚖️ License & Acknowledgments
Distributed under the MIT License. See `LICENSE` for more information.
For attributions regarding WireGuard, BoringTun, Wintun, Steamworks, and Wine/Proton, please see [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).