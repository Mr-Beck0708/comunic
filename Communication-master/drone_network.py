import asyncio
import websockets
import json
import logging

import sys
if sys.stdout.encoding.lower() != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except AttributeError:
        import codecs
        sys.stdout = codecs.getwriter("utf-8")(sys.stdout.detach())

logging.basicConfig(level=logging.INFO, format='%(asctime)s [%(levelname)s] %(message)s')

clients = {}

import argparse
from zeroconf import IPVersion, ServiceInfo, Zeroconf
import socket

# Configuration
FAILED_ATTEMPTS = {}       # {ip: count}
# (Passkey will be read from command line)

def get_local_ip():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        s.connect(('8.8.8.8', 1))
        IP = s.getsockname()[0]
    except Exception:
        IP = '127.0.0.1'
    finally:
        s.close()
    return IP

async def handler(websocket, mesh_passkey):
    client_id = None
    client_name = None
    client_ip = websocket.remote_address[0]
    
    # Check for Ban
    if FAILED_ATTEMPTS.get(client_ip, 0) >= 3:
        logging.warning(f"🚫 Banned IP attempt: {client_ip}")
        await websocket.send(json.dumps({"type": "error", "message": "Access Denied: Too many failed attempts."}))
        await websocket.close()
        return

    try:
        init_str = await websocket.recv()
        init_msg = json.loads(init_str)
        
        if init_msg.get("type") == "init":
            provided_key = init_msg.get("passkey")
            
            if provided_key != mesh_passkey:
                FAILED_ATTEMPTS[client_ip] = FAILED_ATTEMPTS.get(client_ip, 0) + 1
                attempts_left = 3 - FAILED_ATTEMPTS[client_ip]
                logging.warning(f"❌ Invalid passkey (Try {FAILED_ATTEMPTS[client_ip]}/3) from {client_ip}")
                
                await websocket.send(json.dumps({
                    "type": "error", 
                    "message": f"Authentication Failed. {attempts_left} tries left."
                }))
                await websocket.close()
                return
                
            # Success - Reset failure count
            FAILED_ATTEMPTS[client_ip] = 0
            
            client_id = init_msg.get("client_id")
            client_name = init_msg.get("client_name", "Unknown")
            clients[client_id] = {"ws": websocket}
            
            # Send explicit success confirmation
            await websocket.send(json.dumps({"type": "auth_success"}))
            logging.info(f"✅ Device '{client_name}' authenticated and connected")
            
            async for message in websocket:
                msg = json.loads(message)
                msg_type = msg.get("msg_type")
                
                if msg_type == "KeyExchange":
                    logging.info(f"🔐 Key exchange completed with device: {client_name} ({client_id})")
                    clients[client_id]["keys"] = msg
                    
                    # Broadcast this new client's keys to everyone else
                    for cid, cdata in list(clients.items()):
                        if cid != client_id:
                            await cdata["ws"].send(json.dumps(msg))
                            
                    # Send all existing keys to the new client
                    for cid, cdata in list(clients.items()):
                        if cid != client_id and "keys" in cdata:
                            await clients[client_id]["ws"].send(json.dumps(cdata["keys"]))
                            
                elif msg_type == "Text":
                    target_id = msg.get("receiver")
                    if target_id in clients:
                        await clients[target_id]["ws"].send(message)
                        logging.info(f"🔄 Relayed secure message from {client_id} to {target_id}")
                    else:
                        logging.warning(f"Target {target_id} not connected")
                        
                elif msg_type == "Broadcast":
                    for cid, cdata in clients.items():
                        if cid != client_id:
                            await cdata["ws"].send(message)
                    logging.info(f"📡 Secure broadcast relayed from {client_id}")
                    
                elif msg_type == "Ping":
                    pong = {"msg_type": "Pong", "timestamp": msg.get("timestamp")}
                    await websocket.send(json.dumps(pong))
                    
                elif msg_type == "Disconnect":
                    break
    except websockets.exceptions.ConnectionClosed:
        pass
    except Exception as e:
        logging.error(f"Error handling connection: {e}")
    finally:
        if client_id in clients:
            del clients[client_id]
            logging.info(f"Device '{client_name}' ({client_id}) disconnected")


async def main():
    parser = argparse.ArgumentParser(description="Drone Mesh Hub (Router)")
    parser.add_argument("--password", default="drone123", help="Mesh Passkey (default: drone123)")
    parser.add_argument("--port", type=int, default=8888, help="Listening port (default: 8888)")
    args = parser.parse_args()

    local_ip = get_local_ip()
    
    # Register mDNS service
    try:
        zeroconf = Zeroconf()
        desc = {'version': '1.0.0', 'security': 'pqc'}
        info = ServiceInfo(
            "_drone-hub._tcp.local.",
            "Drone Mesh Hub._drone-hub._tcp.local.",
            addresses=[socket.inet_aton(local_ip)],
            port=args.port,
            properties=desc,
            server="drone-hub.local.",
        )
        logging.info(f"🛰️ Broadcasting Hub as 'drone-hub.local' on {local_ip}")
        zeroconf.register_service(info)
    except Exception as e:
        logging.warning(f"⚠️ Local Discovery (mDNS) failed to start: {e}")
        logging.warning("🔗 You can still connect using the IP address directly.")
        zeroconf = None
        info = None

    async def _handler(ws):
        await handler(ws, args.password)

    try:
        async with websockets.serve(_handler, "0.0.0.0", args.port):
            logging.info("============================================================")
            logging.info("🚁 PYTHON CENTRAL DRONE (ROUTER) INITIALIZED")
            logging.info(f"📡 Listening on 0.0.0.0:{args.port}")
            logging.info(f"🔐 Mesh Passkey active: {'[SET]' if args.password else '[BLANK]'}")
            logging.info("============================================================")
            await asyncio.Future()
    except KeyboardInterrupt:
        pass
    finally:
        if zeroconf and info:
            logging.info("🛑 Shutting down mDNS...")
            zeroconf.unregister_service(info)
            zeroconf.close()

if __name__ == "__main__":
    asyncio.run(main())
