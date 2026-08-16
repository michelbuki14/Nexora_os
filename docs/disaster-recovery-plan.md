# Nexora OS Disaster Recovery Plan
# ==============================
# 
# This plan ensures business continuity for the Nexora OS platform in the event of
# infrastructure failures, data center outages, or regional disasters.
# 
# Plan Version: 1.0.0
# Last Reviewed: 2026-08-15
# Next Quarterly Drill: 2026-11-15
#
# =============================================================================
# Table of Contents
# =============================================================================
# 1. Overview and Objectives
# 2. Recovery Time Objective (RTO) and Recovery Point Objective (RPO)
# 3. Backup Strategies
#   - PostgreSQL Backups (pgBackRest)
#   - MinIO Object Storage Backups
#   - Keycloak Realm Exports
#   - Vault Data Recovery
# 4. Restore Procedures
#   - PostgreSQL Restore
#   - MinIO Restore
#   - Keycloak Realm Restore
#   - Full Service Restore
# 5. Quarterly Drill Schedule
# 6. Roles and Responsibilities
# 7. Communication Plan
# 8. Testing and Validation
# 9. Appendix: References and Links
#
# =============================================================================
# 1. Overview and Objectives
# =============================================================================
#
# This disaster recovery (DR) plan covers the Nexora OS platform, which consists of:
# - PostgreSQL database (primary data store)
# - MinIO object storage (document repository)
# - Keycloak (identity and access management)
# - Vault (secrets management)
# - All Nexora OS microservices (API gateway, workforce, audit, tenant)
#
# Objectives:
# - Restore database availability within 4 hours (RTO)
# - Restore data to within 1 hour of last consistent point (RPO)
# - Minimize data loss and downtime
# - Ensure all services can be restored in a consistent state
# - Conduct quarterly drills to validate the plan
#
# =============================================================================
# 2. Recovery Time Objective (RTO) and Recovery Point Objective (RPO)
# =============================================================================
#
# | Component          | RTO (Max) | RPO (Max) | Notes |
# |--------------------|-----------|-----------|-------|
# | PostgreSQL         | 4 hours   | 1 hour    | pgBackRest with WAL archiving |
# | MinIO object store  | 2 hours   | 30 min    | Replicated across Availability Zones |
# | Keycloak           | 4 hours   | 1 hour    | Realm export + database restore |
# | Vault secrets      | 1 hour    | Near real-time | In-memory + periodic snapshots |
# | Full service stack | 8 hours   | 2 hours   | End-to-end service restoration |
#
# =============================================================================
# 3. Backup Strategies
# =============================================================================
#
# ### 3.1 PostgreSQL Backups (pgBackRest)
#
# pgBackRest is the primary backup solution for PostgreSQL. Configuration:
#
# - **Backup full**: Daily at 02:00 UTC
# - **Backup incremental**: Every 2 hours during business hours
# - **WAL archiving**: To MinIO bucket `nexora-wal-archives`
# - **Retention**: 35 days full backups, 14 days incrementals
# - **Compression**: Yes (lz4)
# - **Checksum**: Yes (CRC32c)
#
# Backup configuration is managed via the `pgBackRest` container addon.
# All backups are verified on creation with `pgBackRest --stanza=nexora backup-verify`.
#
# ### 3.2 MinIO Object Storage Backups
#
# MinIO data is backed up using the following strategy:
#
# - **Cross-region replication**: Enabled for `nexora-workforce-documents` bucket
#   - Source: Primary region (e.g., us-east-1)
#   - Destination: Secondary region (e.g., us-west-2)
# - **Bucket-level replication**: Only objects with `x-amz-replication-enabled: true`
# - **Retention**: 90 days minimum
# - **Verification**: Monthly checksum comparison between primary and secondary
#
# ### 3.3 Keycloak Realm Exports
#
# Keycloak realms are exported periodically and stored in version control:
#
# - **Export frequency**: Daily at 03:00 UTC
# - **Export location**: `keycloak/realm-export.json` (version-controlled)
# - **Export content**: Complete realm configuration including users, groups, clients
# - **Verification**: Realm export is validated against live Keycloak instance
#
# ### 3.4 Vault Data Recovery
#
# Vault data is protected as follows:
#
# - **In-memory storage**: Secrets are stored in Vault's memory-only storage
# - **Periodic snapshots**: Vault agent snapshots saved to MinIO
# - **Snapshot frequency**: Every 6 hours
# - **Retention**: 30 days of snapshots
# - **Recovery**: Restore from snapshot + re-seal with existing unseal keys
#
# =============================================================================
# 4. Restore Procedures
# =============================================================================
#
# ### 4.1 PostgreSQL Restore
#
# **Objective**: Restore PostgreSQL to a point within the RPO (1 hour).
#
# **Prerequisites**:
# - pgBackRest installed and configured
# - MinIO WAL archives accessible
# - Access to backup passwords and keys
#
# **Steps**:
#
# 1. Identify the target recovery time (within RPO window)
# 2. Run: `pgBackRest --stanza=nexora restore`
# 3. Specify the backup ID to restore from
# 4. pgBackRest will restore the data directory and WAL files
# 5. Replay WAL archives up to the target time
# 6. Start PostgreSQL: `pg_ctl start -D /var/lib/postgresql/data`
# 7. Verify database integrity: `psql -c "SELECT 1"` 
# 8. Update DNS/connection strings to point to restored instance
#
# **Verification**:
# - Run `pg_check` to validate backup integrity
# - Execute test queries against restored data
# - Confirm tenant isolation policies are intact
#
# ### 4.2 MinIO Restore
#
# **Objective**: Restore MinIO object storage from cross-region replication.
#
# **Prerequisites**:
# - MinIO server deployed in recovery region
# - Replication configuration intact
# - Access keys available
#
# **Steps**:
#
# 1. Deploy MinIO in the recovery region
# 2. Enable cross-region replication from backup bucket
# 3. Verify replication is functioning
# 4. Update DNS/endpoints to point to restored MinIO
# 5. Confirm document accessibility
#
# **Verification**:
# - List objects in restored bucket
# - Verify checksums match primary
# - Test document upload/download
#
# ### 4.3 Keycloak Realm Restore
#
# **Objective**: Restore Keycloak realm configuration.
#
# **Prerequisites**:
# - Realm export JSON available
# - Keycloak server deployed
# - Admin credentials available
#
# **Steps**:
#
# 1. Stop Keycloak (if running)
# 2. Replace `keycloak/realm-export.json` with the target backup
# 3. Start Keycloak with `-Dkeycloak.import=realm-export.json`
# 4. Verify realm configuration
# 5. Confirm users can authenticate
# 6. Update client configuration if needed
#
# ### 4.4 Full Service Restore
#
# **Objective**: Restore all Nexora OS services to a consistent state.
#
# **Prerequisites**:
# - All prerequisite infrastructure (Kubernetes, DB, MinIO, Keycloak, Vault)
# - Backup metadata and restore runbook
# - Communication channels operational
#
# **Steps**:
#
# 1. Restore PostgreSQL (Section 4.1)
# 2. Restore MinIO (Section 4.2) if document storage affected
# 3. Restore Keycloak (Section 4.3) if identity affected
# 4. Restore Vault secrets from snapshots (Section 4.5)
# 5. Apply database migrations: `nexora-migrate`
# 6. Redeploy all Kubernetes services: `kubectl apply -k .`
# 7. Verify health checks pass on all services
# 8. Conduct end-to-end functional test
# 9. Update monitoring dashboards with new instance info
#
# **Post-Restore Validation**:
# - Run integration test suite
# - Verify tenant isolation (cross-tenant tests)
# - Confirm audit log integrity
# - Test key workflows (employee onboarding, document upload, etc.)
#
# =============================================================================
# 5. Quarterly Drill Schedule
# =============================================================================
#
# The following drills will be conducted quarterly, with the first drill
# scheduled within 30 days of plan approval.
#
# ### Drill 1: PostgreSQL Restore Drill (Quarterly)
# - **Date**: 2026-11-15 (first Monday of quarter)
# - **Objective**: Restore PostgreSQL from latest backup within RTO
# - **Participants**: Database team, DevOps, Engineering
# - **Duration**: 2 hours
# - **Success Criteria**:
#   - PostgreSQL restored within 4 hours
#   - Data within RPO (1 hour)
#   - Tenant isolation policies verified
#   - Audit log integrity confirmed
#
# ### Drill 2: Full Service Restore Drill (Quarterly)
# - **Date**: 2026-11-15 (same day as Drill 1)
# - **Objective**: Restore full Nexora OS service stack within RTO
# - **Participants**: DevOps, Engineering, QA, Security
# - **Duration**: 4 hours
# - **Success Criteria**:
#   - All services deployed and healthy
#   - Tenant isolation verified
#   - End-to-end workflows functional
#   - Monitoring dashboards operational
#
# ### Drill 3: MinIO Replication Failover Drill (Quarterly)
# - **Date**: 2026-11-15 (same day as Drills 1 & 2)
# - **Objective**: Verify cross-region MinIO replication failover
# - **Participants**: DevOps, Storage team
# - **Duration**: 1 hour
# - **Success Criteria**:
#   - Replication resumes in failover region
#   - Data accessible in failover region
#   - RPO met for object storage
#
# ### Drill 4: Vault Snapshot Restore Drill (Quarterly)
# - **Date**: 2026-11-15 (same day as Drills 1-3)
# - **Objective**: Restore Vault from snapshot within RTO
# - **Participants**: Security, DevOps
# - **Duration**: 1 hour
# - **Success Criteria**:
#   - Vault unsealed and operational
#   - Secrets accessible
#   - No data loss confirmed
#
# ### Drill 5: Keycloak Realm Restore Drill (Quarterly)
# - **Date**: 2026-11-15 (same day as Drills 1-4)
# - **Objective**: Restore Keycloak realm from export within RTO
# - **Participants**: Security, DevOps
# - **Duration**: 1 hour
# - **Success Criteria**:
#   - Keycloak operational with restored realm
#   - Users can authenticate
#   - No authentication gaps
#
# **Drill Reporting**:
# - Each drill produces a drill report documented in `~/.gstack-dev/drill-reports/`
# - Reports include: date, participants, duration, successes, failures, actions
# - Action items from drills are tracked and must be resolved within 30 days
# - Drill reports are reviewed in the quarterly ops review
#
# =============================================================================
# 6. Roles and Responsibilities
# =============================================================================
#
# | Role | Responsibilities |
# |------|-----------------|
# | **Database Administrator** | pgBackRest configuration, backup verification, PostgreSQL restore |
# | **DevOps Engineer** | Kubernetes restore, MinIO failover, infrastructure orchestration |
# | **Security Engineer** | Vault restore, Keycloak realm restore, secrets validation |
# | **Site Reliability Engineer** | Monitoring validation, health check verification, incident coordination |
# | **Engineering Lead** | End-to-end functional testing, business impact assessment |
# | **Incident Manager** | Coordination of DR activities, stakeholder communication, timeline tracking |
#
# =============================================================================
# 7. Communication Plan
# =============================================================================
#
# **Internal Communication**:
# - #drinks Slack/Teams channel for DR notifications
# - Email to on-call rotation for critical alerts
# - PagerDuty for incident escalation
#
# **External Communication** (if customer-impacting):
# - Status page update (status.nexora-os.com)
# - Customer success team notification (if data affected)
# - Regulatory notification (if personal data affected, per African data protection laws)
#
# **Escalation Path**:
# 1. On-call engineer (first response, within 15 minutes)
# 2. DevOps lead (within 30 minutes)
# 3. Engineering manager (within 1 hour)
# 4. CTO (within 4 hours for customer-impacting incidents)
#
# =============================================================================
# 8. Testing and Validation
# =============================================================================
#
# **Automated Tests**:
# - `bun test` includes RLS isolation tests (run on every commit)
# - `bun run test:evals` includes E2E tests with tier classification
# - Backup verification: `pgBackRest --stanza=nexora backup-verify`
# - Health check endpoints on all services
#
# **Manual Validation**:
# - Quarterly DR drills (as scheduled above)
# - Monthly backup integrity checks
# - Cross-region replication status verification
#
# **Success Metrics**:
# - 100% of quarterly drills pass within RTO/RPO
# - Zero data loss in verified restores
# - 100% tenant isolation verification on restore
# - All audit logs intact after restore
#
# =============================================================================
# 9. Appendix: References and Links
# =============================================================================
#
# - pgBackRest Documentation: https://pgbackrest.org/
# - MinIO Documentation: https://min.io/docs/
# - Keycloak Documentation: https://www.keycloak-docs/
# - Vault Documentation: https://www.vaultproject/docs/
# - Nexora OS Architecture: docs/architecture.md
# - CI/CD Pipeline: .github/workflows/ci.yml
# - DR Runbook: ops/dr-runbook.md (internal, not checked in)
#
# =============================================================================
# Plan Maintenance
# =============================================================================
#
# This plan is reviewed and updated:
# - Quarterly, alongside DR drills
# - After any major infrastructure change
# - After any DR drill failure (lessons learned incorporated)
# - After any regulatory change affecting data protection
#
# Plan owner: Site Reliability Engineering team
# Plan approval: CTO & Security Lead
#
# Distribution: On-call rotation, DevOps team, Security team, Engineering leadership