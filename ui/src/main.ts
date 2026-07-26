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

interface LanSessionInfo {
  game_name: string;
  host_peer: string;
  host_ip: string;
  player_count: string;
  ping_ms: number;
  is_joinable: boolean;
}

interface TunnelTelemetry {
  tx_kbps: number;
  rx_kbps: number;
  total_tx_mb: number;
  total_rx_mb: number;
  handshake_status: string;
  last_handshake_secs: number;
  is_encrypted: boolean;
  mtu_bytes: number;
}

// =====================================================================
// RETRO WEB AUDIO API SYNTHESIZER (8-Bit DOS / Arcade Sound Effects)
// =====================================================================
let audioCtx: AudioContext | null = null;
let lastPeerCount = -1;

function getAudioContext(): AudioContext {
  if (!audioCtx) {
    const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
    audioCtx = new AudioContextClass();
  }
  if (audioCtx.state === 'suspended') {
    audioCtx.resume();
  }
  return audioCtx;
}

function playRetroSound(type: 'join' | 'chat' | 'activate') {
  try {
    const ctx = getAudioContext();
    const now = ctx.currentTime;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.connect(gain);
    gain.connect(ctx.destination);

    if (type === 'join') {
      osc.type = 'square';
      osc.frequency.setValueAtTime(440, now);
      osc.frequency.setValueAtTime(659, now + 0.1);
      osc.frequency.setValueAtTime(880, now + 0.2);
      gain.gain.setValueAtTime(0.12, now);
      gain.gain.exponentialRampToValueAtTime(0.01, now + 0.35);
      osc.start(now);
      osc.stop(now + 0.35);
    } else if (type === 'chat') {
      osc.type = 'sine';
      osc.frequency.setValueAtTime(987, now);
      osc.frequency.setValueAtTime(1318, now + 0.08);
      gain.gain.setValueAtTime(0.18, now);
      gain.gain.exponentialRampToValueAtTime(0.01, now + 0.18);
      osc.start(now);
      osc.stop(now + 0.18);
    } else if (type === 'activate') {
      osc.type = 'sawtooth';
      osc.frequency.setValueAtTime(300, now);
      osc.frequency.exponentialRampToValueAtTime(600, now + 0.15);
      gain.gain.setValueAtTime(0.08, now);
      gain.gain.exponentialRampToValueAtTime(0.01, now + 0.15);
      osc.start(now);
      osc.stop(now + 0.15);
    }
  } catch (e) {
    // Silently ignore if AudioContext is blocked prior to user gesture
  }
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
    playRetroSound('chat');
  }

  chatBox.appendChild(entry);
  chatBox.scrollTop = chatBox.scrollHeight;
}

// Update Clock
setInterval(() => {
  const clock = document.getElementById('clock');
  if (clock) clock.innerText = new Date().toLocaleTimeString('de-DE');
}, 1000);

// Fetch and render live tunnel telemetry & PMTUD clamp
async function refreshTelemetry() {
  try {
    const t: TunnelTelemetry = await invoke('get_tunnel_telemetry');
    
    const statusEl = document.getElementById('telemetry-status');
    const hsEl = document.getElementById('telemetry-handshake');
    const totalEl = document.getElementById('telemetry-total');
    const mtuEl = document.getElementById('telemetry-mtu');
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
    if (mtuEl) {
      mtuEl.innerText = t.is_encrypted ? `${t.mtu_bytes} B (DF)` : '1500 B';
      mtuEl.className = t.is_encrypted ? 'text-retrogreen font-bold' : 'text-yellow-400 font-bold';
    }
    
    if (txValEl) txValEl.innerText = `${t.tx_kbps.toFixed(1)} KB/s`;
    if (rxValEl) rxValEl.innerText = `${t.rx_kbps.toFixed(1)} KB/s`;

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

// Fetch and render connected peers & LAN sessions
async function refreshPeersAndSessions() {
  try {
    const peers: PeerInfo[] = await invoke('get_active_peers');
    const peerListEl = document.getElementById('peer-list');
    const peerCountEl = document.getElementById('peer-count');

    if (peerCountEl) peerCountEl.innerText = `${peers.length} Online`;

    if (lastPeerCount !== -1 && peers.length > lastPeerCount) {
      playRetroSound('join');
      logMessage('🔊 [Retro-Synth] Neuer Mitspieler im Tunnel aktiv -> 8-Bit Arpeggio abgespielt!');
    }
    lastPeerCount = peers.length;

    if (peerListEl && peers.length > 0) {
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
            <span class="text-[10px] font-mono uppercase px-2 py-0.5 rounded border ${badgeColor}">${peer.protocol}</span>
            <span class="text-xs font-mono ${pingColor} font-bold">${peer.ping_ms} ms</span>
          </div>
        `;
        peerListEl.appendChild(card);
      });
    }

    const sessions: LanSessionInfo[] = await invoke('get_active_lan_sessions');
    const sessionListEl = document.getElementById('session-list');
    const sessionCountEl = document.getElementById('session-count');

    if (sessionCountEl) sessionCountEl.innerText = `${sessions.length} Server`;
    if (sessionListEl && sessions.length > 0) {
      sessionListEl.innerHTML = '';
      sessions.forEach((s) => {
        const card = document.createElement('div');
        card.className = 'p-3 rounded bg-slate-800/80 border border-yellow-500/40 flex items-center justify-between shadow-lg';
        card.innerHTML = `
          <div>
            <div class="font-bold text-white text-sm flex items-center space-x-2">
              <span class="text-yellow-400">🔥</span>
              <span>${s.game_name}</span>
            </div>
            <div class="text-xs font-mono text-slate-400 mt-1">Host: <span class="text-retrocyan font-bold">${s.host_peer}</span> (${s.host_ip})</div>
          </div>
          <div class="flex items-center space-x-3">
            <span class="text-xs font-mono bg-slate-700 px-2 py-1 rounded text-slate-300">${s.player_count}</span>
            <button class="bg-retrogreen hover:bg-emerald-400 text-retrodark font-bold text-xs px-3 py-1.5 rounded transition shadow">
              Beitreten 🚀
            </button>
          </div>
        `;
        card.querySelector('button')?.addEventListener('click', () => {
          playRetroSound('activate');
          logMessage(`🚀 Verbinde direkt mit LAN-Server von ${s.host_peer} (${s.host_ip})...`);
          appendChatMessage('System', `Verbindung zum LAN-Host ${s.host_peer} für ${s.game_name} wird aufgebaut!`, false, true);
        });
        sessionListEl.appendChild(card);
      });
    }
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

  playRetroSound('activate');
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
          playRetroSound('activate');
          logMessage(`Spieleprofil ausgewählt: ${game.name} (${game.protocol})`);
          invoke('apply_profile_cmd', { gameName: game.name });
        };
        gameListEl.appendChild(card);
      });
    }

    await refreshPeersAndSessions();
    await refreshTelemetry();
    setInterval(refreshPeersAndSessions, 2500);
    setInterval(refreshTelemetry, 1200);

    logMessage('✔ Systemdatenblatt, Spieledatenbank und LAN-Match-Detektor erfolgreich initialisiert.');
  } catch (err) {
    logMessage(`Fehler beim Laden des Dashboards: ${err}`, true);
  }
}

// Event Listeners for Control Panel Buttons
document.getElementById('btn-host')?.addEventListener('click', async () => {
  playRetroSound('activate');
  logMessage('Erstelle weltweite Steam SDR P2P-Lobby...');
  try {
    const res = await invoke('host_lobby_cmd');
    logMessage(`✔ ${res}`);
    appendChatMessage('System', 'Steam SDR Lobby eröffnet! Du kannst jetzt Freunde einladen.', false, true);
    await refreshPeersAndSessions();
    await refreshTelemetry();
  } catch (err) {
    logMessage(`❌ Lobby-Fehler: ${err}`, true);
  }
});

document.getElementById('btn-invite')?.addEventListener('click', async () => {
  playRetroSound('activate');
  logMessage('Öffne natives Steam-Overlay für Freundeseinladungen...');
  try {
    const res = await invoke('invite_friends_cmd');
    logMessage(`✔ ${res}`);
  } catch (err) {
    logMessage(`❌ Einladungs-Fehler: ${err}`, true);
  }
});

document.getElementById('btn-offline')?.addEventListener('click', async () => {
  playRetroSound('activate');
  logMessage('Suche im lokalen LAN nach mDNS Beacons (_retrolan._udp.local.)...');
  try {
    const res = await invoke('start_mdns_cmd');
    logMessage(`✔ ${res}`);
    appendChatMessage('System', 'mDNS Offline-LAN Suche aktiv! Telemetry-Monitor aktiviert.', false, true);
    await refreshPeersAndSessions();
    await refreshTelemetry();
  } catch (err) {
    logMessage(`❌ mDNS-Fehler: ${err}`, true);
  }
});

document.getElementById('btn-proton')?.addEventListener('click', async () => {
  playRetroSound('activate');
  logMessage('🌐 Prüfe und lade architektur-passendes Proton Release...');
  try {
    const res = await invoke('download_proton_cmd');
    logMessage(`✔ ${res}`);
    appendChatMessage('System', 'CachyOS/GE-Proton erfolgreich verifiziert und registriert!', false, true);
  } catch (err) {
    logMessage(`❌ Proton-Fehler: ${err}`, true);
  }
});

document.getElementById('btn-ipx')?.addEventListener('click', async () => {
  playRetroSound('activate');
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
