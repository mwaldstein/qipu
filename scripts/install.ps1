# Qipu installer script for Windows
# Usage: irm https://raw.githubusercontent.com/mwaldstein/qipu/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

# Configuration
$Repo = "mwaldstein/qipu"
$BinaryName = "qipu"
$InstallDir = if ($env:QIPU_INSTALL_DIR) { $env:QIPU_INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }

# Function to write colored output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

# Detect platform
function Get-Platform {
    $arch = [System.Environment]::Is64BitOperatingSystem
    
    if (-not $arch) {
        Write-ColorOutput "Error: Only 64-bit Windows is supported" "Red"
        exit 1
    }
    
    $target = "x86_64-pc-windows-msvc"
    Write-ColorOutput "Detected platform: $target" "Green"
    return $target
}

# Get latest release version
function Get-LatestVersion {
    Write-Host "Fetching latest release version..."
    
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        $version = $response.tag_name -replace '^v', ''
        
        if (-not $version) {
            Write-ColorOutput "Error: Could not determine latest version" "Red"
            exit 1
        }
        
        Write-ColorOutput "Latest version: v$version" "Green"
        return $version
    }
    catch {
        Write-ColorOutput "Error: Failed to fetch latest version: $_" "Red"
        exit 1
    }
}

# Download and verify binary
function Install-Binary {
    param(
        [string]$Version,
        [string]$Target
    )
    
    $filename = "$BinaryName-$Version-$Target.zip"
    $url = "https://github.com/$Repo/releases/download/v$Version/$filename"
    $checksumUrl = "$url.sha256"
    
    Write-Host "Downloading $filename..."
    
    # Create temporary directory
    $tmpDir = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ([System.IO.Path]::GetRandomFileName()))
    
    try {
        $zipPath = Join-Path $tmpDir $filename
        $checksumPath = "$zipPath.sha256"
        
        # Download binary
        try {
            Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        }
        catch {
            Write-ColorOutput "Error: Failed to download binary: $_" "Red"
            exit 1
        }
        
        # Download and verify checksum
        try {
            Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumPath -UseBasicParsing
            
            Write-Host "Verifying checksum..."
            $expectedHash = (Get-Content $checksumPath -Raw).Trim().Split()[0]
            $actualHash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash
            
            if ($expectedHash.ToLower() -ne $actualHash.ToLower()) {
                Write-ColorOutput "Error: Checksum verification failed" "Red"
                Write-Host "Expected: $expectedHash"
                Write-Host "Actual:   $actualHash"
                exit 1
            }
            Write-ColorOutput "Checksum verified" "Green"
        }
        catch {
            Write-ColorOutput "Warning: Could not verify checksum: $_" "Yellow"
        }
        
        # Extract binary
        Write-Host "Extracting binary..."
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force
        
        # Create install directory if it doesn't exist
        Write-Host "Installing to $InstallDir..."
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        
        # Move binary
        $binaryPath = Join-Path $tmpDir "$BinaryName.exe"
        $targetPath = Join-Path $InstallDir "$BinaryName.exe"
        
        # Remove existing binary if present
        if (Test-Path $targetPath) {
            Remove-Item $targetPath -Force
        }
        
        Move-Item -Path $binaryPath -Destination $targetPath -Force
        
        Write-ColorOutput "Successfully installed $BinaryName to $InstallDir" "Green"
    }
    finally {
        # Clean up temporary directory
        Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Check if binary is in PATH
function Test-Path-Configuration {
    $pathEntries = $env:PATH -split ';'
    
    if ($pathEntries -notcontains $InstallDir) {
        Write-Host ""
        Write-ColorOutput "Warning: $InstallDir is not in your PATH" "Yellow"
        Write-Host "To add it permanently, run:"
        Write-Host ""
        Write-Host "  `$env:PATH = `"`$env:PATH;$InstallDir`""
        Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `$env:PATH, 'User')"
        Write-Host ""
        
        # Add to current session PATH
        $env:PATH = "$env:PATH;$InstallDir"
        Write-ColorOutput "Added to current session PATH" "Green"
    }
    else {
        Write-Host ""
        Write-ColorOutput "$BinaryName is ready to use!" "Green"
        Write-Host "Run '$BinaryName --help' to get started."
    }
}

# Verify installation
function Test-Installation {
    $binaryPath = Join-Path $InstallDir "$BinaryName.exe"
    
    if (Test-Path $binaryPath) {
        try {
            $version = & $binaryPath --version 2>$null | Select-Object -First 1
            Write-Host ""
            Write-ColorOutput "Installation verified: $version" "Green"
        }
        catch {
            Write-ColorOutput "Warning: Could not verify installation: $_" "Yellow"
        }
    }
    else {
        Write-ColorOutput "Error: Installation verification failed" "Red"
        exit 1
    }
}

# Main installation flow
function Main {
    Write-Host "Qipu Installer"
    Write-Host "=============="
    Write-Host ""
    
    $target = Get-Platform
    $version = Get-LatestVersion
    Install-Binary -Version $version -Target $target
    Test-Installation
    Test-Path-Configuration
}

Main
