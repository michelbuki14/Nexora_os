@echo off
set AOS_ENV=audit-dev
set PGUSER=aos
set PGPASS=aos_dev_password
set PGHOST=localhost
set PGPORT=5432
set PGDB=aos
set AOS_DATABASE__URL=postgresql://%PGUSER%:%PGPASS%@%PGHOST%:%PGPORT%/%PGDB%
set REDIS_PASS=aos_dev_password
set AOS_REDIS__URL=redis://:%REDIS_PASS%@localhost:6379/0
cd /d "%~dp0.."
target\debug\aos-audit-service.exe
