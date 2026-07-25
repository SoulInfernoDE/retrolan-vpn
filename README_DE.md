<div align="center">
  <img src="assets/logo.svg" alt="RetroLAN Logo" width="220" />
  <h1>🌐 RetroLAN-VPN</h1>
  <p><b>Zero-Config Layer 2/3 LAN & Steam-Relay VPN-Engine für Retro- & Modern-Gaming</b></p>
  <p><i>[ <a href="README.md">🇬🇧 Read in English / Auf Englisch lesen</a> ]</i></p>
</div>

---

## 🚀 Über das Projekt
RetroLAN-VPN schließt die Lücke zwischen klassischem LAN-Gaming und moderner Netzwerksicherheit. Entwickelt mit **Rust, WireGuard (`boringtun`/`Wintun`) und Steamworks**, verbindet die App ultraschnelle Layer-3-Verschlüsselung im User-Space mit einem automatischen Layer-2-UDP-Broadcast-Reflektor.

### ✨ Hauptfunktionen
* **Zero-Config NAT & IPv6-Traversal:** Wählt automatisch zwischen direktem IPv6-P2P, IPv4-Hole-Punching oder weicht nahtlos auf das **Steam Relay Network (SDR)** von Valve aus, um DS-Lite und CGNAT zu überwinden.
* **IPX-to-UDP-Wrapping:** Fängt alte IPX/SPX-Pakete über einen `wsock32.dll`-Hook ab und tunnelt sie unsichtbar über modernes IPv4.
* **Linux-First & Proton-Manager:** Setzt automatisches `WINE_BIND_IP`-Binding und lädt bei Bedarf optimale Tools wie **Proton-GE** oder **Proton-CachyOS** herunter.
* **Kein System-Eingriff:** Strikter Split-Tunneling-Ansatz (nur Spieldaten nutzen das VPN, dein normales Internet bleibt unberührt). Entfernt virtuelle Adapter beim Beenden spurlos.
* **Offline-LAN-Modus:** Physische lokale Erkennung über mDNS für echte Keller-LAN-Partys ganz ohne Internet oder Steam.
* **Drop-in Mod- & Sprach-Schnittstelle:** Neue Spiele (`games.toml`) und Sprachen (`locales/*.toml`) können per einfacher Textdatei von der Community erweitert werden.

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

### 2. Installation
* **Linux:** Lade das neueste `.AppImage` oder das passende `.deb` / `.rpm` Paket von der [Releases-Seite](https://github.com/SoulInfernoDE/retrolan-vpn/releases) herunter.
* **Windows:** Lade die Datei `RetroLAN-Setup.exe` herunter und führe sie aus.
* **Portabler Modus:** Entpacke das Archiv einfach in einen beliebigen Ordner und starte die Anwendung direkt; alle Konfigurationsdateien werden lokal im Verzeichnis verlegt.

### 3. Schritt-für-Schritt-Bedienung
1. **RetroLAN-VPN starten:** Die App scant beim Start automatisch deine installierten Proton-Versionen und prüft deinen Netzwerkstatus.
2. **Lobby erstellen oder beitreten:**
   * **Online (Steam):** Klicke auf **„Lobby erstellen“**, um Steam-Freunde einzuladen. Sollte eine direkte P2P-Verbindung an einem strengen DS-Lite-/CGNAT-Router scheitern, schaltet die App sofort unsichtbar auf Valves weltweite SDR-Relay-Server um.
   * **Offline (Physisches LAN):** Wähle den **„Offline-Modus“**, um Freunde am selben Switch oder WLAN per lokaler mDNS-Suche zu finden.
3. **Spiel starten:** Starte dein Retro-Spiel normal über Steam oder aus dem Ordner. RetroLAN erkennt den Titel anhand der `games.toml` und konfiguriert im Hintergrund automatisch das passende Profil (z. B. IPX-Wrapper-Injektion oder Interface-Binding).
4. **Losspielen:** Genieße klassische LAN-Spiele mit minimalem Ping und komplett ohne lästige Router- oder Port-Freigaben!

---

## ⚖️ Lizenz & Hinweise
Veröffentlicht unter der MIT-Lizenz. Details siehe `LICENSE`.
Hinweise zu den genutzten Open-Source-Technologien (WireGuard, BoringTun, Wintun, Steamworks, Wine/Proton) finden sich in der Datei [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
