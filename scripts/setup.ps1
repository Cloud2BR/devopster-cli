# devopster setup for Windows
# Requires PowerShell 5+ and an internet connection.

function Write-Info($Message) {
    Write-Host "info $Message" -ForegroundColor Cyan
}

function Write-Success($Message) {
    Write-Host "done $Message" -ForegroundColor Green
}

function Write-Warn($Message) {
    Write-Host "warn $Message" -ForegroundColor Yellow
}

function Show-Menu {
    param(
        [string]$Prompt,
        [string[]]$Options
    )

    $selected = 0
    $rendered = 0
    [Console]::CursorVisible = $false

    try {
        while ($true) {
            if ($rendered -gt 0) {
                [Console]::SetCursorPosition(0, [Math]::Max(0, [Console]::CursorTop - $rendered))
            }
            for ($i = 0; $i -lt $rendered; $i++) {
                Write-Host (' ' * [Console]::WindowWidth)
            }
            if ($rendered -gt 0) {
                [Console]::SetCursorPosition(0, [Math]::Max(0, [Console]::CursorTop - $rendered))
            }

            Write-Host "> $Prompt" -ForegroundColor Cyan
            for ($i = 0; $i -lt $Options.Length; $i++) {
                if ($i -eq $selected) {
                    Write-Host "  > $($Options[$i])" -ForegroundColor Yellow
                } else {
                    Write-Host "    $($Options[$i])" -ForegroundColor DarkGray
                }
            }
            $rendered = $Options.Length + 1

            $key = [Console]::ReadKey($true)
            switch ($key.Key) {
                'UpArrow' { $selected = ($selected - 1 + $Options.Length) % $Options.Length }
                'DownArrow' { $selected = ($selected + 1) % $Options.Length }
                'Enter' { break }
            }
        }
    }
    finally {
        [Console]::CursorVisible = $true
    }

    return $selected
}

function Test-InteractiveSession {
    return -not [Console]::IsInputRedirected -and -not [Console]::IsOutputRedirected
}

function Start-BrowserWatcher {
    param(
        [string]$Path
    )

    Set-Content -Path $Path -Value '' -NoNewline
    Start-Job -ArgumentList $Path -ScriptBlock {
        param($WatchPath)

        $lastUrl = ''
        while ($true) {
            $url = ''
            if (Test-Path $WatchPath) {
                $url = (Get-Content -Path $WatchPath -Raw -ErrorAction SilentlyContinue) -replace '\s', ''
            }

            if ($url -and $url -ne $lastUrl) {
                $lastUrl = $url
                try {
                    Start-Process $url | Out-Null
                }
                catch {
                }
            }

            Start-Sleep -Milliseconds 300
        }
    }
}

Write-Info "devopster setup (Windows)"

# ── Validate Docker availability (host dependency) ───────────────────────────
if (Get-Command docker -ErrorAction SilentlyContinue) {
    Write-Success "Docker detected: $(docker --version)"
} else {
    Write-Warn "Docker is required but not installed."
    Write-Host "    Install docs: https://docs.docker.com/desktop/install/windows-install/"
    exit 1
}

docker info *> $null
if ($LASTEXITCODE -ne 0) {
    Write-Warn "Docker is installed but the daemon is not reachable."
    Write-Warn "Start Docker Desktop, then re-run this setup script."
    exit 1
}

# ── Build image and open shell ────────────────────────────────────────────────
Write-Info "Building devopster container image..."
docker build --target dev -t devopster-cli-dev .

$openFile = Join-Path $PWD.Path '.devopster_open_url'
$browserJob = Start-BrowserWatcher -Path $openFile

$dockerCommand = @('bash')
$dockerArgs = @('run', '--rm')
if (Test-InteractiveSession) {
    $dockerArgs += '-it'
    $options = @(
        'Open an interactive shell',
        'Run devopster launcher',
        'Run devopster init',
        'Run devopster login status',
        'Show devopster help'
    )
    $selection = Show-Menu -Prompt 'Choose how to start inside Docker' -Options $options

    switch ($selection) {
        1 { $dockerCommand = @('devopster') }
        2 { $dockerCommand = @('devopster', 'init') }
        3 { $dockerCommand = @('devopster', 'login', 'status') }
        4 { $dockerCommand = @('devopster', '--help') }
    }
} else {
    Write-Info 'Non-interactive session detected. Skipping setup menu and opening bash.'
}

Write-Success 'Starting Docker runtime.'
try {
    $dockerArgs += @(
        '-v', "${env:USERPROFILE}\.config\devopster:/root/.config/devopster",
        '-v', "${PWD}:/app",
        '-w', '/app',
        'devopster-cli-dev'
    ) + $dockerCommand

    & docker @dockerArgs
}
finally {
    if ($browserJob) {
        Stop-Job $browserJob -Force -ErrorAction SilentlyContinue | Out-Null
        Remove-Job $browserJob -Force -ErrorAction SilentlyContinue | Out-Null
    }
    Remove-Item $openFile -Force -ErrorAction SilentlyContinue
}
