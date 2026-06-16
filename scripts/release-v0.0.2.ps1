# savant-bot v0.0.2 release creation script
# Reads GITHUB_TOKEN from .env, posts release via REST API.
# Usage: pwsh -File scripts/release-v0.0.2.ps1

$ErrorActionPreference = "Stop"

# --- Token loading (redacted on display) ---
$envFile = Join-Path (Get-Location) ".env"
if (-not (Test-Path $envFile)) {
    Write-Output "ERROR: .env file not found at $envFile"
    exit 1
}
$envContent = Get-Content $envFile
$tokenLine = $envContent | Where-Object { $_.StartsWith("GITHUB_TOKEN=") } | Select-Object -First 1
if (-not $tokenLine) {
    Write-Output "ERROR: GITHUB_TOKEN= line not found in .env"
    exit 1
}
$token = $tokenLine.Substring("GITHUB_TOKEN=".Length).Trim()
$env:GITHUB_TOKEN = $token
Write-Output "Token loaded. Length: $($token.Length) chars (redacted)."

# --- Payload construction ---
$body = @"
## savant-bot v0.0.2 — Patch release

Ships the openrouter/auto → openrouter/free default-model fix from v0.0.1. No code or API changes; only default value, documentation, and version bumps. Wire-compatible with v0.0.1.

### Fixed

- **Default LLM model** (`src/config.rs`): `llm_default_model` default corrected from `openrouter/auto` to `openrouter/free`. The earlier default was a wrong-interpretation bug (operator's original URL had `openrouter/free` as the literal model slug, not a navigation hint).
- **README.md intro line:** Updated to reference `openrouter/free` (was `openrouter/auto`).

### Audit

- **Verified unchanged:** all `src/*.rs` source files, `.env.example`, migrations. ECHO Protocol conformity still v0.1.3 (this release bumps app version only).
- **Validated:** all 6 ECHO validation commands PASS; 25/25 unit tests pass.
- **FID-151 N/A:** no new `pub fn` introduced this release.
- **Spec consistency:** `Cargo.toml` v0.0.2 ↔ `protocol.config.yaml` `project.version` v0.0.2 ↔ `CHANGELOG.md` ↔ `README.md` v0.0.2.
- **Tracked by:** `FID-2026-0616-011` (archived in `dev/fids/archive/`).

### Upgrade path

```bash
git pull origin main
cargo build --release
```

Users who set `LLM_DEFAULT_MODEL` explicitly are unaffected. Users relying on the v0.0.1 default now get the corrected `openrouter/free`.

### Dual-versioning note

This release bumps the **app version** (`0.0.2`). The **protocol version** (used for ECHO Protocol conformity tracking via `VERSION` + `protocol.config.yaml:protocol.version`) remains `0.1.3` — the protocol itself did not change this release. See `CHANGELOG.md` preamble for the rule.

**Full Changelog:** [`v0.0.1...v0.0.2`](https://github.com/fame0528/savant-bot/compare/v0.0.1...v0.0.2)
"@

$payload = @{
    tag_name         = "v0.0.2"
    target_commitish = "main"
    name             = "savant-bot v0.0.2 — Ship openrouter/free default fix"
    draft            = $false
    prerelease       = $false
    body             = $body
}

$json = $payload | ConvertTo-Json -Depth 10
$jsonPath = Join-Path $env:TEMP "savant-bot-release-v0.0.2.json"
$json | Set-Content -Path $jsonPath -Encoding utf8 -NoNewline
Write-Output "Payload written. Length: $($json.Length) chars. Path: $jsonPath"

# --- POST ---
Write-Output "POSTing to https://api.github.com/repos/fame0528/savant-bot/releases ..."
try {
    $resp = Invoke-RestMethod `
        -Uri "https://api.github.com/repos/fame0528/savant-bot/releases" `
        -Method Post `
        -Headers @{
            Authorization = "Bearer $env:GITHUB_TOKEN"
            Accept        = "application/vnd.github+json"
            "User-Agent"  = "savant-bot-release-script"
        } `
        -ContentType "application/json" `
        -Body $json

    Write-Output ""
    Write-Output "=== RELEASE CREATED ==="
    Write-Output ("html_url:      " + $resp.html_url)
    Write-Output ("tag_name:      " + $resp.tag_name)
    Write-Output ("name:          " + $resp.name)
    Write-Output ("id:            " + $resp.id)
    Write-Output ("published_at:  " + $resp.published_at)
    Write-Output ("author.login:  " + $resp.author.login)
    Write-Output ("target_commitish: " + $resp.target_commitish)
    Write-Output ""
    Write-Output "Verification: visit $($resp.html_url) in browser to confirm render."
} catch {
    Write-Output ""
    Write-Output "=== ERROR ==="
    if ($_.Exception.Response) {
        $statusCode = [int]$_.Exception.Response.StatusCode
        Write-Output ("Status:   " + $statusCode)
    }
    Write-Output ("Message:  " + $_.Exception.Message)
    if ($_.ErrorDetails) {
        Write-Output ("ErrorDetails: " + $_.ErrorDetails.Message)
    }
    exit 1
}
