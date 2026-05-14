import http.server
import socketserver
import os
import socket
from zeroconf import IPVersion, ServiceInfo, Zeroconf
import sys

if sys.stdout.encoding.lower() != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except AttributeError:
        import codecs
        sys.stdout = codecs.getwriter("utf-8")(sys.stdout.detach())

PORT = 8080
DIRECTORY = "web_app"

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

class WASMHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        super().end_headers()

# Register WASM mime type
WASMHandler.extensions_map.update({
    '.wasm': 'application/wasm',
    '.js': 'application/javascript',
})

local_ip = get_local_ip()
try:
    zeroconf = Zeroconf()
    desc = {'version': '1.0.0', 'security': 'pqc'}
    info = ServiceInfo(
        "_http._tcp.local.",
        "Drone Mesh Web._http._tcp.local.",
        addresses=[socket.inet_aton(local_ip)],
        port=PORT,
        properties=desc,
        server="drone-mesh.local.",
    )
    print(f"🛰️ Broadcasting as 'http://drone-mesh.local:{PORT}'")
    zeroconf.register_service(info)
except Exception as e:
    print(f"⚠️ Local Discovery (mDNS) failed to start: {e}")
    print("🔗 You can still connect using the IP address directly.")
    zeroconf = None
    info = None

print(f"🚀 Mission Control Server starting...")
print(f"📡 Serving '{DIRECTORY}' on http://{local_ip}:{PORT}")
print(f"🔐 Quantum Cryptography (WASM) support: ENABLED")

try:
    with socketserver.TCPServer(("", PORT), WASMHandler) as httpd:
        httpd.serve_forever()
except KeyboardInterrupt:
    print("\n🛑 Server stopping...")
finally:
    if zeroconf and info:
        zeroconf.unregister_service(info)
        zeroconf.close()
