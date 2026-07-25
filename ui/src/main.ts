import { invoke } from '@tauri-apps/api/core';

interface SystemStatus {
  avx2: boolean;
  ntsync: boolean;
  steam_online: boolean;
  mdns_active: boolean;
}

interface GameProfile {
  name: string;
  steam_appid: number | null;
  protocol: string;
  recommended_proton: string | null;
  notes: string | null;
}

// Log Helper
function logMessage(msg: string, isError = false) {
  const logBox = document.getElementById('log-box');
  if (!logBox) return;
  const entry = document.createElement('div');
  const time = new Date().toLocaleTimeString('de-DE');
  entry.className = isError ? 'text-red-400' : 'text-slate-300';
  entry.innerHTML = `<span class="text-slate-500">[${time}]</span> ${msg}`;
  logBox.appendChild(entry);
  logBox.scrollTop = logBox.scrollHeight;
}

// Update Clock
setInterval(() => {
  const clock = document.getElementById('clock');
  if (clock) clock.innerText = new Date().toLocaleTimeString('de-DE');
}, 1000);

// Initialize Dashboard
async function initDashboard() {
  try {
    logMessage('Frage Hardware- & Netzwerk-Status von Rust-Kern ab...');
    
    // 1. Fetch System Status
    const status: SystemStatus = await invoke('get_system_status');
    
    const avxEl = document.getElementById('status-avx');
    if (avxEl) {
      avxEl.innerHTML = status.avx2 
        ? `<span class="inline-block w-2 h-2 rounded-full bg-retrogreen mr-2"></span> AVX2 (x86-64-v3) Aktiv`
        : `<span class="inline-block w-2 h-2 rounded-full bg-red-500 mr-2"></span> Standard x86-64`;
    }

    const ntsyncEl = document.getElementById('status-ntsync');
    if (ntsyncEl) {
      ntsyncEl.innerHTML = status.ntsync
        ? `<span class="inline-block w-2 h-2 rounded-full bg-retrogreen mr-2"></span> /dev/ntsync Geladen`
        : `<span class="inline-block w-2 h-2 rounded-full bg-yellow-500 mr-2"></span> esync / fsync Fallback`;
    }

    const steamEl = document.getElementById('status-steam');
    if (steamEl) {
      steamEl.innerHTML = status.steam_online
        ? `<span class="inline-block w-2 h-2 rounded-full bg-retrocyan mr-2"></span> SDR Relay Verbunden`
        : `<span class="inline-block w-2 h-2 rounded-full bg-slate-500 mr-2"></span> Offline LAN (mDNS)`;
    }

    // 2. Fetch Game Profiles
    const games: GameProfile[] = await invoke('get_game_list');
    const gameListEl = document.getElementById('game-list');
    const countEl = document.getElementById('game-count');
    
    if (countEl) countEl.innerText = `${games.length} Titel`;
    if (gameListEl) {
      gameListEl.innerHTML = '';
      games.forEach((game) => {
        const card = document.createElement('div');
        card.className = 'p-4 rounded bg-slate-800/60 border border-slate-700 hover:border-retrocyan transition flex justify-between items-start cursor-pointer';
        card.innerHTML = `
          <div>
            <div class="font-bold text-white flex items-center space-x-2">
              <span>${game.name}</span>
              <span class="text-xs font-mono uppercase px-2 py-0.5 rounded bg-slate-700 text-slate-300">${game.protocol}</span>
            </div>
            <div class="text-xs text-slate-400 mt-1">${game.notes || 'Keine Notizen verfügbar.'}</div>
            <div class="text-xs font-mono text-retrogreen mt-2">✨ Empfohlen: ${game.recommended_proton || 'Standard Proton'}</div>
          </div>
          <button class="bg-slate-700 hover:bg-retrocyan hover:text-retrodark text-xs font-bold px-3 py-1.5 rounded transition">
            Profil Laden
          </button>
        `;
        card.onclick = () => {
          logMessage(`Spieleprofil ausgewählt: ${game.name} (${game.protocol})`);
          invoke('apply_profile_cmd', { gameName: game.name });
        };
        gameListEl.appendChild(card);
      });
    }

    logMessage('✔ Systemdatenblatt und Spieledatenbank erfolgreich geladen.');
  } catch (err) {
    logMessage(`Fehler beim Laden des Dashboards: ${err}`, true);
  }
}

// Event Listeners for Control Panel Buttons
document.getElementById('btn-host')?.addEventListener('click', async () => {
  logMessage('Erstelle weltweite Steam SDR P2P-Lobby...');
  try {
    const res = await invoke('host_lobby_cmd');
    logMessage(`✔ ${res}`);
  } catch (err) {
    logMessage(`❌ Lobby-Fehler: ${err}`, true);
  }
});

document.getElementById('btn-offline')?.addEventListener('click', async () => {
  logMessage('Suche im lokalen LAN nach mDNS Beacons (_retrolan._udp.local.)...');
  try {
    const res = await invoke('start_mdns_cmd');
    logMessage(`✔ ${res}`);
  } catch (err) {
    logMessage(`❌ mDNS-Fehler: ${err}`, true);
  }
});

document.getElementById('btn-ipx')?.addEventListener('click', async () => {
  logMessage('Erneuere wsock32.dll Proxy-Shim im Spielverzeichnis...');
  try {
    const res = await invoke('deploy_ipx_cmd');
    logMessage(`✔ ${res}`);
  } catch (err) {
    logMessage(`❌ IPX-Fehler: ${err}`, true);
  }
});

// Run Init
window.addEventListener('DOMContentLoaded', initDashboard);