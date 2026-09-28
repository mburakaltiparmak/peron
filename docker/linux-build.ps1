# Builds and tests Peron for Linux inside Docker (Docker Desktop must be running).
#   .\docker\linux-build.ps1          # lint + tests + .deb/.rpm/.AppImage into dist-linux/
#   .\docker\linux-build.ps1 test     # lint + tests only
param([ValidateSet("build", "test")] [string]$Mode = "build")
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent

docker build -t peron-linux-build -f "$root\docker\linux-build.Dockerfile" "$root\docker"
if ($LASTEXITCODE) { exit $LASTEXITCODE }

# Named volumes keep node_modules, the cargo cache and build output fast and off the Windows disk.
docker run --rm `
  -v "${root}:/src" `
  -v peron-node-modules:/src/node_modules `
  -v peron-cargo-registry:/root/.cargo/registry `
  -v peron-linux-target:/target `
  peron-linux-build bash -c "tr -d '\r' < docker/linux-build.sh | bash -s $Mode"
exit $LASTEXITCODE
