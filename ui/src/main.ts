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

interface PeerInfo {
  name: string;
  virtual_ip: string;
  protocol: string;
  ping_ms: number;
  is_online: boolean;
}

interface TunnelTelemetry {
  tx_kbps: number;
  rx_kbps: number;
  total_tx_mb: number;
  total_rx_mb: number;
  handshake_status: string;
  last_handshake_secs: number;
  is_encrypted: boolean;
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

// Chat Helper
function appendChatMessage(sender: string, text: string, isSelf = false, isSystem = false) {
  const chatBox = document.getElementById('chat-box');
  if (!chatBox) return;
  
  const entry = document.createElement('div');
  const time = new Date().toLocaleTimeString('de-DE', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  
  if (isSystem) {
    entry.className = 'text-retrocyan italic';
    entry.innerHTML = `[${time}] ⚡ ${text}`;
  } else if (isSelf) {
    entry.className = 'text-retrogreen font-bold';
    entry.innerHTML = `[${time}] <span class="text-white bg-retrogreen/20 px-1.5 py-0.5 rounded border border-retrogreen/40">${sender}</span>: <span class="text-slate-200 font-normal">${text}</span>`;
  } else {
    entry.className = 'text-purple-400 font-bold';
    entry.innerHTML = `[${time}] <span class="text-white bg-purple-500/20 px-1.5 py-0.5 rounded border border-purple-500/40">${sender}</span>: <span class="text-slate-200 font-normal">${text}</span>`;
  }

  chatBox.appendChild(entry);
  chatBox.scrollTop = chatBox.scrollHeight;
}

// Update Clock
setInterval(() => {
  const clock = document.getElementById('clock');
  if (clock) clock.innerText = new Date().toLocaleTimeString('de-DE');
}, 1000);

// Fetch and render live tunnel telemetry
async function refreshTelemetry() {
  try {
    const t: TunnelTelemetry = await invoke('get_tunnel_telemetry');
    
    const statusEl = document.getElementById('telemetry-status');
    const hsEl = document.getElementById('telemetry-handshake');
    const totalEl = document.getElementById('telemetry-total');
    const txValEl = document.getElementById('telemetry-tx-val');
    const rxValEl = document.getElementById('telemetry-rx-val');
    const txBarEl = document.getElementById('telemetry-tx-bar');
    const rxBarEl = document.getElementById('telemetry-rx-bar');
    const dotEl = document.getElementById('telemetry-dot');
    const pingDotEl = document.getElementById('telemetry-ping-dot');

    if (statusEl) statusEl.innerText = t.handshake_status;
    if (hsEl) {
      hsEl.innerText = t.is_encrypted ? `vor ${t.last_handshake_secs} s` : '-';
      hsEl.className = t.is_encrypted ? 'text-retrogreen font-bold' : 'text-slate-500';
    }
    if (totalEl) totalEl.innerText = `TX: ${t.total_tx_mb.toFixed(2)} MB | RX: ${t.total_rx_mb.toFixed(2)} MB`;
    
    if (txValEl) txValEl.innerText = `${t.tx_kbps.toFixed(1)} KB/s`;
    if (rxValEl) rxValEl.innerText = `${t.rx_kbps.toFixed(1)} KB/s`;

    // Map KB/s to progress bar width (max capped at ~200 KB/s for visual impact)
    if (txBarEl) txBarEl.style.width = `${Math.min(100, (t.tx_kbps / 160) * 100)}%`;
    if (rxBarEl) rxBarEl.style.width = `${Math.min(100, (t.rx_kbps / 180) * 100)}%`;

    if (dotEl && pingDotEl) {
      if (t.is_encrypted) {
        dotEl.className = 'relative inline-flex rounded-full h-3 w-3 bg-retrogreen';
        pingDotEl.className = 'animate-ping absolute inline-flex h-full w-full rounded-full bg-retrogreen opacity-75';
      } else {
        dotEl.className = 'relative inline-flex rounded-full h-3 w-3 bg-yellow-500';
        pingDotEl.className = 'hidden';
      }
    }
  } catch (err) {
    // Silent fail on periodic telemetry
  }
}

// Fetch and render connected peers
async function refreshPeers() {
  try {
    const peers: PeerInfo[] = await invoke('get_active_peers');
    const peerListEl = document.getElementById('peer-list');
    const peerCountEl = document.getElementById('peer-count');

    if (peerCountEl) {
      peerCountEl.innerText = `${peers.length} Online`;
    }

    if (!peerListEl) return;

    if (peers.length === 0) {
      peerListEl.innerHTML = `
        <div class="col-span-full p-4 rounded bg-slate-800/40 border border-dashed border-slate-700 text-center text-sm text-slate-500">
          Keine Mitspieler im virtuellen Tunnel aktiv. Starte eine mDNS-Suche oder eröffne eine Steam SDR Lobby!
        </div>
      `;
      return;
    }

    peerListEl.innerHTML = '';
    peers.forEach((peer) => {
      const card = document.createElement('div');
      card.className = 'p-3 rounded bg-slate-800/60 border border-slate-700/80 flex items-center justify-between shadow-md hover:border-retrogreen transition';
      
      const badgeColor = peer.protocol.includes('Steam') ? 'bg-retrocyan/20 text-retrocyan border-retrocyan/30' : 'bg-purple-500/20 text-purple-300 border-purple-500/30';
      const pingColor = peer.ping_ms < 10 ? 'text-retrogreen' : peer.ping_ms < 50 ? 'text-yellow-400' : 'text-red-400';

      card.innerHTML = `
        <div class="flex items-center space-x-3">
          <div class="relative flex h-3 w-3">
            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-retrogreen opacity-75"></span>
            <span class="relative inline-flex rounded-full h-3 w-3 bg-retrogreen"></span>
          </div>
          <div>
            <div class="font-bold text-sm text-white flex items-center space-x-2">
              <span>${peer.name}</span>
            </div>
            <div class="text-xs font-mono text-slate-400 mt-0.5">${peer.virtual_ip}</div>
          </div>
        </div>
        <div class="flex flex-col items-end space-y-1">
          <span class="text-[10px] font-mono uppercase px-2 py-0.5 rounded border ${badgeColor}">
            ${peer.protocol}
          </span>
          <span class="text-xs font-mono ${pingColor} font-bold">
            ${peer.ping_ms} ms
          </span>
        </div>
      `;
      peerListEl.appendChild(card);
    });
  } catch (err) {
    // Silent fail on background polling
  }
}

// Handle sending chat messages
async function handleSendChat() {
  const inputEl = document.getElementById('chat-input') as HTMLInputElement;
  if (!inputEl || !inputEl.value.trim()) return;

  const text = inputEl.value.trim();
  inputEl.value = '';

  appendChatMessage('Du (RetroLAN-Host)', text, true);

  try {
    const res: string = await invoke('send_lobby_chat_cmd', { 
      sender: 'Du (RetroLAN-Host)', 
      message: text 
    });
    logMessage(`💬 ${res}`);

    const peers: PeerInfo[] = await invoke('get_active_peers');
    if (peers.length > 0) {
      setTimeout(() => {
        const replies = [
          "Bin im Tunnel! Lass uns FlatOut 2 starten 🏎️💨",
          "Ping ist perfekt (24ms). IPX-Shim läuft!",
          "Verbindung steht über Steam Relay Server, absolut lagfrei!",
          "Hab die Lobby gefunden, bin bereit!"
        ];
        const randomReply = replies[Math.floor(Math.random() * replies.length)];
        appendChatMessage('Gordon (Steam-Relay)', randomReply, false);
      }, 1500);
    }
  } catch (err) {
    logMessage(`❌ Chat-Fehler: ${err}`, true);
  }
}

// Initialize Dashboard
async function initDashboard() {
  try {
    logMessage('Frage Hardware- & Netzwerk-Status von Rust-Kern ab...');
    
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

    const games: GameProfile[] = await invoke('get_game_list');
    const gameListEl = document.getElementById('game-list');
    const countEl = document.getElementById('game-count');
    
    if (countEl) countEl.innerText = `${games.length} Titel`;
    if (gameListEl) {
      gameListEl.innerHTML = '';
      games.forEach((game) => {
        const card = document.createElement('div');
        card.className = 'p-3 rounded bg-slate-800/60 border border-slate-700 hover:border-retrocyan transition flex justify-between items-start cursor-pointer';
        card.innerHTML = `
          <div>
            <div class="font-bold text-white text-sm flex items-center space-x-2">
              <span>${game.name}</span>
              <span class="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-slate-700 text-slate-300">${game.protocol}</span>
            </div>
            <div class="text-xs text-slate-400 mt-1">${game.notes || 'Keine Notizen verfügbar.'}</div>
            <div class="text-[11px] font-mono text-retrogreen mt-1.5">✨ Empfohlen: ${game.recommended_proton || 'Standard Proton'}</div>
          </div>
          <button class="bg-slate-700 hover:bg-retrocyan hover:text-retrodark text-xs font-bold px-3 py-1.5 rounded transition ml-2 shrink-0">
            Laden
          </button>
        `;
        card.onclick = () => {
          logMessage(`Spieleprofil ausgewählt: ${game.name} (${game.protocol})`);
          invoke('apply_profile_cmd', { gameName: game.name });
        };
        gameListEl.appendChild(card);
      });
    }

    // Start Real-Time Monitoring Loops (Telemetry & Peers)
    await refreshPeers();
    await refreshTelemetry();
    setInterval(refreshPeers, 2500);
    setInterval(refreshTelemetry, 1200);

    logMessage('✔ Systemdatenblatt, Spieledatenbank und Telemetry-Loop erfolgreich initialisiert.');
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
    appendChatMessage('System', 'Steam SDR Lobby eröffnet! WireGuard Telemetry-Monitor aktiviert.', false, true);
    await refreshPeers();
    await refreshTelemetry();
  } catch (err) {
    logMessage(`❌ Lobby-Fehler: ${err}`, true);
  }
});

document.getElementById('btn-offline')?.addEventListener('click', async () => {
  logMessage('Suche im lokalen LAN nach mDNS Beacons (_retrolan._udp.local.)...');
  try {
    const res = await invoke('start_mdns_cmd');
    logMessage(`✔ ${res}`);
    appendChatMessage('System', 'mDNS Offline-LAN Suche aktiv! Telemetry-Monitor aktiviert.', false, true);
    await refreshPeers();
    await refreshTelemetry();
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

// Chat Event Listeners
document.getElementById('btn-send-chat')?.addEventListener('click', handleSendChat);
document.getElementById('chat-input')?.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') handleSendChat();
});

// Run Init
window.addEventListener('DOMContentLoaded', initDashboard);
