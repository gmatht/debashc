# PowerShell script to build and run WASM project
# This script builds the WASM target and opens it in Edge or default browser

param(
    [string]$Browser = "edge",  # Can be "edge", "default", or "chrome"
    [int]$Port = (Get-Random -Minimum 8000 -Maximum 65535)  # Random port between 8000 and 65535
)

Write-Host "=== WASM Build and Run Script ===" -ForegroundColor Green
Write-Host ""

# Function to check if a command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Function to check if a port is available
function Test-Port($Port) {
    try {
        $listener = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Loopback, $Port)
        $listener.Start()
        $listener.Stop()
        return $true
    }
    catch {
        return $false
    }
}

# Function to check if wasm-pack is installed
function Test-WasmPack {
    if (Test-Command "wasm-pack") {
        return $true
    }
    return $false
}

# Function to install wasm-pack
function Install-WasmPack {
    Write-Host "Installing wasm-pack..." -ForegroundColor Yellow
    try {
        cargo install wasm-pack
        Write-Host "wasm-pack installed successfully!" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "Failed to install wasm-pack: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to build WASM target
function Build-Wasm {
    Write-Host "Building WASM target..." -ForegroundColor Yellow
    
    # Create www directory if it doesn't exist
    if (!(Test-Path "www")) {
        New-Item -ItemType Directory -Path "www" | Out-Null
        Write-Host "Created www directory" -ForegroundColor Green
    }
    
    try {
        wasm-pack build --target web --out-dir www/pkg
        Write-Host "WASM build completed successfully!" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "WASM build failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to start HTTP server
function Start-HttpServer {
    param([int]$Port)
    
    # Try to find an available port starting from the given one
    $maxTries = 10
    $currentPort = $Port
    $portFound = $false
    
    for ($i = 0; $i -lt $maxTries; $i++) {
        if (Test-Port $currentPort) {
            $portFound = $true
            break
        }
        Write-Host "Port $currentPort is in use, trying next port..." -ForegroundColor Yellow
        $currentPort = Get-Random -Minimum 8000 -Maximum 65535
    }
    
    if (-not $portFound) {
        Write-Host "Could not find an available port after $maxTries attempts." -ForegroundColor Red
        return $false
    }
    
    Write-Host "Starting HTTP server on port $currentPort..." -ForegroundColor Yellow
    
    # Check if Python is available
    if (Test-Command "python") {
        $pythonCmd = "python"
    }
    elseif (Test-Command "python3") {
        $pythonCmd = "python3"
    }
    else {
        Write-Host "Python not found. Please install Python to run the HTTP server." -ForegroundColor Red
        return $false
    }
    
    try {
        # Start the server in the background
        $serverProcess = Start-Process -FilePath $pythonCmd -ArgumentList "-m", "http.server", $currentPort -WorkingDirectory "www" -PassThru -WindowStyle Hidden
        Write-Host "HTTP server started (PID: $($serverProcess.Id))" -ForegroundColor Green
        
        # Wait a moment for server to start
        Start-Sleep -Seconds 2
        
        # Update the script scope port variable
        $script:Port = $currentPort
        return $true
    }
    catch {
        Write-Host "Failed to start HTTP server: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to open browser
function Open-Browser {
    param([string]$Browser, [int]$Port)
    
    $url = "http://localhost:$Port"
    Write-Host "Opening $Browser at $url..." -ForegroundColor Yellow
    
    try {
        switch ($Browser.ToLower()) {
            "edge" {
                Start-Process "msedge" -ArgumentList $url
            }
            "chrome" {
                Start-Process "chrome" -ArgumentList $url
            }
            "default" {
                Start-Process $url
            }
            default {
                Start-Process $url
            }
        }
        Write-Host "Browser opened successfully!" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "Failed to open browser: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Main execution
Write-Host "Checking prerequisites..." -ForegroundColor Cyan

# Check if wasm-pack is installed
if (!(Test-WasmPack)) {
    Write-Host "wasm-pack not found. Installing..." -ForegroundColor Yellow
    if (!(Install-WasmPack)) {
        Write-Host "Failed to install wasm-pack. Exiting." -ForegroundColor Red
        exit 1
    }
}
else {
    Write-Host "wasm-pack is already installed" -ForegroundColor Green
}

# Build WASM target
if (!(Build-Wasm)) {
    Write-Host "WASM build failed. Exiting." -ForegroundColor Red
    exit 1
}

# Start HTTP server
if (!(Start-HttpServer -Port $Port)) {
    Write-Host "Failed to start HTTP server. Exiting." -ForegroundColor Red
    exit 1
}

# Open browser
if (!(Open-Browser -Browser $Browser -Port $Port)) {
    Write-Host "Failed to open browser." -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Setup Complete ===" -ForegroundColor Green
Write-Host "WASM application is running at: http://localhost:$Port" -ForegroundColor Cyan
Write-Host "Press Ctrl+C to stop the server when done" -ForegroundColor Yellow
Write-Host ""

# Keep the script running to maintain the server
try {
    while ($true) {
        Start-Sleep -Seconds 1
    }
}
catch {
    Write-Host ""
    Write-Host "Server stopped." -ForegroundColor Yellow
}
