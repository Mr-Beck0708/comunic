# Drone Mesh: Permanent Environment Fix Script
# Run this script if you ever have missing dependencies or crypto errors.

Write-Host "🚀 Starting Permanent Fix for Drone Mesh Environment..." -ForegroundColor Cyan

# 1. Ensure Virtual Environment exists
if (-not (Test-Path ".venv")) {
    Write-Host "📦 Creating virtual environment..."
    python -m venv .venv
}

$PYTHON = ".\.venv\Scripts\python.exe"
$PIP = ".\.venv\Scripts\pip.exe"
$MATURIN = ".\.venv\Scripts\maturin.exe"

# 2. Upgrade Pip
Write-Host "🆙 Upgrading pip..."
& $PYTHON -m pip install --upgrade pip

# 3. Install Python Dependencies
Write-Host "📚 Installing Python dependencies..."
& $PIP install -r requirements.txt

# 4. Build and Install Rust Crypto Module
Write-Host "🦀 Building Post-Quantum Rust Module..."
if (Test-Path $MATURIN) {
    & $MATURIN develop --features python
} else {
    & $PIP install maturin
    & $MATURIN develop --features python
}

# 5. Final Verification
Write-Host "🔍 Verifying installation..."
$test = & $PYTHON -c "import websockets, zeroconf, aiohttp, drone_crypto; print('SUCCESS')"
if ($test -eq "SUCCESS") {
    Write-Host "✅ ENVIRONMENT RECOVERY COMPLETE!" -ForegroundColor Green
    Write-Host "💡 Note: Please restart your IDE to pick up the changes."
} else {
    Write-Host "❌ Verification failed. Please check the errors above." -ForegroundColor Red
}
