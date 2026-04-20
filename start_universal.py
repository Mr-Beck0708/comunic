import subprocess
import socket
import time
import sys
import os
import atexit

mesh_proc = None

def cleanup():
    global mesh_proc
    if mesh_proc:
        print("\n🧹 Final cleanup of background processes...")
        mesh_proc.terminate()
        mesh_proc.wait()

atexit.register(cleanup)

if sys.stdout.encoding.lower() != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except AttributeError:
        # Fallback for older python versions
        import codecs
        sys.stdout = codecs.getwriter("utf-8")(sys.stdout.detach())

def get_all_ips():
    ips = []
    try:
        # Get all network interfaces
        import subprocess
        output = subprocess.check_output("ipconfig", shell=True).decode()
        import re
        # Find all IPv4 addresses
        matches = re.findall(r"IPv4 Address\. . . . . . . . . . . : ([\d\.]+)", output)
        ips = [ip for ip in matches if ip != "127.0.0.1"]
    except Exception:
        pass
    
    # Fallback/Fallback method
    try:
        hostname = socket.gethostname()
        addr_infos = socket.getaddrinfo(hostname, None)
        for info in addr_infos:
            ip = info[4][0]
            if "." in ip and ip != "127.0.0.1" and ip not in ips:
                ips.append(ip)
    except:
        pass
        
    return ips if ips else ["127.0.0.1"]

def start_mesh():
    active_ips = get_all_ips()
    primary_ip = active_ips[0]
    
    print("\n" + "="*60)
    print("🚁 UNIVERSAL DYNAMIC MESH CONTROLLER")
    print("="*60)
    print(f"📍 Network Host: {primary_ip}")
    print(f"🛰️ Local URL:    http://drone-mesh.local:8181")
    print("="*60)

    # Start Unified Gateway
    global mesh_proc
    print("\n🚀 Starting Unified Secure Gateway (Dashboard + Hub)...")
    mesh_proc = subprocess.Popen([sys.executable, "unified_hub.py", "--password", "drone123"])
    
    time.sleep(2)
    print("\n✅ SECURE DYNAMIC MESH IS LIVE!")
    print("-" * 40)
    print(f"🌐 ACCESS ANYWHERE (Same Wi-Fi):")
    print(f"   URL:  http://drone-mesh.local:8181")
    for ip in active_ips:
        print(f"   (Or:  http://{ip}:8181)")
    print("-" * 40)
    print(f"🌍 GLOBAL TUNNEL (Any Network):")
    print(f"   Run this in a NEW terminal for global 5G/LTE access:")
    print(f"   Command: npx localtunnel --port 8181")
    print("-" * 40)
    print("\nPress Ctrl+C to shut down the mesh.")

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("\n\n🛑 Shutting down Mesh...")
        mesh_proc.terminate()
        print("✅ Systems offline.")

if __name__ == "__main__":
    start_mesh()
