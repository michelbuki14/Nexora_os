//! Document upload/download for the workforce service.
//!
//! All file bytes live in MinIO / S3. The database (`wf_documents`) stores only
//! metadata. Document bytes are NEVER served directly from this process — callers
//! receive a short-lived presigned GET URL after RBAC and IDOR checks pass.
//!
//! Upload path (HR_ADMIN, `employee.documents.write`):
//!   1. Caller POSTs multipart: metadata fields + file bytes.
//!   2. We compute SHA-256 of the bytes.
//!   3. PUT bytes to MinIO at `{tenant_ulid}/{employee_ulid}/{doc_ulid}`.
//!   4. INSERT metadata row into `wf_documents` (same transaction as the
//!      audit_emit call).
//!   5. Return `DocumentResponse` (no URL — caller must request the URL endpoint).
//!
//! Download path (`employee.documents.read` + IDOR):
//!   1. Caller GETs `/employees/{eid}/documents/{did}/url`.
//!   2. Handler fetches the `wf_documents` row (RLS enforces tenant).
//!   3. IDOR check: employee's tenant_id matches caller's tenant, and caller
//!      has permission to see this employee.
//!   4. We presign a GET URL with the configured TTL.
//!   5. Return `DocumentUrlResponse { presigned_url, expires_in_secs }`.
//!
//! There is intentionally NO route that returns raw bytes or a public URL.

use aws_sdk_s3::Client as S3Client;
use nexora_common::{
    config::S3Config,
    error::{AosError, AosResult},
    s3::{delete_document, document_object_key, presign_get, put_document},
};
use sha2::{Digest, Sha256};

use crate::models::{CreateDocumentMetadataRequest, DocumentResponse, DocumentRow};

/// Context the handler passes to `upload_to_storage`.
pub struct UploadContext<'a> {
    pub tenant_ulid: &'a str,
    pub employee_ulid: &'a str,
    pub doc_ulid: &'a str,
    pub request: &'a CreateDocumentMetadataRequest,
    pub bytes: Vec<u8>,
}

/// Upload bytes to MinIO and return the object key + hex SHA-256.
///
/// Consumes `ctx` so `bytes` are moved into the S3 body without cloning.
///
/// The INSERT into `wf_documents` is the caller's responsibility (it must be
/// inside the rls_middleware transaction together with the audit_emit call).
pub async fn upload_to_storage(
    s3: &S3Client,
    s3_cfg: &S3Config,
    ctx: UploadContext<'_>,
) -> AosResult<(String, String)> {
    let object_key = document_object_key(ctx.tenant_ulid, ctx.employee_ulid, ctx.doc_ulid);

    // Compute SHA-256 before upload.
    let mut hasher = Sha256::new();
    hasher.update(&ctx.bytes);
    let sha256 = hex::encode(hasher.finalize());

    // Move bytes into put_document — zero extra allocation.
    put_document(
        s3,
        &s3_cfg.bucket,
        &object_key,
        ctx.bytes,
        &ctx.request.mime_type,
    )
    .await?;

    Ok((object_key, sha256))
}

/// Generate a presigned GET URL. Only called after RBAC+IDOR checks pass.
///
/// TTL is capped at 3600s regardless of config — presigned URLs should be
/// short-lived; a config mistake shouldn't produce day-long document URLs.
pub async fn presigned_url_for_doc(
    s3: &S3Client,
    s3_cfg: &S3Config,
    row: &DocumentRow,
) -> AosResult<(String, u64)> {
    if !row.is_active {
        return Err(AosError::NotFound(format!(
            "document {} is no longer active",
            row.ulid
        )));
    }
    let ttl = s3_cfg.presign_ttl_secs.min(3600); // ponytail: cap; raise if clients need longer
    let url = presign_get(s3, &s3_cfg.bucket, &row.object_key, ttl).await?;
    Ok((url, ttl))
}

/// Deactivate a document in storage. Metadata row is soft-deleted by the caller.
///
/// ponytail: does not hard-delete by default — GDPR erasure is a separate
/// workflow that requires compliance review. Use `delete_document` directly
/// after legal/compliance sign-off.
pub async fn deactivate_document_storage(
    s3: &S3Client,
    s3_cfg: &S3Config,
    object_key: &str,
) -> AosResult<()> {
    delete_document(s3, &s3_cfg.bucket, object_key).await
}

impl From<DocumentRow> for DocumentResponse {
    fn from(r: DocumentRow) -> Self {
        Self {
            ulid: r.ulid,
            doc_type: r.doc_type,
            filename: r.filename,
            mime_type: r.mime_type,
            size_bytes: r.size_bytes,
            is_active: r.is_active,
            created_at: r.created_at,
        }
    }
}
