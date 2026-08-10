-- `actor_id` on audit_events holds the actor's synthetic ULID (hash of the
-- Keycloak `sub`), which the workforce service has always bound as a 26-char
-- ULID string. The column was mistakenly typed UUID, so any write from a
-- ULID-first service failed with a type mismatch (GET /employees 500).
--
-- Widen to text and cast existing UUID values to their canonical text form so
-- historical rows are unchanged (hash chain stays intact) and new ULIDs fit.
ALTER TABLE audit_events
    ALTER COLUMN actor_id TYPE VARCHAR(36) USING actor_id::text;

-- Index rebuilds automatically on the column type change.
