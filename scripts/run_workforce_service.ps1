# Starts the workforce service on port 3002.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\run_workforce_service.ps1

$proto = "postgresql"
$user  = "nexora"
$pw    = "nexora_dev_password"
$host_ = "localhost"
$port  = "5432"
$db    = "nexora"

$env:NEXORA_ENV           = "workforce-dev"
$env:NEXORA_DATABASE__URL = "${proto}://${user}:${pw}@${host_}:${port}/${db}"
$env:NEXORA_REDIS__URL    = "redis://:nexora_dev_password@localhost:6379/0"

Set-Location (Split-Path $PSScriptRoot -Parent)
& ".\target\debug\nexora-workforce-service.exe"
