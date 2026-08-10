# Starts the workforce service on port 3002.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\run_workforce_service.ps1

$proto = "postgresql"
$user  = "aos"
$pw    = "aos_dev_password"
$host_ = "localhost"
$port  = "5432"
$db    = "aos"

$env:AOS_ENV           = "workforce-dev"
$env:AOS_DATABASE__URL = "${proto}://${user}:${pw}@${host_}:${port}/${db}"
$env:AOS_REDIS__URL    = "redis://:aos_dev_password@localhost:6379/0"

Set-Location (Split-Path $PSScriptRoot -Parent)
& ".\target\debug\aos-workforce-service.exe"
