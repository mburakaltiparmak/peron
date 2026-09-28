# End-to-end test of the SSH remote view against a throwaway Ubuntu container running sshd.
# Uses the static CLI from dist-linux (run .\docker\linux-build.ps1 first) and a temporary key;
# nothing touches your ~/.ssh. Checks exactly the ssh invocation the app uses
# (peron_core::remote::ssh_args) plus key/known_hosts options for this isolated setup.
param([string]$Work = "$env:TEMP\peron-remote-e2e", [int]$Port = 2222)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$cli = "$root\dist-linux\peron-cli-x86_64-linux"
if (-not (Test-Path $cli)) { throw "missing $cli — run .\docker\linux-build.ps1 first" }

# The previous run's key is read-only for us (see icacls below): make it deletable first, or the
# old key survives, ssh-keygen can't overwrite it and no key.pub is produced.
if (Test-Path "$Work\key") { icacls "$Work\key" /grant "${env:USERNAME}:(F)" | Out-Null }
Remove-Item $Work -Recurse -Force -ErrorAction SilentlyContinue
if (Test-Path $Work) { throw "could not clean $Work" }
New-Item -ItemType Directory -Force $Work | Out-Null
# PowerShell 7 passes "" as a real empty argument (the old '""' trick would set a 2-char passphrase).
ssh-keygen -q -t ed25519 -N "" -f "$Work\key" | Out-Null
if (-not (Test-Path "$Work\key.pub")) { throw "ssh-keygen failed" }
# Windows OpenSSH ignores private keys readable by other users.
icacls "$Work\key" /inheritance:r /grant:r "${env:USERNAME}:(R)" | Out-Null
Copy-Item $cli "$Work\peron-cli"

docker rm -f peron-sshd 2>$null | Out-Null
docker run -d --name peron-sshd -p "${Port}:22" -v "${Work}:/e2e:ro" ubuntu:22.04 bash -c @'
set -e
apt-get update -qq && apt-get install -y -qq openssh-server python3 >/dev/null
mkdir -p /run/sshd /root/.ssh && cp /e2e/key.pub /root/.ssh/authorized_keys && chmod 600 /root/.ssh/authorized_keys
install -m 755 /e2e/peron-cli /usr/local/bin/peron-cli
mkdir -p /srv/shop-api && cd /srv/shop-api && nohup python3 -m http.server 8765 >/dev/null 2>&1 &
exec /usr/sbin/sshd -D -e
'@ | Out-Null

$sshOpts = @("-i", "$Work\key", "-o", "UserKnownHostsFile=$Work\known_hosts", "-o", "StrictHostKeyChecking=accept-new",
             "-o", "BatchMode=yes", "-o", "ConnectTimeout=8", "-p", "$Port", "--", "root@127.0.0.1")
for ($i = 0; $i -lt 60; $i++) {
  ssh @sshOpts true 2>$null; if ($LASTEXITCODE -eq 0) { break }; Start-Sleep -Seconds 2
}

$results = [ordered]@{}
function Check($name, [bool]$ok) { $script:results[$name] = if ($ok) { "PASS" } else { "FAIL" } }

$json = ssh @sshOpts peron-cli list --format json --all --udp --system
Check "ssh + peron-cli list --json exit 0" ($LASTEXITCODE -eq 0)
$json | Set-Content "$Work\remote.json" -Encoding utf8
$snap = $json | ConvertFrom-Json
$srv = $snap.entries | Where-Object { $_.localPort -eq 8765 } | Select-Object -First 1
Check "schemaVersion 1" ($snap.schemaVersion -eq 1)
Check "remote port 8765 listed" ([bool]$srv)
Check "owner python3 with project shop-api" ($srv.processName -like "python3*" -and $srv.project -eq "shop-api")

ssh @sshOpts peron-cli kill --pid $srv.pid --start-ms 1 --yes 2>$null
Check "stale start time refused (exit 5 = changed)" ($LASTEXITCODE -eq 5)
ssh @sshOpts peron-cli kill --pid $srv.pid --start-ms $srv.processStartMs --yes | Out-Null
Check "remote kill exit 0" ($LASTEXITCODE -eq 0)
Start-Sleep -Seconds 2
$after = ssh @sshOpts peron-cli list --format json | ConvertFrom-Json
Check "port 8765 gone after kill" (-not ($after.entries | Where-Object localPort -eq 8765))
ssh @sshOpts peron-cli kill 1 --yes 2>$null
Check "PID 1 / unknown port refused (exit 3 or 6)" ($LASTEXITCODE -in 3, 6)

docker rm -f peron-sshd | Out-Null
$results.GetEnumerator() | ForEach-Object { "{0,-4} {1}" -f $_.Value, $_.Key }
"remote JSON saved to $Work\remote.json — check the GUI parser with:`n  `$env:PERON_REMOTE_JSON=`"$Work\remote.json`"; cargo test --manifest-path src-tauri/Cargo.toml -p peron-core parses_real_remote_output"
if ($results.Values -contains "FAIL") { exit 1 }
