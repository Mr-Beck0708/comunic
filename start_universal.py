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

def get_primary_ip():
    """Get the primary IP address used for internet/network access."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        # Doesn't need to be reachable, just triggers the routing table lookup
        s.connect(('10.255.255.255', 1))
        IP = s.getsockname()[0]
    except Exception:
        IP = '127.0.0.1'
    finally:
        s.close()
    return IP

def get_all_ips():
    primary = get_primary_ip()
    ips = [primary] if primary != "127.0.0.1" else []
    
    try:
        # Get all network interfaces for fallback
        import subprocess
        output = subprocess.check_output("ipconfig", shell=True).decode()
        import re
        matches = re.findall(r"IPv4 Address\. . . . . . . . . . . : ([\d\.]+)", output)
        for ip in matches:
            if ip != "127.0.0.1" and ip not in ips:
                ips.append(ip)
    except Exception:
        pass
        
    return ips if ips else ["127.0.0.1"]

def start_mesh():
    active_ips = get_all_ips()
    primary_ip = active_ips[0]
    
    print("\n" + "="*60)
    print("🚁 UNIVERSAL DYNAMIC MESH CONTROLLER")
    print("="*60)
    print(f"🚀 PRIMARY IP:   {primary_ip}")
    print(f"🔗 DASHBOARD:    http://{primary_ip}:8080")
    print(f"🛰️ mDNS:         http://drone-mesh.local:8080")
    print("="*60)

    # Start Unified Gateway
    global mesh_proc
    print("\n🚀 Starting Unified Secure Gateway (Dashboard + Hub)...")
    mesh_proc = subprocess.Popen([sys.executable, "unified_hub.py", "--password", "drone123"])
    
    time.sleep(2)
    print("\n✅ SECURE DYNAMIC MESH IS LIVE!")
    print("-" * 40)
    print(f"🌐 ACCESS ANYWHERE (Same Wi-Fi):")
    print(f"   URL:  http://drone-mesh.local:8080")
    for ip in active_ips:
        print(f"   (Or:  http://{ip}:8080)")
    print("-" * 40)
    print(f"🌍 GLOBAL TUNNEL (Any Network):")
    print(f"   Run this in a NEW terminal for global 5G/LTE access:")
    print(f"   Command: npx localtunnel --port 8080")
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
