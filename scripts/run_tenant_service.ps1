# Starts the tenant service on port 3004 (config.tenant-dev.toml).
# Usage: powershell -ExecutionPolicy Bypass -File scripts\run_tenant_service.ps1

$proto = "postgresql"
$user  = "aos"
$pw    = "aos_dev_password"
$host_ = "localhost"
$port  = "5432"
$db    = "aos"

$env:AOS_ENV           = "tenant-dev"
$env:AOS_DATABASE__URL = "${proto}://${user}:${pw}@${host_}:${port}/${db}"
$env:AOS_REDIS__URL    = "redis://:aos_dev_password@localhost:6379/0"

Set-Location (Split-Path $PSScriptRoot -Parent)
& ".\target\debug\aos-tenant-service.exe"
