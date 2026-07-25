<div align="center">
  <img src="assets/logo.svg" alt="RetroLAN Logo" width="220" />
  <h1>🌐 RetroLAN-VPN</h1>
  <p><b>Zero-Config Layer 2/3 LAN & Steam-Relay VPN-Engine für Retro- & Modern-Gaming</b></p>
  <p><i>[ <a href="README.md">🇬🇧 Read in English / Auf Englisch lesen</a> ]</i></p>
</div>

---

## 🚀 Über das Projekt
RetroLAN-VPN schließt die Lücke zwischen klassischem LAN-Gaming und moderner Netzwerksicherheit. Entwickelt mit **Rust, WireGuard (`boringtun`/`Wintun`), Tauri v2, Vite 8 und Steamworks**, verbindet die App ultraschnelle Layer-3-Verschlüsselung im User-Space mit einem automatischen Layer-2-UDP-Broadcast-Reflektor.

### ✨ Hauptfunktionen
* **Zero-Config NAT & IPv6-Traversal:** Wählt automatisch zwischen direktem IPv6-P2P, IPv4-Hole-Punching oder weicht nahtlos auf das **Steam Relay Network (SDR)** von Valve aus, um DS-Lite und CGNAT zu überwinden.
* **IPX-to-UDP-Wrapping:** Fängt alte IPX/SPX-Pakete über einen `wsock32.dll`-Hook ab und tunnelt sie unsichtbar über modernes IPv4.
* **Linux-First & AVX2/NTSYNC Proton-Manager:** Setzt automatisches `WINE_BIND_IP`-Binding, scant Hardware-CPU-Befehlssätze (`x86-64-v3`) und lädt dynamisch optimale Tools wie **Proton-CachyOS v3** oder **GE-Proton** herunter.
* **Kein System-Eingriff:** Strikter Split-Tunneling-Ansatz (nur Spieldaten nutzen das VPN, dein normales Internet bleibt unberührt). Entfernt virtuelle Adapter beim Beenden spurlos.
* **Offline-LAN-Modus:** Physische lokale Erkennung über mDNS (`mdns-sd v0.20+`) für echte Keller-LAN-Partys ganz ohne Internet oder Steam.
* **Moderne Cyberpunk-GUI:** Entwickelt mit Tauri v2, Vite 8, TypeScript und Tailwind CSS für ein blitzschnelles, ressourcenschonendes Dashboard-Erlebnis – vollständig kompatibel mit NSolid und Node.js LTS.

### 🎥 Schnellanleitung & UI-Vorschau
*Die animierte, interaktive Schnellanleitung (Lobby-Erstellung, Protokollwechsel und automatische Proton-Erkennung) wird hier beim Release der Benutzeroberfläche präsentiert:*

<div align="center">
  <!-- PLATZHALTER FÜR DIE FINALE ANIMIERTE DEMO (demo.webp / demo.gif) -->
  <img src="https://raw.githubusercontent.com/SoulInfernoDE/retrolan-vpn/main/assets/logo.svg" alt="RetroLAN App Demo Vorschau" width="600" style="opacity: 0.7; border-radius: 10px;" />
  <p><i>▲ Interaktive RetroLAN UI-Demo (Lobby-Host, IPX-Hooking & Steam-Relay im Einsatz) ▲</i></p>
</div>

---

## 📦 Installation & Bedienung

### 1. Systemanforderungen & Abhängigkeiten
* **Linux (Empfohlen):** Moderne Distribution (Arch/CachyOS, Ubuntu 22.04+, Fedora). Nutzt natives User-Space `TUN/TAP` (benötigt standardmäßige `polkit`-Berechtigungen zur Adapter-Erstellung).
* **Windows:** Windows 10/11 (64-Bit). Nutzt den gebündelten offiziellen **Wintun**-Treiber (löst beim allerersten Start eine einmalige UAC-Administratorabfrage aus).
* **Steam-Client (Optional aber empfohlen):** Wird für das automatische Routing über das Steam Relay Network (SDR) sowie für P2P-Lobby-Einladungen benötigt.

### 2. Vorkompilierte Installation
* **Linux:** Lade das neueste `.AppImage` oder das passende `.deb` / `.rpm` Paket von der [Releases-Seite](https://github.com/SoulInfernoDE/retrolan-vpn/releases) herunter.
* **Windows:** Lade die Datei `RetroLAN-Setup.exe` herunter und führe sie aus.
* **Portabler Modus:** Entpacke das Archiv einfach in einen beliebigen Ordner und starte die Anwendung direkt; alle Konfigurationsdateien werden lokal im Verzeichnis verlegt.

### 3. Schritt-für-Schritt-Bedienung
1. **RetroLAN-VPN starten:** Die App scant beim Start automatisch deine Hardware-Architektur (AVX2/FMA), prüft auf `/dev/ntsync` und analysiert deinen Netzwerkstatus.
2. **Lobby erstellen oder beitreten:**
   * **Online (Steam):** Klicke auf **„Steam SDR Lobby Hosten“**, um Steam-Freunde einzuladen. Sollte eine direkte P2P-Verbindung an einem strengen DS-Lite-/CGNAT-Router scheitern, schaltet die App sofort unsichtbar auf Valves weltweite SDR-Relay-Server um.
   * **Offline (Physisches LAN):** Wähle **„Offline-LAN Beitreten“**, um Freunde am selben Switch oder WLAN per lokaler mDNS-Suche zu finden.
3. **Spiel starten:** Wähle dein Spiel aus der Community-Datenbank (`games.toml`). RetroLAN konfiguriert im Hintergrund automatisch das passende Profil (z. B. Interface-Binding, Proton-CachyOS v3 Aktivierung oder IPX-Wrapper-Injektion).
4. **Losspielen:** Genieße klassische LAN-Spiele mit minimalem Ping und komplett ohne lästige Router- oder Port-Freigaben!

### 4. Kompilieren aus dem Quellcode (Build-Anleitung)
Möchtest du RetroLAN-VPN selbst aus dem Quellcode kompilieren, folge diesen Schritten unter Verwendung der nativen Rust Tauri CLI (zu 100 % kompatibel mit Node.js und NSolid):

#### A. Voraussetzungen installieren
* **Rust & Cargo:** Installiere Rust über [rustup.rs](https://rustup.rs/) (Version 1.78+ erforderlich).
* **Native Tauri v2 CLI:** Installiere das CLI-Werkzeug global über Cargo, um Konflikte in gehärteten Node/NSolid-Umgebungen komplett zu vermeiden:
  ```bash
  cargo install tauri-cli --version "^2.0.0" --locked
  ```
* **Node.js / NSolid:** Installiere Node.js v20+ oder NSolid LTS.
* **System-Build-Abhängigkeiten:**
  * **Debian / Ubuntu:** `sudo apt update && sudo apt install -y build-essential curl wget file libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  * **Arch Linux / CachyOS:** `sudo pacman -S --needed base-devel curl wget openssl gtk3 webkit2gtk appindicator-gtk3 librsvg patchelf`
  * **Windows:** Installiere die *Microsoft Visual C++ Build Tools* und das *WebView2 SDK*.

#### B. Kompilieren & Ausführen
```bash
# 1. Repository klonen
git clone https://github.com/SoulInfernoDE/retrolan-vpn.git
cd retrolan-vpn

# 2. Web-Abhängigkeiten des Tauri-Frontends installieren (Vite 8 & Tailwind)
npm install

# 3. App kompilieren und über die native Rust CLI im Entwicklungsmodus starten
cargo tauri dev

# 4. Optimiertes Release-Paket und Installer generieren
cargo tauri build
```
Die fertigen Installationsdateien und ausführbaren Binärdateien finden sich anschließend unter `target/release/bundle/`.

---

## ⚖️ Lizenz & Hinweise
Veröffentlicht unter der MIT-Lizenz. Details siehe `LICENSE`.
Hinweise zu den genutzten Open-Source-Technologien (WireGuard, BoringTun, Wintun, Steamworks, Wine/Proton) finden sich in der Datei [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).