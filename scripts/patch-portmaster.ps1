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
        if (Test-Path (Join-Path $latestBackup "portmaster.exe")) {
            Get-Process -Name "*portmaster*" -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -notlike "*core*" } | Stop-Process -Force -ErrorAction SilentlyContinue
            Copy-Item -Path (Join-Path $latestBackup "portmaster.exe") -Destination (Join-Path $InstallDir "portmaster.exe") -Force
        }
        if (Test-Path (Join-Path $latestBackup "portmaster.zip")) {
            Copy-Item -Path (Join-Path $latestBackup "portmaster.zip") -Destination (Join-Path $InstallDir "portmaster.zip") -Force
            Copy-Item -Path (Join-Path $latestBackup "portmaster.zip") -Destination (Join-Path $DataDir "portmaster.zip") -Force
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
    $pilotUrl = "$BaseUrl/pilot-web.zip"
    $appUrl = "$BaseUrl/portmaster-app-windows-amd64.exe"

    function Invoke-DownloadWithRetry {
        param(
            [string]$Uri,
            [string]$OutFile,
            [string]$Label,
            [bool]$Required = $true
        )
        for ($attempt = 1; $attempt -le 3; $attempt++) {
            try {
                Write-Host "[*] Downloading $Label ($attempt/3) from $Uri..." -ForegroundColor Gray
                Invoke-WebRequest -Uri $Uri -OutFile $OutFile -UseBasicParsing -Headers @{ "User-Agent" = "PortmasterPatcher" }
                if ((Test-Path $OutFile) -and ((Get-Item $OutFile).Length -gt 0)) {
                    return $true
                }
            } catch {
                Write-Host "[!] Attempt $attempt failed for ${Label}: $($_.Exception.Message)" -ForegroundColor Yellow
                # On retry 1 failure, stop Portmaster firewall/kext so it cannot block GitHub downloads.
                $svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
                if ($svc -and $svc.Status -eq "Running") {
                    Write-Host "[*] Stopping $ServiceName and cleaning kext before retrying download..." -ForegroundColor Yellow
                    Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
                    Start-Sleep -Seconds 1
                    Repair-NetworkState
                }
                Start-Sleep -Seconds 2
            }
        }
        if ($Required) {
            throw "Failed to download required file '$Label' from '$Uri'."
        }
        return $false
    }

    $tempDir = Join-Path $env:TEMP ("pm-patch-" + [System.Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try {
        $tempBin = Join-Path $tempDir $binName
        $null = Invoke-DownloadWithRetry -Uri $binUrl -OutFile $tempBin -Label $binName -Required $true

        $tempApp = Join-Path $tempDir "portmaster.exe"
        $hasApp = Invoke-DownloadWithRetry -Uri $appUrl -OutFile $tempApp -Label "portmaster-app-windows-amd64.exe" -Required $true

        $tempZip = Join-Path $tempDir "portmaster.zip"
        $hasUi = Invoke-DownloadWithRetry -Uri $uiUrl -OutFile $tempZip -Label "portmaster.zip" -Required $true

        $tempPilotZip = Join-Path $tempDir "pilot-web.zip"
        $hasPilot = Invoke-DownloadWithRetry -Uri $pilotUrl -OutFile $tempPilotZip -Label "pilot-web.zip" -Required $false

        # Capture full exe paths of running Portmaster tray UI processes before stopping them
        $trayProcs = Get-Process -ErrorAction SilentlyContinue | Where-Object {
            ($_.ProcessName -ieq "portmaster" -or $_.ProcessName -ieq "portmaster-app" -or $_.ProcessName -ieq "portmaster_app")
        }
        $capturedTrayPaths = @()
        $appWasRunning = $false
        foreach ($tp in $trayProcs) {
            $appWasRunning = $true
            try {
                if ($tp.Path -and ($capturedTrayPaths -notcontains $tp.Path)) {
                    $capturedTrayPaths += $tp.Path
                }
            } catch {}
        }
        if ($appWasRunning) {
            Write-Host "[*] Stopping Portmaster tray UI processes ($($trayProcs.Count))..." -ForegroundColor Gray
            $trayProcs | Stop-Process -Force -ErrorAction SilentlyContinue
            # Also release WebView2 child file locks owned by Portmaster UI
            Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" -ErrorAction SilentlyContinue |
                Where-Object { $_.CommandLine -match 'io\.safing\.portmaster|Portmaster' } |
                ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
            Start-Sleep -Milliseconds 1200
        }
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

        # Backup existing binaries
        $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
        $currentBackup = Join-Path $BackupDir $stamp
        New-Item -ItemType Directory -Path $currentBackup -Force | Out-Null

        $existingBin = Join-Path $InstallDir "portmaster-core.exe"
        if (Test-Path $existingBin) {
            Copy-Item -Path $existingBin -Destination (Join-Path $currentBackup "portmaster-core.exe") -Force
            Write-Host "[*] Backed up original binary to: $currentBackup" -ForegroundColor Gray
        }
        $existingApp = Join-Path $InstallDir "portmaster.exe"
        if ($hasApp -and (Test-Path $existingApp)) {
            Copy-Item -Path $existingApp -Destination (Join-Path $currentBackup "portmaster.exe") -Force
        }
        $existingZip = Join-Path $InstallDir "portmaster.zip"
        if (Test-Path $existingZip) {
            Copy-Item -Path $existingZip -Destination (Join-Path $currentBackup "portmaster.zip") -Force
        }
        $existingConfig = Join-Path $DataDir "config.json"
        if (Test-Path $existingConfig) {
            Copy-Item -Path $existingConfig -Destination (Join-Path $currentBackup "config.json") -Force
            Write-Host "[*] Backed up config.json to: $currentBackup" -ForegroundColor Gray
        }

        # Install new core binaries
        Copy-Item -Path $tempBin -Destination (Join-Path $InstallDir $binName) -Force
        Copy-Item -Path $tempBin -Destination (Join-Path $InstallDir "portmaster-core.exe") -Force

        # Install new tray binary in InstallDir, all captured running paths, and standard candidate paths
        $trayDestinations = @((Join-Path $InstallDir "portmaster.exe"))
        foreach ($cp in $capturedTrayPaths) {
            if ($trayDestinations -notcontains $cp) {
                $trayDestinations += $cp
            }
        }
        $knownTrayPaths = @(
            "C:\Program Files\Portmaster\portmaster.exe",
            "C:\Program Files\Safing\Portmaster\portmaster.exe",
            (Join-Path $DataDir "updates\windows_amd64\app\portmaster-app.exe")
        )
        foreach ($kp in $knownTrayPaths) {
            if ((Test-Path $kp) -and ($trayDestinations -notcontains $kp)) {
                $trayDestinations += $kp
            }
        }
        foreach ($destExe in $trayDestinations) {
            Copy-Item -Path $tempApp -Destination $destExe -Force
            Write-Host "[*] Updated tray UI binary: $destExe" -ForegroundColor Gray
        }

        # Verify updated tray binary actually contains the new Xray VPN strings (not legacy SPN)
        $trayBytes = [System.IO.File]::ReadAllBytes((Join-Path $InstallDir "portmaster.exe"))
        $trayAscii = [System.Text.Encoding]::ASCII.GetString($trayBytes)
        $hasXrayTray = $trayAscii.Contains("Xray VPN")
        $hasLegacySpn = $trayAscii.Contains("Enable SPN")
        if (-not $hasXrayTray -or $hasLegacySpn) {
            Write-Warning "Installed tray binary verification unexpected (XrayVPN=$hasXrayTray, LegacyEnableSPN=$hasLegacySpn)."
        } else {
            Write-Host "[+] Verified tray binary contains Xray VPN menu (no Enable SPN)." -ForegroundColor Green
        }
        $trayHash = (Get-FileHash -Path (Join-Path $InstallDir "portmaster.exe") -Algorithm SHA256).Hash

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
            if ($hasPilot -and (Test-Path $tempPilotZip)) {
                $pilotWebDest = Join-Path $DataDir "pilot-web"
                New-Item -ItemType Directory -Path $pilotWebDest -Force | Out-Null
                Expand-Archive -Path $tempPilotZip -DestinationPath $pilotWebDest -Force
                Write-Host "[*] Extracted pilot UI bundle to: $pilotWebDest" -ForegroundColor Gray
            } else {
                Write-Host "[*] Pilot UI bundle not available; /pilot will serve the existing files or 404." -ForegroundColor Yellow
            }

            # 2. Traditional module directory: <DataDir>\ui\modules\portmaster
            $uiDest = Join-Path $DataDir "ui\modules\portmaster"
            New-Item -ItemType Directory -Path $uiDest -Force | Out-Null
            Expand-Archive -Path $tempZip -DestinationPath $uiDest -Force
            Write-Host "[*] Extracted UI bundle to: $uiDest" -ForegroundColor Gray

            # 3. Update in updates dir if present
            $updatesUi = Join-Path $DataDir "updates\windows_amd64\ui\modules\portmaster"
            if (Test-Path (Split-Path $updatesUi)) {
                New-Item -ItemType Directory -Path $updatesUi -Force | Out-Null
                Copy-Item -Path $tempZip -Destination (Join-Path $updatesUi "portmaster.zip") -Force
                Expand-Archive -Path $tempZip -DestinationPath $updatesUi -Force
            }
        }

        # Clear WebView2 HTTP cache so the desktop window loads fresh UI bundles immediately
        $webViewProfiles = Get-ChildItem -Path "C:\Users" -Directory -ErrorAction SilentlyContinue
        foreach ($prof in $webViewProfiles) {
            $ebBase = Join-Path $prof.FullName "AppData\Local\io.safing.portmaster\EBWebView\Default"
            foreach ($cacheSub in @("Cache", "Code Cache", "GPUCache", "DawnGraphiteCache", "DawnWebGPUCache")) {
                $cachePath = Join-Path $ebBase $cacheSub
                if (Test-Path $cachePath) {
                    Remove-Item -Path $cachePath -Recurse -Force -ErrorAction SilentlyContinue
                }
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
                try {
                    $statusResp = Invoke-RestMethod -Uri "http://127.0.0.1:817/api/v1/ping" -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
                    if ($statusResp) {
                        $apiOk = $true
                        break
                    }
                } catch {
                    Start-Sleep -Seconds 1
                }
            }
        }

        if ($apiOk) {
            Write-Host "[+] API is responsive and ready on http://127.0.0.1:817" -ForegroundColor Green
        } else {
            Write-Host "[!] API is still initializing; background workers are running." -ForegroundColor Yellow
        }

        $launchExe = if ($capturedTrayPaths.Count -gt 0 -and (Test-Path $capturedTrayPaths[0])) {
            $capturedTrayPaths[0]
        } else {
            Join-Path $InstallDir "portmaster.exe"
        }
        if ($hasApp -and $appWasRunning -and (Test-Path $launchExe)) {
            Write-Host "[*] Relaunching Portmaster tray UI ($launchExe)..." -ForegroundColor Gray
            Start-Process -FilePath $launchExe -ArgumentList "--background", "--with-prompts", "--with-notifications"
        }

        Write-Host ""
        Write-Host "==========================================================" -ForegroundColor Green
        Write-Host " [+] Portmaster Windows patched successfully!" -ForegroundColor Green
        Write-Host "     Service: $ServiceName (Running)" -ForegroundColor Green
        Write-Host "     Core: $binName ($InstallDir\portmaster-core.exe)" -ForegroundColor Green
        Write-Host "     Tray: $InstallDir\portmaster.exe (SHA256: $trayHash)" -ForegroundColor Green
        Write-Host "     UI:   $InstallDir\portmaster.zip" -ForegroundColor Green
        Write-Host "     Backup saved in: $currentBackup" -ForegroundColor Green
        Write-Host "==========================================================" -ForegroundColor Green
    } finally {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Invoke-PortmasterPatch @args
