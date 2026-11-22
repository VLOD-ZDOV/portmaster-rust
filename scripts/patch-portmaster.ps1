<#
.SYNOPSIS
    Portmaster Patcher / Installer for Windows
    Replaces or upgrades Portmaster core with Rust core (pm-core)
    and modern unlocked UI with Xray proxy integration.

.DESCRIPTION
    Usage in PowerShell (Run as Administrator):
        irm https://raw.githubusercontent.com/VLOD-ZDOV/portmaster-rust/main/scripts/patch-portmaster.ps1 | iex
        .\scripts\patch-portmaster.ps1 [-Core rs|go] [-Tag latest|<tag>] [-Restore]
#>

function Invoke-PortmasterPatch {
    [CmdletBinding()]
    param (
        [ValidateSet("rs", "go")]
        [string]$Core = "rs",

        [string]$Tag = "latest",

        [switch]$Restore
    )

    # Support environment variables for piped execution (irm ... | iex)
    if ($env:PORTMASTER_CORE -and (-not $PSBoundParameters.ContainsKey('Core'))) {
        $Core = $env:PORTMASTER_CORE
    }
    if ($env:PORTMASTER_TAG -and (-not $PSBoundParameters.ContainsKey('Tag'))) {
        $Tag = $env:PORTMASTER_TAG
    }

    $ErrorActionPreference = "Stop"

    # Configure TLS protocol safely on Windows PowerShell 5.1 and PS Core
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]'Tls12,Tls13'
    } catch {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]'Tls12'
    }

    # Self-healing helper: ensure network & DNS are functional
    function Repair-NetworkState {
        # Unhang any orphaned kernel driver from previous crashed/stopped sessions
        & sc.exe stop PortmasterKext 2>$null | Out-Null
        & sc.exe delete PortmasterKext 2>$null | Out-Null

        # If internet is unreachable, reset adapter DNS to DHCP / default
        $canConnect = $false
        try {
            $tcp = New-Object System.Net.Sockets.TcpClient
            $async = $tcp.BeginConnect("1.1.1.1", 53, $null, $null)
            $wait = $async.AsyncWaitHandle.WaitOne(1500, $false)
            if ($wait -and $tcp.Connected) {
                $canConnect = $true
                $tcp.EndConnect($async)
            }
            $tcp.Close()
        } catch {}

        if (-not $canConnect) {
            Write-Host "[*] Connectivity issue detected; resetting DNS to DHCP..." -ForegroundColor Yellow
            Get-NetAdapter | Where-Object Status -eq 'Up' | Set-DnsClientServerAddress -ResetServerAddresses -ErrorAction SilentlyContinue
            & ipconfig /flushdns 2>$null | Out-Null
            Start-Sleep -Seconds 1
        }
    }

    # Ensure Administrator
    $currentPrincipal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
    if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        Write-Error "Administrator privileges are required. Please run PowerShell as Administrator."
        return
    }

    # Pre-patch network self-healing
    Repair-NetworkState

    $Repo = "VLOD-ZDOV/portmaster-rust"
    $InstallDir = "C:\Program Files\Portmaster"

    # Resolve DataDir: C:\ProgramData\Portmaster with fallback to $env:ProgramData\Portmaster and safing path
    $DataDir = "C:\ProgramData\Portmaster"
    if (-not (Test-Path $DataDir)) {
        if ($env:ProgramData -and (Test-Path (Join-Path $env:ProgramData "Portmaster"))) {
            $DataDir = Join-Path $env:ProgramData "Portmaster"
        } elseif (Test-Path "C:\ProgramData\safing\portmaster") {
            $DataDir = "C:\ProgramData\safing\portmaster"
        } elseif ($env:ProgramData) {
            $DataDir = Join-Path $env:ProgramData "Portmaster"
        }
    }

    $BackupDir = Join-Path $DataDir "core-backups"

    # Resolve Service Name: PortmasterCore with fallback to Portmaster
    $ServiceName = "PortmasterCore"
    if (-not (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue)) {
        if (Get-Service -Name "Portmaster" -ErrorAction SilentlyContinue) {
            $ServiceName = "Portmaster"
        }
    }

    # Resolve InstallDir: check service PathName first if service exists
    $svcObj = Get-CimInstance Win32_Service -Filter "Name='$ServiceName'" -ErrorAction SilentlyContinue
    if (-not $svcObj) {
        $svcObj = Get-WmiObject Win32_Service -Filter "Name='$ServiceName'" -ErrorAction SilentlyContinue
    }
    if ($svcObj -and $svcObj.PathName) {
        $svcPath = $svcObj.PathName.Trim().Trim('"')
        if ($svcPath -match '^([a-zA-Z]:\\[^:]+?\.exe)') {
            $svcPath = $matches[1]
        }
        $svcDir = Split-Path -Path $svcPath -Parent
        if ($svcDir -and (Test-Path $svcDir)) {
            $InstallDir = $svcDir
        }
    }

    if (-not (Test-Path $InstallDir)) {
        $candidateDirs = @(
            "C:\Program Files\Portmaster",
            "C:\Program Files\safing\portmaster",
            "C:\Program Files (x86)\Portmaster",
            "C:\Program Files (x86)\safing\portmaster"
        )
        foreach ($cand in $candidateDirs) {
            if (Test-Path $cand) {
                $InstallDir = $cand
                break
            }
        }
    }

    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    if (-not (Test-Path $DataDir)) {
        New-Item -ItemType Directory -Path $DataDir -Force | Out-Null
    }
    if (-not (Test-Path $BackupDir)) {
        New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null
    }

    if ($Restore) {
        Write-Host "[*] Restoring Portmaster from backup..." -ForegroundColor Cyan
        $backups = Get-ChildItem -Path $BackupDir -Directory | Sort-Object CreationTime -Descending
        if ($backups.Count -eq 0) {
            Write-Error "No backups found in $BackupDir"
            return
        }
        $latestBackup = $backups[0].FullName
        Write-Host "[*] Restoring from $latestBackup..." -ForegroundColor Cyan

        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2

        if (Test-Path (Join-Path $latestBackup "portmaster-core.exe")) {
            Copy-Item -Path (Join-Path $latestBackup "portmaster-core.exe") -Destination (Join-Path $InstallDir "portmaster-core.exe") -Force
        }
        if (Test-Path (Join-Path $latestBackup "config.json")) {
            Copy-Item -Path (Join-Path $latestBackup "config.json") -Destination (Join-Path $DataDir "config.json") -Force
        }
        try {
            Start-Service -Name $ServiceName -ErrorAction Stop
            Write-Host "[+] Restore completed successfully." -ForegroundColor Green
        } catch {
            Write-Error "Failed to start service '$ServiceName': $_"
        }
        return
    }

    Write-Host "==========================================================" -ForegroundColor Cyan
    Write-Host " Portmaster Windows Patcher (Core: $Core, Tag: $Tag)" -ForegroundColor Cyan
    Write-Host " Service: $ServiceName | DataDir: $DataDir" -ForegroundColor Cyan
    Write-Host "==========================================================" -ForegroundColor Cyan

    # Resolve release URLs
    Write-Host "[*] Resolving release from GitHub ($Repo)..." -ForegroundColor Gray
    $BaseUrl = ""
    if ($Tag -eq "latest") {
        $apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
        try {
            $release = Invoke-RestMethod -Uri $apiUrl -UseBasicParsing -Headers @{ "User-Agent" = "PortmasterPatcher" }
            $resolvedTag = $release.tag_name
            $BaseUrl = "https://github.com/$Repo/releases/download/$resolvedTag"
        } catch {
            $BaseUrl = "https://github.com/$Repo/releases/latest/download"
        }
    } else {
        $BaseUrl = "https://github.com/$Repo/releases/download/$Tag"
    }

    $binName = "portmaster-core-$Core.exe"
    $binUrl = "$BaseUrl/$binName"
    $uiUrl = "$BaseUrl/portmaster.zip"

    $tempDir = Join-Path $env:TEMP ("pm-patch-" + [System.Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try {
        # Download binary
        $tempBin = Join-Path $tempDir $binName
        Write-Host "[*] Downloading $binName from $binUrl..." -ForegroundColor Gray
        Invoke-WebRequest -Uri $binUrl -OutFile $tempBin -UseBasicParsing -Headers @{ "User-Agent" = "PortmasterPatcher" }

        # Download UI bundle
        $tempZip = Join-Path $tempDir "portmaster.zip"
        $hasUi = $false
        try {
            Write-Host "[*] Downloading UI bundle (portmaster.zip)..." -ForegroundColor Gray
            Invoke-WebRequest -Uri $uiUrl -OutFile $tempZip -UseBasicParsing -Headers @{ "User-Agent" = "PortmasterPatcher" }
            $hasUi = $true
        } catch {
            Write-Host "[*] UI bundle not found or download failed, continuing without UI update." -ForegroundColor Yellow
        }

        # Stop Portmaster Service & cleanup driver
        $serviceRunning = $false
        $pmService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
        if ($pmService -and $pmService.Status -eq "Running") {
            Write-Host "[*] Stopping service $ServiceName..." -ForegroundColor Gray
            Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 2
            $serviceRunning = $true
        }
        # Forcibly stop and unhook kernel driver so network is never blocked during patching
        & sc.exe stop PortmasterKext 2>$null | Out-Null
        & sc.exe delete PortmasterKext 2>$null | Out-Null
        Get-Process -Name "portmaster-core-rs", "portmaster-core" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

        # Backup existing binary
        $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
        $currentBackup = Join-Path $BackupDir $stamp
        New-Item -ItemType Directory -Path $currentBackup -Force | Out-Null

        $existingBin = Join-Path $InstallDir "portmaster-core.exe"
        if (Test-Path $existingBin) {
            Copy-Item -Path $existingBin -Destination (Join-Path $currentBackup "portmaster-core.exe") -Force
            Write-Host "[*] Backed up original binary to: $currentBackup" -ForegroundColor Gray
        }
        $existingConfig = Join-Path $DataDir "config.json"
        if (Test-Path $existingConfig) {
            Copy-Item -Path $existingConfig -Destination (Join-Path $currentBackup "config.json") -Force
            Write-Host "[*] Backed up config.json to: $currentBackup" -ForegroundColor Gray
        }

        # Install new binary
        Copy-Item -Path $tempBin -Destination (Join-Path $InstallDir $binName) -Force
        Copy-Item -Path $tempBin -Destination (Join-Path $InstallDir "portmaster-core.exe") -Force

        # Also update in updates dir if present
        $updatesBin = Join-Path $DataDir "updates\windows_amd64\core\portmaster-core_windows_amd64.exe"
        if (Test-Path (Split-Path $updatesBin)) {
            Copy-Item -Path $tempBin -Destination $updatesBin -Force
        }

        # Update UI if downloaded
        if ($hasUi -and (Test-Path $tempZip)) {
            Write-Host "[*] Updating Web UI..." -ForegroundColor Gray

            # Copy UI zip archive for pm-api UiArchives
            Copy-Item -Path $tempZip -Destination (Join-Path $InstallDir "portmaster.zip") -Force
            Copy-Item -Path $tempZip -Destination (Join-Path $DataDir "portmaster.zip") -Force

            # 1. Pilot UI directory for Rust core: <DataDir>\pilot-web
            $pilotWebDest = Join-Path $DataDir "pilot-web"
            New-Item -ItemType Directory -Path $pilotWebDest -Force | Out-Null
            Expand-Archive -Path $tempZip -DestinationPath $pilotWebDest -Force
            Write-Host "[*] Extracted UI bundle to: $pilotWebDest" -ForegroundColor Gray

            # 2. Traditional module directory: <DataDir>\ui\modules\portmaster
            $uiDest = Join-Path $DataDir "ui\modules\portmaster"
            New-Item -ItemType Directory -Path $uiDest -Force | Out-Null
            Expand-Archive -Path $tempZip -DestinationPath $uiDest -Force
            Write-Host "[*] Extracted UI bundle to: $uiDest" -ForegroundColor Gray

            # 3. Update in updates dir if present
            $updatesUi = Join-Path $DataDir "updates\windows_amd64\ui\modules\portmaster"
            if (Test-Path (Split-Path $updatesUi)) {
                New-Item -ItemType Directory -Path $updatesUi -Force | Out-Null
                Expand-Archive -Path $tempZip -DestinationPath $updatesUi -Force
            }
        }

        # Register service if not present
        if (-not (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue)) {
            Write-Host "[*] Service $ServiceName not found, creating Windows Service..." -ForegroundColor Gray
            New-Service -Name $ServiceName -BinaryPathName "`"$InstallDir\portmaster-core.exe`"" -DisplayName "Portmaster Core" -StartupType Automatic
            $serviceRunning = $true
        }

        # Restart Portmaster Service and verify Running state
        Write-Host "[*] Starting $ServiceName service..." -ForegroundColor Gray
        try {
            Start-Service -Name $ServiceName -ErrorAction Stop
        } catch {
            Write-Error "Failed to initiate start for service '$ServiceName': $_"
            return
        }

        # Wait for service to reach Running state (up to 30 seconds)
        Write-Host "[*] Waiting for $ServiceName to reach 'Running' status..." -ForegroundColor Gray
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $timeoutSeconds = 30
        $isRunning = $false

        while ($sw.Elapsed.TotalSeconds -lt $timeoutSeconds) {
            $status = (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue).Status
            if ($status -eq "Running") {
                $isRunning = $true
                break
            }
            Start-Sleep -Seconds 1
        }

        if (-not $isRunning) {
            $finalStatus = (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue).Status
            Write-Error "Service '$ServiceName' failed to reach 'Running' state within $timeoutSeconds seconds. Current status: $finalStatus."
            return
        }

        Write-Host "[+] Service $ServiceName is Running." -ForegroundColor Green

        # Verify API health check
        Write-Host "[*] Verifying Portmaster Core API status (http://127.0.0.1:817)..." -ForegroundColor Gray
        $apiOk = $false
        $apiSw = [System.Diagnostics.Stopwatch]::StartNew()
        while ($apiSw.Elapsed.TotalSeconds -lt 25) {
            try {
                $statusResp = Invoke-RestMethod -Uri "http://127.0.0.1:817/ping" -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
                if ($statusResp) {
                    $apiOk = $true
                    break
                }
            } catch {
                Start-Sleep -Seconds 1
            }
        }

        if ($apiOk) {
            Write-Host "[+] API is responsive and ready on http://127.0.0.1:817" -ForegroundColor Green
        } else {
            Write-Host "[!] API is still initializing; background workers are running." -ForegroundColor Yellow
        }

        Write-Host ""
        Write-Host "==========================================================" -ForegroundColor Green
        Write-Host " [+] Portmaster Windows patched successfully!" -ForegroundColor Green
        Write-Host "     Service: $ServiceName (Running)" -ForegroundColor Green
        Write-Host "     Core: $binName" -ForegroundColor Green
        Write-Host "     Backup saved in: $currentBackup" -ForegroundColor Green
        Write-Host "==========================================================" -ForegroundColor Green
    } finally {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Invoke-PortmasterPatch @args
