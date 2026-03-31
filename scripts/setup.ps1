# devopster setup for Windows
# Requires PowerShell 5+ and an internet connection.

Write-Host "==> devopster setup (Windows)"

# ── Install Docker Desktop if missing ────────────────────────────────────────
if (Get-Command docker -ErrorAction SilentlyContinue) {
    Write-Host "==> Docker already installed: $(docker --version)"
} else {
    Write-Host "==> Installing Docker Desktop via winget..."
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install -e --id Docker.DockerDesktop
    } else {
        Write-Host "==> winget not found. Please install Docker Desktop manually:"
        Write-Host "    https://docs.docker.com/desktop/install/windows-install/"
        exit 1
    }
    Write-Host ""
    Write-Host "==> Docker Desktop installed."
    Write-Host "    Please start Docker Desktop, then re-run: make setup"
    exit 0
}

# ── Build image and open shell ────────────────────────────────────────────────
Write-Host "==> Building devopster container image..."
docker build --target dev -t devopster-cli-dev .
docker run --rm -it `
    -v "${env:USERPROFILE}\.config\devopster:/root/.config/devopster" `
    -v "${PWD}:/app" `
    -w /app `
    devopster-cli-dev `
    bash
