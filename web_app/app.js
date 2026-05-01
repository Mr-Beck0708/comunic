/* ============================================
   DRONE MESH - Browser Client Application
   ============================================ */

import init, { WasmKyberKem, WasmDilithiumSignature } from './pkg/drone_crypto.js';

// ---- State ----
let wasmReady = false;
let kyber = null;
let dilithium = null;
let myKyberPub = null;
let myKyberSec = null;
let myDilithiumPub = null;
let myDilithiumSec = null;

let ws = null;
let myDeviceId = '';
let myDeviceName = '';
let peers = {};          // { deviceId: { name, kyber, dilithium } }
let targetDevice = null; // null = broadcast
let autoScroll = true;
let showCiphertext = false;
let connectionTime = 0;
let pingInterval = null;
let uptimeInterval = null;

async function loadWasm() {
    const btn = document.getElementById('connect-btn');
    const btnText = btn.querySelector('.btn-text');
    
    try {
        btnText.textContent = 'Initializing Crypto...';
        btn.disabled = true;

        await init();
        kyber = new WasmKyberKem();
        dilithium = new WasmDilithiumSignature();
        
        const kKeys = kyber.generate_keypair();
        myKyberPub = kKeys[0];
        myKyberSec = kKeys[1];
        
        const dKeys = dilithium.generate_keypair();
        myDilithiumSec = dKeys[0];
        myDilithiumPub = dKeys[1];
        
        wasmReady = true;
        btn.disabled = false;
        btnText.textContent = 'Connect to Mesh';
        console.log("Post-Quantum WASM Cryptography Initialized");
    } catch (e) {
        console.error("WASM Load Failed:", e);
        showConnectError("Crypto initialization failed. Please refresh.");
        btnText.textContent = 'Crypto Error';
    }
}

loadWasm();

// ---- Utility ----
function generateId() {
    return 'WEB_' + Math.random().toString(36).substr(2, 8).toUpperCase();
}

function hexEncode(str) {
    return Array.from(new TextEncoder().encode(str))
        .map(b => b.toString(16).padStart(2, '0')).join('');
}

function hexDecode(hex) {
    const bytes = [];
    for (let i = 0; i < hex.length; i += 2) {
        bytes.push(parseInt(hex.substr(i, 2), 16));
    }
    return new TextDecoder().decode(new Uint8Array(bytes));
}

function bytesToHex(bytes) {
    return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

function hexToBytes(hex) {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
        bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
    }
    return bytes;
}

function timeNow() {
    return new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

// ---- Connection ----
function connectToDrone() {
    const nameInput = document.getElementById('device-name');
    const addrInput = document.getElementById('drone-addr');
    const passInput = document.getElementById('mesh-passkey');
    const btn = document.getElementById('connect-btn');
    const btnText = btn.querySelector('.btn-text');
    const btnLoader = btn.querySelector('.btn-loader');
    const errorEl = document.getElementById('connect-error');

    myDeviceName = nameInput.value.trim() || 'Browser Device';
    let addr = addrInput.value.trim();
    const passkey = passInput.value.trim();
    myDeviceId = generateId();

    // SMART ROUTING: If no address provided, use the unified gateway on the current host
    if (!addr) {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.host; // Host includes the port (e.g. drone-mesh.local:8080)
        addr = `${protocol}//${host}/ws`;
        addrInput.value = addr;
        addSystemMessage(`🔍 Routing via Unified Gateway: ${addr}`);
    } else {
        // Ensure protocol prefix
        if (!addr.startsWith('ws://') && !addr.startsWith('wss://')) {
            addr = 'ws://' + addr;
        }
    }

    // UI loading state
    btnText.style.display = 'none';
    btnLoader.style.display = 'inline-block';
    btn.disabled = true;
    errorEl.style.display = 'none';

    try {
        ws = new WebSocket(addr);
    } catch (e) {
        showConnectError('Invalid address format');
        return;
    }

    ws.onopen = () => {
        if (!wasmReady) {
            showConnectError('Cryptography not ready. Please wait...');
            ws.close();
            return;
        }

        try {
            // Send init message
            ws.send(JSON.stringify({
                type: 'init',
                client_id: myDeviceId,
                client_name: myDeviceName,
                passkey: passkey
            }));

            // Send key exchange with real WASM keys
            const browserKeys = {
                name: myDeviceName,
                kyber: bytesToHex(myKyberPub),
                dilithium: bytesToHex(myDilithiumPub)
            };
            const keyMsg = {
                msg_type: 'KeyExchange',
                sender: myDeviceId,
                receiver: 'drone',
                timestamp: new Date().toISOString(),
                payload: hexEncode(JSON.stringify(browserKeys)),
                signature: null
            };
            ws.send(JSON.stringify(keyMsg));

            updateConnectionStatus(true);
            // NOTE: We no longer switch screens here. We wait for 'auth_success'.
        } catch (err) {
            showConnectError(`Key exchange failed: ${err.message}`);
            ws.close();
        }
    };

    ws.onmessage = (event) => {
        handleIncomingMessage(event.data);
    };

    ws.onerror = (err) => {
        console.error("WebSocket Error:", err);
        showConnectError(`Connection failed. Check Drone address and Firewall.`);
    };

    ws.onclose = () => {
        if (document.getElementById('chat-screen').classList.contains('active')) {
            updateConnectionStatus(false);
            addSystemMessage('Connection to drone lost');
        }
    };

    // Timeout
    setTimeout(() => {
        if (ws && ws.readyState !== WebSocket.OPEN) {
            ws.close();
            showConnectError('Connection timed out. Is the drone running?');
        }
    }, 8000);
}

function showConnectError(msg) {
    const btn = document.getElementById('connect-btn');
    const btnText = btn.querySelector('.btn-text');
    const btnLoader = btn.querySelector('.btn-loader');
    const errorEl = document.getElementById('connect-error');

    btnText.style.display = 'inline';
    btnLoader.style.display = 'none';
    btn.disabled = false;
    errorEl.textContent = msg;
    errorEl.style.display = 'block';
}

function updateConnectionStatus(connected) {
    const dot = document.querySelector('.status-dot');
    const text = document.getElementById('status-text');
    if (connected) {
        dot.classList.add('connected');
        text.textContent = 'Connected';
    } else {
        dot.classList.remove('connected');
        text.textContent = 'Disconnected';
    }
}

function disconnect() {
    if (ws) {
        ws.send(JSON.stringify({
            msg_type: 'Disconnect',
            sender: myDeviceId,
            receiver: 'drone',
            timestamp: new Date().toISOString(),
            payload: null,
            signature: null
        }));
        ws.close();
    }
    // Reset state
    peers = {};
    targetDevice = null;
    document.getElementById('chat-screen').classList.remove('active');
    document.getElementById('connect-screen').classList.add('active');

    const btn = document.getElementById('connect-btn');
    btn.querySelector('.btn-text').style.display = 'inline';
    btn.querySelector('.btn-loader').style.display = 'none';
    btn.disabled = false;
    document.getElementById('connect-error').style.display = 'none';

    // Clear messages
    const container = document.getElementById('messages-container');
    container.innerHTML = `
        <div class="welcome-message" id="welcome-msg">
            <div class="welcome-icon">🛡️</div>
            <h3>Secure Channel Active</h3>
            <p>Messages in this mesh are encrypted end-to-end.</p>
            <p class="welcome-hint">Select a device from the sidebar or broadcast to all.</p>
        </div>`;
    
    stopDiagnostics();
}

// ---- Diagnostics Logic ----
function startDiagnostics() {
    document.getElementById('stat-my-id').textContent = myDeviceId;
    document.getElementById('stat-server').textContent = ws.url.split('//')[1].split('/')[0];
    
    // Latency Heartbeat
    sendPing();
    pingInterval = setInterval(sendPing, 3000);
    
    // Uptime counter
    connectionTime = 0;
    uptimeInterval = setInterval(() => {
        connectionTime++;
        const mins = Math.floor(connectionTime / 60).toString().padStart(2, '0');
        const secs = (connectionTime % 60).toString().padStart(2, '0');
        document.getElementById('stat-uptime').textContent = `${mins}:${secs}`;
    }, 1000);
}

function stopDiagnostics() {
    clearInterval(pingInterval);
    clearInterval(uptimeInterval);
    document.getElementById('stat-latency').textContent = '-- ms';
}

function sendPing() {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
            msg_type: 'Ping',
            timestamp: Date.now()
        }));
    }
}

// ---- Message Handling ----
function handleIncomingMessage(raw) {
    let msg;
    try {
        msg = JSON.parse(raw);
    } catch {
        return;
    }

    const msgType = msg.msg_type || msg.type;

    if (msgType === 'error') {
        // Hub rejected us (e.g. invalid passkey)
        showConnectError(msg.message || 'Connection Error');
        if (ws) ws.close();
        return;
    }

    if (msgType === 'auth_success') {
        // Hub confirmed we are authorized!
        document.getElementById('connect-screen').classList.remove('active');
        document.getElementById('chat-screen').classList.add('active');
        document.getElementById('my-device-label').textContent = myDeviceName;
        
        addSystemMessage(`Encrypted session started for "${myDeviceName}"`);
        startDiagnostics();
        return;
    }

    if (msgType === 'KeyExchange') {
        // A peer announced their keys
        const senderId = msg.sender;
        if (senderId === myDeviceId) return;

        try {
            const keysJson = hexDecode(msg.payload);
            const keys = JSON.parse(keysJson);
            peers[senderId] = {
                name: keys.name || senderId,
                kyber: keys.kyber,
                dilithium: keys.dilithium
            };
            renderDeviceList();
            addSystemMessage(`📡 ${keys.name || senderId} joined the mesh`);
        } catch (e) {
            console.error('Key parse error', e);
        }

    } else if (msgType === 'Text') {
        const senderId = msg.sender;
        if (senderId === myDeviceId) return;

        let text = '';
        try {
            if (wasmReady && peers[senderId] && peers[senderId].dilithium) {
                const encrypted = hexToBytes(msg.payload);
                const signature = hexToBytes(msg.signature);
                const peerDilithiumPub = hexToBytes(peers[senderId].dilithium);
                
                const isValid = dilithium.verify(encrypted, signature, peerDilithiumPub);
                if (isValid) {
                    text = kyber.decrypt_message(encrypted, myKyberSec);
                } else {
                    text = '[⚠️ Cryptographic Signature Mismatch]';
                }
            } else {
                text = '[🔐 Encrypted - Waiting for Peer Keys...]';
            }
        } catch (e) {
            text = '[Decryption failed: ' + e + ']';
        }

        const senderName = (peers[senderId] && peers[senderId].name) || senderId;
        addReceivedMessage(senderName, senderId, text, msg.payload);

    } else if (msgType === 'Broadcast') {
        const senderId = msg.sender;
        if (senderId === myDeviceId) return;

        let text = '';
        try {
            text = hexDecode(msg.payload);
        } catch {
            text = '[Encrypted broadcast]';
        }

        const senderName = (peers[senderId] && peers[senderId].name) || senderId;
        addReceivedMessage(senderName + ' (broadcast)', senderId, text, msg.payload);

    } else if (msgType === 'DeviceList') {
        // Parse the device list from the drone
        try {
            const devicesJson = hexDecode(msg.payload);
            const devices = JSON.parse(devicesJson);
            // devices is an array of [id, { name }]
            for (const [did, info] of devices) {
                if (did !== myDeviceId && !peers[did]) {
                    peers[did] = {
                        name: info.name || did,
                        kyber: null,
                        dilithium: null
                    };
                }
            }
            renderDeviceList();
        } catch (e) {
            console.error('DeviceList parse error', e);
        }
    } else if (msgType === 'Pong') {
        const now = Date.now();
        const sent = msg.timestamp;
        const latency = now - sent;
        document.getElementById('stat-latency').textContent = `${latency}ms`;
    }
}

// ---- Sending Messages ----
function sendMessage() {
    const input = document.getElementById('msg-input');
    const text = input.value.trim();
    if (!text || !ws || ws.readyState !== WebSocket.OPEN) return;

    // Hide welcome message
    const welcome = document.getElementById('welcome-msg');
    if (welcome) welcome.remove();

    const payload = hexEncode(text);

    if (targetDevice && wasmReady && peers[targetDevice] && peers[targetDevice].kyber) {
        try {
            const targetKyberPub = hexToBytes(peers[targetDevice].kyber);
            const encrypted = kyber.encrypt_message(text, targetKyberPub);
            const signature = dilithium.sign(encrypted, myDilithiumSec);
            
            const outMsg = {
                msg_type: 'Text',
                sender: myDeviceId,
                receiver: targetDevice,
                timestamp: new Date().toISOString(),
                payload: bytesToHex(encrypted),
                signature: bytesToHex(signature)
            };
            ws.send(JSON.stringify(outMsg));
            
            const targetName = (peers[targetDevice] && peers[targetDevice].name) || targetDevice;
            addSentMessage(text, `To ${targetName}`, outMsg.payload);
            console.log(`Sent encrypted message to ${targetDevice}`);
        } catch (e) {
            console.error("Encryption error:", e);
            addSystemMessage('Encryption failed: ' + e);
        }
    } else {
        // Broadcast or fallback to simple hex encoding
        const outMsg = {
            msg_type: 'Broadcast',
            sender: myDeviceId,
            receiver: 'all',
            timestamp: new Date().toISOString(),
            payload: payload,
            signature: hexEncode('browser_sig')
        };
        ws.send(JSON.stringify(outMsg));
        addSentMessage(text, 'Broadcast', outMsg.payload);
        console.log("Sent broadcast message");
    }

    input.value = '';
    input.focus();
}

function handleInputKey(e) {
    if (e.key === 'Enter') sendMessage();
}

// ---- UI Rendering ----
function addSystemMessage(text) {
    const container = document.getElementById('messages-container');
    const div = document.createElement('div');
    div.className = 'message-bubble system';
    div.innerHTML = `<span class="msg-text">${text}</span>`;
    container.appendChild(div);
    if (autoScroll) container.scrollTop = container.scrollHeight;
}

function addSentMessage(text, label, ciphertext) {
    const container = document.getElementById('messages-container');
    const div = document.createElement('div');
    div.className = 'message-bubble sent';
    div.innerHTML = `
        <div class="msg-sender">${label} <span class="crypto-badge">Encrypted</span></div>
        <div class="msg-text">${escapeHtml(text)}</div>
        <div class="message-ciphertext">
            <div style="font-weight:bold;margin-bottom:2px">CIPHERTEXT (HEX):</div>
            ${ciphertext || 'N/A'}
        </div>
        <div class="msg-time">${timeNow()}</div>`;
    container.appendChild(div);
    if (autoScroll) container.scrollTop = container.scrollHeight;
}

function addReceivedMessage(senderName, senderId, text, ciphertext) {
    const container = document.getElementById('messages-container');

    // Hide welcome message
    const welcome = document.getElementById('welcome-msg');
    if (welcome) welcome.remove();

    const div = document.createElement('div');
    div.className = 'message-bubble received';
    div.innerHTML = `
        <div class="msg-sender">${escapeHtml(senderName)} <span class="crypto-badge">Verified</span></div>
        <div class="msg-text">${escapeHtml(text)}</div>
        <div class="message-ciphertext">
            <div style="font-weight:bold;margin-bottom:2px">CIPHERTEXT (HEX):</div>
            ${ciphertext || 'N/A'}
        </div>
        <div class="msg-time">${timeNow()}</div>`;
    container.appendChild(div);
    if (autoScroll) container.scrollTop = container.scrollHeight;
}

function escapeHtml(str) {
    const el = document.createElement('span');
    el.textContent = str;
    return el.innerHTML;
}

// ---- Device List / Sidebar ----
function renderDeviceList() {
    const list = document.getElementById('device-list');
    const peerIds = Object.keys(peers);

    document.getElementById('device-count-num').textContent = peerIds.length;

    if (peerIds.length === 0) {
        list.innerHTML = `
            <div class="empty-devices">
                <p>No other devices connected</p>
                <span class="empty-hint">Waiting for mesh peers...</span>
            </div>`;
        return;
    }

    list.innerHTML = '';
    for (const pid of peerIds) {
        const peer = peers[pid];
        const initials = (peer.name || '??').substring(0, 2).toUpperCase();
        const selected = targetDevice === pid ? 'selected' : '';
        const div = document.createElement('div');
        div.className = `device-item ${selected}`;
        div.dataset.id = pid;
        div.addEventListener('click', () => selectTarget(pid));
        div.innerHTML = `
            <div class="device-avatar">${initials}</div>
            <div class="device-info">
                <div class="device-info-name">${escapeHtml(peer.name)}</div>
                <div class="device-info-id">${pid}</div>
            </div>`;
        list.appendChild(div);
    }
}

function selectTarget(deviceId) {
    targetDevice = deviceId;
    const name = (peers[deviceId] && peers[deviceId].name) || deviceId;
    document.getElementById('target-name').textContent = name;
    document.getElementById('target-clear').style.display = 'flex';
    
    addSystemMessage(`Target selected: ${name}`);
    renderDeviceList();
    
    if (window.innerWidth <= 768) {
        toggleSidebar(); 
    }
}

function clearTarget() {
    targetDevice = null;
    document.getElementById('target-name').textContent = 'Everyone (Broadcast)';
    document.getElementById('target-clear').style.display = 'none';
    renderDeviceList();
}

// ---- UI Helpers ----
function toggleSidebar() {
    document.getElementById('sidebar').classList.toggle('open');
    document.getElementById('sidebar-overlay').classList.toggle('visible');
}

function copyKey(type) {
    const key = type === 'kyber' ? bytesToHex(myKyberPub) : bytesToHex(myDilithiumPub);
    navigator.clipboard.writeText(key).then(() => {
        addSystemMessage(`Key Copied: ${type.toUpperCase()}`);
    });
}

function clearChat() {
    const container = document.getElementById('messages-container');
    container.innerHTML = '';
    addSystemMessage('Chat history cleared');
}

function toggleAutoScroll() {
    autoScroll = !autoScroll;
    document.getElementById('scroll-toggle-text').textContent = `Scroll: ${autoScroll ? 'ON' : 'OFF'}`;
}

function toggleCiphertext() {
    showCiphertext = !showCiphertext;
    const container = document.getElementById('messages-container');
    const btnText = document.getElementById('ciphertext-toggle-text');
    const btn = document.getElementById('ciphertext-btn');
    
    if (showCiphertext) {
        container.classList.add('show-ciphertext');
        btnText.textContent = 'Ciphertext: ON';
        btn.style.background = 'rgba(0, 212, 255, 0.15)';
        btn.style.borderColor = 'var(--accent)';
    } else {
        container.classList.remove('show-ciphertext');
        btnText.textContent = 'Ciphertext: OFF';
        btn.style.background = 'var(--glass)';
        btn.style.borderColor = 'var(--glass-border)';
    }
}

// ---- Global Export for HTML event handlers (since we are a module) ----
window.connectToDrone = connectToDrone;
window.toggleSidebar = toggleSidebar;
window.disconnect = disconnect;
window.selectTarget = selectTarget;
window.clearTarget = clearTarget;
window.sendMessage = sendMessage;
window.handleInputKey = handleInputKey;
window.copyKey = copyKey;
window.clearChat = clearChat;
window.toggleAutoScroll = toggleAutoScroll;
window.toggleCiphertext = toggleCiphertext;
