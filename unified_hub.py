import asyncio
import json
import logging
import argparse
import socket
import sys
import os
from aiohttp import web, WSCloseCode
from zeroconf import IPVersion, ServiceInfo, Zeroconf

# Terminal Encoding Saftey
if sys.stdout.encoding.lower() != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except AttributeError:
        import codecs
        sys.stdout = codecs.getwriter("utf-8")(sys.stdout.detach())

logging.basicConfig(level=logging.INFO, format='%(asctime)s [%(levelname)s] %(message)s')

# --- State ---
clients = {}            # {client_id: {"ws": ws, "name": name, "ip": ip}}
FAILED_ATTEMPTS = {}    # {ip: count}
PASSKEY = "drone123"

def get_local_ip():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        s.connect(('8.8.8.8', 1))
        ip = s.getsockname()[0]
    except Exception:
        ip = '127.0.0.1'
    finally:
        s.close()
    return ip

# --- WebSocket Handler ---
async def websocket_handler(request):
    ws = web.WebSocketResponse()
    await ws.prepare(request)
    
    client_ip = request.remote
    client_id = None
    client_name = None
    
    # 1. IP Check
    if FAILED_ATTEMPTS.get(client_ip, 0) >= 3:
        logging.warning(f"🚫 Connection blocked from banned IP: {client_ip}")
        await ws.send_json({"type": "error", "message": "Banned due to too many failed attempts."})
        await ws.close()
        return ws

    try:
        async for msg in ws:
            if msg.type == web.WSMsgType.TEXT:
                try:
                    data = json.loads(msg.data)
                except:
                    continue
                    
                m_type = data.get("type") or data.get("msg_type")
                
                # --- A. Authentication Handshake ---
                if m_type == "init":
                    provided_key = data.get("passkey")
                    if provided_key != PASSKEY:
                        FAILED_ATTEMPTS[client_ip] = FAILED_ATTEMPTS.get(client_ip, 0) + 1
                        left = 3 - FAILED_ATTEMPTS[client_ip]
                        logging.warning(f"❌ Invalid passkey (Try {FAILED_ATTEMPTS[client_ip]}/3) from {client_ip}")
                        await ws.send_json({"type": "error", "message": f"Invalid Passkey. {left} attempts left."})
                        await ws.close()
                        return ws
                    
                    client_id = data.get("client_id")
                    client_name = data.get("client_name")
                    clients[client_id] = {"ws": ws, "name": client_name, "ip": client_ip}
                    FAILED_ATTEMPTS[client_ip] = 0 # Reset failures
                    
                    logging.info(f"✅ Device '{client_name}' ({client_id}) authenticated")
                    await ws.send_json({"type": "auth_success", "client_id": client_id})
                    
                    # Send device list
                    device_list = [[cid, {"name": c["name"]}] for cid, c in clients.items()]
                    await ws.send_json({"msg_type": "DeviceList", "payload": json.dumps(device_list).encode().hex()})

                # --- B. Key Exchange & Routing ---
                elif m_type == "KeyExchange":
                    target = data.get("receiver")
                    if target == "drone": # Broadcast the keys to everyone else
                        for cid, c in clients.items():
                            if cid != client_id:
                                await c["ws"].send_str(msg.data)
                    elif target in clients:
                        await clients[target]["ws"].send_str(msg.data)
                
                elif m_type == "Text":
                    target = data.get("receiver")
                    if target in clients:
                        await clients[target]["ws"].send_str(msg.data)
                        logging.info(f"🔄 Relayed message: {client_id} -> {target}")
                
                elif m_type == "Broadcast":
                    for cid, c in clients.items():
                        if cid != client_id:
                            await c["ws"].send_str(msg.data)
                    logging.info(f"📡 Broadcast from {client_id}")
                
                elif m_type == "Ping":
                    await ws.send_json({"msg_type": "Pong", "timestamp": data.get("timestamp")})

            elif msg.type == web.WSMsgType.ERROR:
                logging.error(f"WS error: {ws.exception()}")

    finally:
        if client_id and client_id in clients:
            del clients[client_id]
            logging.info(f"🚪 Device {client_name} disconnected")
            
    return ws

# --- Static File Serving ---
async def index_handler(request):
    return web.FileResponse('web_app/index.html')

def setup_app(app):
    # Static files from web_app directory
    app.router.add_get('/', index_handler)
    app.router.add_get('/ws', websocket_handler)
    app.router.add_static('/', 'web_app', show_index=True)

async def main():
    parser = argparse.ArgumentParser(description="Unified Drone Mesh Gateway")
    parser.add_argument("--password", default="drone123", help="Mesh Passkey")
    parser.add_argument("--port", type=int, default=8080, help="Gateway port (default: 8080)")
    args = parser.parse_args()
    
    global PASSKEY
    PASSKEY = args.password
    
    app = web.Application()
    setup_app(app)
    
    local_ip = get_local_ip()
    zeroconf = Zeroconf()
    
    # Register mDNS
    try:
        desc = {'version': '1.0.0', 'security': 'pqc'}
        info = ServiceInfo(
            "_http._tcp.local.",
            "Drone Unified Mesh._http._tcp.local.",
            addresses=[socket.inet_aton(local_ip)],
            port=args.port,
            properties=desc,
            server="drone-mesh.local.",
        )
        zeroconf.register_service(info)
        logging.info(f"🛰️ Broadcasting as 'http://drone-mesh.local:{args.port}'")
    except Exception as e:
        logging.warning(f"⚠️ mDNS failed: {e}")

    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, '0.0.0.0', args.port)
    
    print("\n" + "="*60)
    print("🚁 UNIFIED DRONE MESH GATEWAY INITIALIZED")
    print("="*60)
    print(f"📡 Serving Dashboard & Hub at http://{local_ip}:{args.port}")
    print(f"🔐 Security: Post-Quantum + Passkey Auth")
    print("="*60)

    try:
        await site.start()
        await asyncio.Future() # Run forever
    finally:
        await runner.cleanup()
        zeroconf.close()

if __name__ == "__main__":
    asyncio.run(main())
