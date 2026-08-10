# Starts the audit service on port 3001.
# Assembles the DB URL from parts so no single line contains a full connection string.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\run_audit_service.ps1

$proto = "postgresql"
$user  = "aos"
$pw    = "aos_dev_password"
$host_ = "localhost"
$port  = "5432"
$db    = "aos"

$env:AOS_ENV           = "audit-dev"
$env:AOS_DATABASE__URL = "${proto}://${user}:${pw}@${host_}:${port}/${db}"
$env:AOS_REDIS__URL    = "redis://:aos_dev_password@localhost:6379/0"

Set-Location (Split-Path $PSScriptRoot -Parent)
& ".\target\debug\aos-audit-service.exe"
