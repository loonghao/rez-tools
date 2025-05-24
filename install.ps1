# rez-tools installation script for Windows

param(
    [string]$InstallDir = "$env:USERPROFILE\.local\bin"
)

# Configuration
$Repo = "loonghao/rez-tools"
$BinaryName = "rt.exe"

# Colors for output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    
    $colorMap = @{
        "Red" = "Red"
        "Green" = "Green" 
        "Yellow" = "Yellow"
        "Blue" = "Cyan"
        "White" = "White"
    }
    
    Write-Host $Message -ForegroundColor $colorMap[$Color]
}

# Detect platform
function Get-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    
    switch ($arch) {
        "AMD64" { return "windows-x86_64" }
        "x86" { return "windows-x86" }
        default {
            Write-ColorOutput "Error: Unsupported architecture: $arch" "Red"
            exit 1
        }
    }
}

# Get latest release info
function Get-LatestRelease {
    $apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
    
    try {
        $response = Invoke-RestMethod -Uri $apiUrl -Method Get
        return $response
    }
    catch {
        Write-ColorOutput "Error: Failed to fetch release information: $_" "Red"
        exit 1
    }
}

# Download and install binary
function Install-Binary {
    param([string]$Platform)
    
    $assetName = "rt-$Platform.zip"
    
    Write-ColorOutput "Detecting platform: $Platform" "Blue"
    
    # Get release info
    Write-ColorOutput "Fetching latest release information..." "Blue"
    $releaseInfo = Get-LatestRelease
    
    # Find download URL
    $asset = $releaseInfo.assets | Where-Object { $_.name -eq $assetName }
    if (-not $asset) {
        Write-ColorOutput "Error: Could not find download URL for $assetName" "Red"
        Write-ColorOutput "Available assets:" "Yellow"
        $releaseInfo.assets | ForEach-Object { Write-Host "  - $($_.name)" }
        exit 1
    }
    
    $downloadUrl = $asset.browser_download_url
    
    # Create install directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    
    # Download binary
    Write-ColorOutput "Downloading $assetName..." "Blue"
    $tempFile = [System.IO.Path]::GetTempFileName() + ".zip"
    
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile
    }
    catch {
        Write-ColorOutput "Error: Failed to download binary: $_" "Red"
        exit 1
    }
    
    # Extract binary
    Write-ColorOutput "Extracting binary..." "Blue"
    try {
        Expand-Archive -Path $tempFile -DestinationPath $InstallDir -Force
        Remove-Item $tempFile
    }
    catch {
        Write-ColorOutput "Error: Failed to extract binary: $_" "Red"
        exit 1
    }
    
    Write-ColorOutput "✅ Successfully installed rez-tools to $InstallDir\$BinaryName" "Green"
}

# Check if binary is in PATH
function Test-PathConfiguration {
    $currentPath = $env:PATH
    if ($currentPath -notlike "*$InstallDir*") {
        Write-ColorOutput "Warning: $InstallDir is not in your PATH" "Yellow"
        Write-ColorOutput "Add the following directory to your PATH:" "Yellow"
        Write-ColorOutput "$InstallDir" "Blue"
        Write-Host ""
        Write-ColorOutput "You can do this by running:" "Yellow"
        Write-ColorOutput "`$env:PATH += `";$InstallDir`"" "Blue"
        Write-ColorOutput "Or add it permanently through System Properties > Environment Variables" "Yellow"
        Write-Host ""
    }
}

# Test installation
function Test-Installation {
    Write-ColorOutput "Testing installation..." "Blue"
    
    $binaryPath = Join-Path $InstallDir $BinaryName
    
    try {
        $result = & $binaryPath --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput "✅ Installation test passed" "Green"
            Write-ColorOutput "Run '$BinaryName --help' to get started" "Blue"
        }
        else {
            Write-ColorOutput "❌ Installation test failed" "Red"
            exit 1
        }
    }
    catch {
        Write-ColorOutput "❌ Installation test failed: $_" "Red"
        exit 1
    }
}

# Main installation process
function Main {
    Write-ColorOutput "rez-tools Installation Script" "Blue"
    Write-Host "=============================="
    Write-Host ""
    
    # Detect platform and install
    $platform = Get-Platform
    Install-Binary -Platform $platform
    
    # Check PATH and test
    Test-PathConfiguration
    Test-Installation
    
    Write-Host ""
    Write-ColorOutput "🎉 rez-tools has been successfully installed!" "Green"
    Write-Host ""
    Write-ColorOutput "Next steps:" "Blue"
    Write-Host "1. Add $InstallDir to your PATH if not already done"
    Write-Host "2. Run 'rt check-rez' to verify your rez environment"
    Write-Host "3. Run 'rt install-rez' if rez is not installed"
    Write-Host "4. Run 'rt list' to see available tools"
}

# Run main function
Main
