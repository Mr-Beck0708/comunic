# Drone Mesh - Windows Startup Script
# Usage: ./start_device.ps1 <device_id> <device_name>

param (
    [Parameter(Mandatory=$true)]
    [string]$DeviceId,

    [Parameter(Mandatory=$true)]
    [string]$DeviceName,

    [string]$DroneHost = "localhost",
    [int]$DronePort = 8181
)

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "🚀 Starting Drone Mesh Client (Python)" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# Check for virtual environment
if (-not (Test-Path "..\.venv\Scripts\python.exe")) {
    Write-Host "❌ Error: Virtual environment not found at ..\.venv" -ForegroundColor Red
    Write-Host "Please ensure you are running this from the scripts/ directory."
    exit 1
}

# Run the device client
$Python = "..\.venv\Scripts\python.exe"
& $Python ..\device_client.py --id $DeviceId --name $DeviceName --host $DroneHost --port $DronePort
