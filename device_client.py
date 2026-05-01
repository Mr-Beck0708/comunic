import asyncio
import websockets
import json
import logging
import argparse
import sys
from drone_crypto import PyKyberKem, PyDilithiumSignature

if sys.stdout.encoding.lower() != 'utf-8':
    sys.stdout.reconfigure(encoding='utf-8')

logging.basicConfig(level=logging.INFO, format='%(asctime)s [%(levelname)s] %(message)s')

class ClientDevice:
    def __init__(self, device_id, device_name):
        self.device_id = device_id
        self.device_name = device_name
        self.kyber = PyKyberKem()
        self.dilithium = PyDilithiumSignature()
        self.kyber_pub, self.kyber_sec = self.kyber.generate_keypair()
        self.dilithium_sec, self.dilithium_pub = self.dilithium.generate_keypair()
        self.peers = {}

    async def connect(self, drone_addr, passkey="drone123"):
        uri = f"ws://{drone_addr}"
        logging.info(f"Connecting to drone at {uri}...")
        
        try:
            async with websockets.connect(uri) as websocket:
                logging.info(f"✅ Connected to central drone as '{self.device_name}'")
                
                # Send Init with Passkey
                init_msg = {
                    "client_id": self.device_id,
                    "client_name": self.device_name,
                    "type": "init",
                    "passkey": passkey
                }
                await websocket.send(json.dumps(init_msg))
                
                # Wait for Auth Confirmation
                auth_resp = await websocket.recv()
                auth_msg = json.loads(auth_resp)
                
                if auth_msg.get("type") == "error":
                    logging.error(f"❌ Auth Failed: {auth_msg.get('message')}")
                    return
                elif auth_msg.get("type") != "auth_success":
                    logging.error("❌ Unexpected handshake response from hub")
                    return
                
                logging.info("🛡️ Authenticated successfully")

                # Build Key Payload
                keys_payload = {
                    "name": self.device_name,
                    "kyber": bytes(self.kyber_pub).hex(),
                    "dilithium": bytes(self.dilithium_pub).hex()
                }
                keys_json = json.dumps(keys_payload)
                key_msg = {
                    "msg_type": "KeyExchange",
                    "sender": self.device_id,
                    "receiver": "drone",
                    "timestamp": "2026-04-20T00:00:00Z",
                    "payload": keys_json.encode('utf-8').hex(),
                    "signature": None
                }
                await websocket.send(json.dumps(key_msg))
                
                # Setup receive loop
                recv_task = asyncio.create_task(self.receive_loop(websocket))
                
                # Setup interactive loop
                print("="*60)
                print(f"💬 DEVICE '{self.device_name}' - Connected to Drone Mesh via Python")
                print("Commands:")
                print("  @device_id message - Send private post-quantum encrypted message")
                print("  /list              - List connected devices")
                print("  /quit              - Disconnect")
                print("="*60)
                
                loop = asyncio.get_event_loop()
                while True:
                    line = await loop.run_in_executor(None, sys.stdin.readline)
                    line = line.strip()
                    if not line:
                        continue
                        
                    if line == "/quit":
                        break
                    elif line == "/list":
                        print("Connected peers:")
                        for pid, pdata in self.peers.items():
                            print(f"  - {pdata['name']} ({pid})")
                    elif line.startswith("@"):
                        parts = line.split(" ", 1)
                        if len(parts) == 2:
                            target_id = parts[0][1:]
                            message = parts[1]
                            if target_id in self.peers:
                                # Use highly secure Rust code seamlessly in Python!
                                target_kyber = bytes.fromhex(self.peers[target_id]["kyber"])
                                encrypted = self.kyber.encrypt_message(message, target_kyber)
                                signature = self.dilithium.sign(bytes(encrypted), bytes(self.dilithium_sec))
                                
                                out_msg = {
                                    "msg_type": "Text",
                                    "sender": self.device_id,
                                    "receiver": target_id,
                                    "timestamp": "2026-04-20",
                                    "payload": bytes(encrypted).hex(),
                                    "signature": bytes(signature).hex()
                                }
                                await websocket.send(json.dumps(out_msg))
                                print(f"📤 Encrypted & Sent to {target_id} via Rust Kyber+Dilithium")
                            else:
                                print(f"❌ Unknown device in mesh: {target_id}")

                recv_task.cancel()
        except Exception as e:
            logging.error(f"Error: {e}")

    async def receive_loop(self, websocket):
        try:
            async for msg_str in websocket:
                msg = json.loads(msg_str)
                msg_type = msg.get("msg_type")
                
                if msg_type == "Text":
                    sender = msg.get("sender")
                    enc_hex = msg.get("payload")
                    sig_hex = msg.get("signature")
                    if enc_hex and sig_hex and sender in self.peers:
                        encrypted = bytes.fromhex(enc_hex)
                        signature = bytes.fromhex(sig_hex)
                        peer_dili = bytes.fromhex(self.peers[sender]["dilithium"])
                        
                        # Verify & Decrypt via Rust Module
                        valid = self.dilithium.verify(encrypted, signature, peer_dili)
                        if valid:
                            decrypted = self.kyber.decrypt_message(encrypted, bytes(self.kyber_sec))
                            print(f"\n🔐 📨 [Encrypted by {self.peers[sender]['name']}] {sender}: {decrypted}")
                        else:
                            print(f"\n🚨 [WARNING] Forged signature from {sender} detected!")
                    else:
                        print(f"\n🔐 📨 [Encrypted message from {sender}] - Waiting for Peer Keys...")
                            
                elif msg_type == "KeyExchange":
                    sender = msg.get("sender")
                    if sender != self.device_id:
                        keys_json = bytes.fromhex(msg.get("payload")).decode('utf-8')
                        keys = json.loads(keys_json)
                        self.peers[sender] = keys
                        print(f"\n📡 Learned Post-Quantum keys for device {keys.get('name')} ({sender})")
                    
        except asyncio.CancelledError:
            pass
        except Exception as e:
            logging.error(f"Receive loop error: {e}")

async def main():
    parser = argparse.ArgumentParser(description="Drone Mesh Device Client")
    parser.add_argument("--id", required=True, help="Unique Device ID")
    parser.add_argument("--name", required=True, help="Human-readable Device Name")
    parser.add_argument("--host", default="localhost", help="Central Drone IP (default: localhost)")
    parser.add_argument("--port", type=int, default=8080, help="Central Drone Port (default: 8080)")
    parser.add_argument("--password", default="drone123", help="Mesh Passkey (default: drone123)")
    args = parser.parse_args()

    # The new Unified Gateway uses the /ws path
    uri = f"{args.host}:{args.port}/ws"
    client = ClientDevice(args.id, args.name)
    await client.connect(uri, args.password)

if __name__ == "__main__":
    asyncio.run(main())
