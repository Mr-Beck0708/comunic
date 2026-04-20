# 🚁 Drone Mesh: Quantum-Secure Mission Control

A state-of-the-art, post-quantum secure mesh communication system. This project enables a distributed network of drones and operators to communicate over an end-to-end encrypted channel that is resistant to quantum computing attacks.

![Mission Control Preview](https://img.shields.io/badge/Security-Post--Quantum-cyan) ![UI](https://img.shields.io/badge/Aesthetics-Glassmorphic-blueviolet) ![Status](https://img.shields.io/badge/Status-Field--Ready-green)

---

## 🔐 Cryptographic Pillars
The system implements a **Hybrid Cryptographic Model** to ensure absolute privacy and authenticity:

*   **Kyber-1024 (KEM)**: Post-quantum key encapsulation used to establish a shared secret between peers.
*   **Dilithium-G (ML-DSA)**: Multi-level digital signatures to ensure every message is authentic and hasn't been tampered with.
*   **AES-256-GCM**: Military-grade symmetric encryption for the high-speed data payload.
*   **WASM Engine**: The entire crypto stack is written in **Rust** and compiled to **WebAssembly**, allowing your browser to perform heavy-duty math at native speeds.

---

## 🛠️ System Architecture

### 1. The Hub (`drone_network.py`)
The "Central Brain" of the mesh. It acts as a high-speed relay and gatekeeper.
- **Mesh Passkey**: Strictly validates all incoming devices.
- **Brute-Force Protector**: Automatically bans IPs after 3 failed attempts.
- **Zero-Knowledge Relay**: The Hub passes encrypted bytes without ever seeing the contents.

### 2. Mission Control UI (`web_app/`)
A premium, mobile-responsive dashboard for operators.
- **Glassmorphic Design**: Futuristic dark-mode interface with real-time animations.
- **Live Diagnostics**: Real-time tracking of latency (ms), uptime, and mesh node count.
- **Quantum Identity**: One-tap access to your PQC public keys.
- **Ciphertext Debugger**: A special tool to reveal the raw encrypted hex strings flowing over the air.

### 3. Drone Clients (`device_client.py`)
Python-based virtual drones that join the mesh to simulate a real-world fleet.

---

## 🚀 Getting Started (Universal Access)

The easiest way to start the entire mesh is using the **Universal Controller**. This script handles discovery (mDNS), starts the secure hub, and launches the mission control dashboard in one go.

```bash
.\.venv\Scripts\python.exe start_universal.py
```

### 📱 Accessing the Mesh
Once the controller is running, you have two ways to connect:

1.  **Local Access (Same Wi-Fi)**:
    - Open your browser and go to: **`http://drone-mesh.local:8080`**
    - The "Drone Address" will be auto-detected! Just enter your **Mesh Passkey**.

2.  **Global Access (Any Network / 5G)**:
    - If you are away from home, run this in a second terminal:
      `npx localtunnel --port 8080`
    - Open the URL provided by localtunnel on your phone.
    - The system will smartly route your post-quantum traffic through the tunnel.

---

## 🛠️ Individual Components (Manual Start)
If you prefer to start components manually, use these commands:

### 1. The Hub (Router)
```bash
python drone_network.py --password your_passkey
```

### 2. The Web Dashboard
```bash
python serve.py
```

---

## 🎮 Usage Guide

### **Connecting**
1.  Open the web app.
2.  Enter the Hub's IP and your **Mesh Passkey**.
3.  Upon successful auth, the **Mission Control** will unlock.

### **Secure Messaging**
1.  Select a drone from the **Active Mesh Nodes** sidebar.
2.  Type your message and hit **Send**.
3.  The "Verified" badge on received messages confirms the Dilithium signature was validated.

### **Diagnostics**
- Toggle the **Hamburger Menu** to see real-time network stability stats.
- Toggle **Ciphertext: ON** in System Tools to see the raw post-quantum data stream.

---

## ⚙️ Technical Requirements
- **Python 3.13+** (Websockets, Argparse)
- **Rust 1.75+** (pqc-kyber, dilithium-rs, aes-gcm)
- **wasm-pack** (to build the browser engine)
- **Modern Browser** (Safari/Chrome/Firefox with WASM support)

---

### **"Efinal Read" - Project Baseline v1.0.0**
*Securing the skies with the math of tomorrow.*
