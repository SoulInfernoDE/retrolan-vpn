<div align="center">
  <img src="assets/logo.svg" alt="RetroLAN Logo" width="220" />
  <h1>🌐 RetroLAN-VPN</h1>
  <p><b>Zero-Config Layer 2/3 LAN & Steam Relay VPN Engine for Classic & Modern Gaming</b></p>
  <p><i>[ <a href="README_DE.md">🇩🇪 Auf Deutsch lesen / Read in German</a> ]</i></p>
</div>

---

## 🚀 About The Project
RetroLAN-VPN bridges the gap between retro LAN gaming and modern network security. Built with **Rust, WireGuard (`boringtun`/`Wintun`), and Steamworks**, it combines high-speed User-Space Layer 3 encryption with an automated Layer 2 UDP broadcast reflector.

### ✨ Key Features
* **Zero-Config NAT & IPv6 Traversal:** Automatically routes via IPv6 P2P, IPv4 STUN hole-punching, or seamlessly falls back to Valve's global **Steam Relay Network (SDR)** to bypass DS-Lite and CGNAT.
* **IPX-to-UDP Wrapping:** Intercepts retro IPX/SPX packets via a lightweight `wsock32.dll` hook and tunnels them over modern IPv4.
* **Linux-First & Proton Manager:** Auto-injects `WINE_BIND_IP` bindings and automatically downloads compatibility tools like **Proton-GE** or **Proton-CachyOS**.
* **Clean OS Footprint:** Strict Subnet-Only Routing (Split-Tunneling) keeps your regular browser and Discord traffic untouched. Auto-cleans virtual adapters on exit.
* **Offline LAN Mode:** Physical local discovery via mDNS for true basement LAN parties without internet or Steam.
* **Drop-in Modding & i18n:** Add games via `games.toml` or translations via `locales/*.toml` without compiling a single line of code.

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

### 2. Installation
* **Linux:** Download the latest `.AppImage` or `.deb` / `.rpm` package from the [Releases Page](https://github.com/SoulInfernoDE/retrolan-vpn/releases).
* **Windows:** Download and run `RetroLAN-Setup.exe`.
* **Portable Mode:** Simply extract the archive and run the executable directly; all configs are read from the local directory.

### 3. How to Use (Step-by-Step)
1. **Launch RetroLAN-VPN:** The app automatically scans for existing Proton versions and checks your network status.
2. **Host or Join a Lobby:**
   * **Online (Steam):** Click **"Host Lobby"** to invite Steam friends. If direct P2P fails due to strict CGNAT/DS-Lite, the app instantly switches to Valve's global SDR relay servers.
   * **Offline (Physical LAN):** Select **"Offline Mode"** to discover friends on the same physical switch via local mDNS beacons.
3. **Launch Your Game:** Start your classic game via Steam or directly from your hard drive. RetroLAN automatically applies the required profile from `games.toml` (e.g., binding interfaces or injecting the IPX wrapper).
4. **Play:** Enjoy classic LAN gaming with zero latency and zero manual router configuration!

---

## ⚖️ License & Acknowledgments
Distributed under the MIT License. See `LICENSE` for more information.
For attributions regarding WireGuard, BoringTun, Wintun, Steamworks, and Wine/Proton, please see [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
