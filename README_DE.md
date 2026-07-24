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

---

## ⚖️ Lizenz & Hinweise
Veröffentlicht unter der MIT-Lizenz. Details siehe `LICENSE`.
Hinweise zu den genutzten Open-Source-Technologien (WireGuard, BoringTun, Wintun, Steamworks, Wine/Proton) finden sich in der Datei [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
