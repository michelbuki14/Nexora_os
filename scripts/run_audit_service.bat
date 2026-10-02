@echo off
set NEXORA_ENV=audit-dev
set PGUSER=nexora
set PGPASS=nexora_dev_password
set PGHOST=localhost
set PGPORT=5432
set PGDB=nexora
set NEXORA_DATABASE__URL=postgresql://%PGUSER%:%PGPASS%@%PGHOST%:%PGPORT%/%PGDB%
set REDIS_PASS=nexora_dev_password
set NEXORA_REDIS__URL=redis://:%REDIS_PASS%@localhost:6379/0
cd /d "%~dp0.."
target\debug\nexora-audit-service.exe
