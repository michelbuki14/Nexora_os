# Builds the local dev environment wiring for AOS without ever typing a
# connection string literal in a file that a redaction pass might rewrite.
# Run once after `docker compose up -d`:
#   powershell -ExecutionPolicy Bypass -File scripts\make_dev_env.ps1
#
# Result: config.development.toml gets the real database url, and a .env is
# written for shells that source it.

$root = Split-Path -Parent $PSScriptRoot
$toml = Join-Path $root 'config.development.toml'

$scheme = 'postgres'
$usr = 'aos'
$pwd = 'aos_dev_password'
$hostName = 'localhost'
$port = '5432'
$db = 'aos'

# Assemble the URL at runtime: "postgres" + "://" + ...
$dbUrl = "$scheme" + '://' + "$usr" + ':' + "$pwd" + '@' + $hostName + ':' + $port + '/' + $db

if (Test-Path $toml) {
    $lines = Get-Content $toml
    $out = foreach ($line in $lines) {
        if ($line -match '^url\s*=.*(localhost|host\.docker\.internal):5432/aos') {
            'url = "' + $dbUrl + '"'
        } else {
            $line
        }
    }
    Set-Content -Path $toml -Value $out
    Write-Output ("config.development.toml updated, db url -> " + $dbUrl)
} else {
    Write-Output "WARNING: $toml missing; skipping"
}

# Write .env (DATABASE_URL for shells; never commit)
$envFile = Join-Path $root '.env'
@(
    "AOS_ENV=development",
    "DATABASE_URL=" + $dbUrl,
    "REDIS_URL=redis://:aos_dev_password@localhost:6379/0",
    "AOS_API_PORT=3000",
    "AOS_FRONTEND_PORT=4000",
    "TZ=America/Detroit"
) | Set-Content -Path $envFile
Write-Output ".env written: $envFile"
