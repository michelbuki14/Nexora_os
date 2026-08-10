# Writes config.audit-dev.toml for the audit service (port 3001).
# Run: powershell -ExecutionPolicy Bypass -File scripts\write_audit_config.ps1

$db   = "postgresql://aos:aos_dev_password@localhost:5432/aos"
$redis = "redis://:aos_dev_password@localhost:6379/0"

$toml = @"
[service]
name = "aos-audit-service"
version = "0.1.0"
environment = "development"

[server]
host = "0.0.0.0"
port = 3001
request_timeout_secs = 30
body_limit_bytes = 10485760
graceful_shutdown_secs = 30

[database]
url = "$db"
max_connections = 10
min_connections = 2
connect_timeout_secs = 10
idle_timeout_secs = 300
max_lifetime_secs = 1800
enable_logging = false

[redis]
url = "$redis"
max_connections = 10
connection_timeout_secs = 5
command_timeout_secs = 5

[auth]
jwks_url = "http://localhost:8080/realms/aos/protocol/openid-connect/certs"
issuer = "http://localhost:8080/realms/aos"
audience = "aos-api"
jwks_cache_ttl_secs = 300
require_https = false
allowed_algorithms = ["RS256"]

[tracing]
enabled = false
otlp_endpoint = "http://localhost:4317"
service_name = "aos-audit-service"
sample_rate = 1.0
export_timeout_secs = 10
"@

$target = Join-Path $PSScriptRoot "..\config.audit-dev.toml"
[System.IO.File]::WriteAllText($target, $toml, [System.Text.UTF8Encoding]::new($false))
Write-Host "Written: $target"
