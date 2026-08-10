-- Runs once on first Postgres container start (docker-entrypoint-initdb.d).
-- Keycloak's compose config points at a separate DB/user that the default
-- POSTGRES_DB=aos init does not create. Without this, Keycloak never becomes ready.

DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'keycloak') THEN
    CREATE ROLE keycloak LOGIN PASSWORD 'keycloak_dev_password';
  END IF;
END
$$;

SELECT 'CREATE DATABASE keycloak OWNER keycloak'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'keycloak')\gexec
