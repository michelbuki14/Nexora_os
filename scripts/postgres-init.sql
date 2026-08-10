-- Create the Keycloak database and user.
-- Executed by the postgres container on first boot via docker-entrypoint-initdb.d.
-- The aos user (POSTGRES_USER) already exists at this point.

CREATE USER keycloak WITH PASSWORD 'keycloak_dev_password';
CREATE DATABASE keycloak OWNER keycloak;
GRANT ALL PRIVILEGES ON DATABASE keycloak TO keycloak;
