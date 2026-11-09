# Clack Windows PowerShell Installer
# Usage:
#   irm https://raw.githubusercontent.com/ashrafpy/clack/main/install.ps1 | iex
# Or from Command Prompt (cmd.exe):
#   curl.exe -fsSL https://raw.githubusercontent.com/ashrafpy/clack/main/install.ps1 | powershell -NoProfile -Command -

$ErrorActionPreference = "Stop"

function Write-Info {
    param([string]$Message)
    Write-Host "==> " -ForegroundColor Blue -NoNewline
    Write-Host $Message -ForegroundColor White
}

function Write-Success {
    param([string]$Message)
    Write-Host "✔ " -ForegroundColor Green -NoNewline
    Write-Host $Message -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠ " -ForegroundColor Yellow -NoNewline
    Write-Host $Message -ForegroundColor Yellow
}

function Write-Failure {
    param([string]$Message)
    Write-Host "✘ Error: " -ForegroundColor Red -NoNewline
    Write-Host $Message -ForegroundColor Red
}

try {
    # Ensure TLS 1.2 is enabled for legacy PowerShell 5.1 environments
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    # Architecture detection
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($env:PROCESSOR_ARCHITEW6432) {
        $arch = $env:PROCESSOR_ARCHITEW6432
    }

    switch ($arch) {
        "AMD64" {
            $target = "x86_64-pc-windows-msvc"
        }
        default {
            throw "Unsupported Windows architecture '$arch'. Clack currently supports x86_64 (64-bit Intel/AMD)."
        }
    }

    $repo = "ashrafpy/clack"
    $zipName = "clack-$target.zip"
    $releaseUrl = "https://github.com/$repo/releases/latest/download/$zipName"

    # Setup temporary directory
    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("clack-install-" + [System.Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try {
        $zipPath = Join-Path $tempDir $zipName
        Write-Info "Downloading Clack for $target..."

        $webClient = New-Object System.Net.WebClient
        try {
            $webClient.Headers.Add("User-Agent", "clack-installer-powershell")
            $webClient.DownloadFile($releaseUrl, $zipPath)
        } catch {
            # Fallback to Invoke-WebRequest if WebClient encounters an issue
            Invoke-WebRequest -Uri $releaseUrl -OutFile $zipPath -UseBasicParsing
        }

        Write-Info "Extracting archive..."
        $extractDir = Join-Path $tempDir "extracted"
        Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force

        # Locate clack.exe
        $sourceExe = Get-ChildItem -Path $extractDir -Filter "clack.exe" -Recurse | Select-Object -First 1
        if (-not $sourceExe -or -not (Test-Path $sourceExe.FullName)) {
            throw "Failed to locate clack.exe in downloaded archive."
        }

        # Target installation directory: $env:USERPROFILE\.local\bin
        $installDir = Join-Path $env:USERPROFILE ".local\bin"
        if (-not (Test-Path $installDir)) {
            New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        }

        $destExe = Join-Path $installDir "clack.exe"
        Copy-Item -Path $sourceExe.FullName -Destination $destExe -Force
        Write-Success "Installed clack.exe to $destExe"

        # Check and update User PATH
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        $pathParts = if ([string]::IsNullOrWhiteSpace($userPath)) { @() } else { $userPath -split ';' }
        $alreadyInPath = $false
        foreach ($part in $pathParts) {
            if ($part.TrimEnd('\') -ieq $installDir.TrimEnd('\')) {
                $alreadyInPath = $true
                break
            }
        }

        if (-not $alreadyInPath) {
            $newUserPath = if ([string]::IsNullOrWhiteSpace($userPath)) {
                $installDir
            } else {
                "$userPath;$installDir"
            }
            [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
            Write-Success "Added $installDir to User PATH."
        }

        # Update PATH in the current process session
        $procPath = $env:PATH
        $procPathParts = if ([string]::IsNullOrWhiteSpace($procPath)) { @() } else { $procPath -split ';' }
        $alreadyInProcPath = $false
        foreach ($part in $procPathParts) {
            if ($part.TrimEnd('\') -ieq $installDir.TrimEnd('\')) {
                $alreadyInProcPath = $true
                break
            }
        }
        if (-not $alreadyInProcPath) {
            $env:PATH = "$installDir;$env:PATH"
        }

        Write-Host ""
        Write-Host "✨ Clack was successfully installed! ✨" -ForegroundColor Green
        Write-Host ""
        Write-Host "Quick start:"
        Write-Host "  clack               " -ForegroundColor White -NoNewline; Write-Host "# Run with default settings" -ForegroundColor Gray
        Write-Host "  clack --volume 0.5  " -ForegroundColor White -NoNewline; Write-Host "# Set volume to 50%" -ForegroundColor Gray
        Write-Host "  clack --help        " -ForegroundColor White -NoNewline; Write-Host "# See all available options" -ForegroundColor Gray
        Write-Host ""
    } finally {
        if (Test-Path $tempDir) {
            Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
} catch {
    Write-Failure $_.Exception.Message
    exit 1
}
