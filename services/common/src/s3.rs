//! S3-compatible object storage client for AOS document management.
//!
//! All document bytes live in MinIO / S3 — never in the database. The workforce
//! service stores only metadata (`wf_documents`). Access to document bytes is
//! exclusively through short-lived presigned GET URLs issued by the service
//! after RBAC and IDOR checks pass. There is no public-read path.
//!
//! Object keys are namespaced `{tenant_ulid}/{employee_ulid}/{doc_ulid}` so that
//! a key from another tenant is not merely unauthorized but unguessable.

use aws_config::{BehaviorVersion, SdkConfig};
use aws_credential_types::{provider::SharedCredentialsProvider, Credentials};
use aws_sdk_s3::config::{AsyncSleep, Builder as S3Builder, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use std::time::Duration;
use tokio::time::sleep;

use crate::config::S3Config;
use crate::error::{AosError, AosResult};

/// Wrapper to make `tokio::time::sleep` implement `AsyncSleep`.
#[derive(Debug)]
struct TokioSleep;

impl AsyncSleep for TokioSleep {
    fn sleep(&self, duration: Duration) -> aws_sdk_s3::config::Sleep {
        let fut = async move {
            sleep(duration).await;
        };
        aws_sdk_s3::config::Sleep::new(Box::pin(fut))
    }
}

/// Build an S3 `Client` from our config. Call once at startup and clone the Arc.
pub fn build_s3_client(cfg: &S3Config) -> Client {
    let creds = Credentials::new(
        cfg.access_key_id.expose(),
        cfg.secret_access_key.expose(),
        None,
        None,
        "aos-config",
    );
    let sdk_config = SdkConfig::builder()
        .region(Region::new(cfg.region.clone()))
        .credentials_provider(SharedCredentialsProvider::new(creds))
        .endpoint_url(&cfg.endpoint)
        .sleep_impl(TokioSleep)
        .behavior_version(BehaviorVersion::latest())
        .build();

    let s3_cfg = S3Builder::from(&sdk_config)
        .force_path_style(cfg.force_path_style)
        .build();

    Client::from_conf(s3_cfg)
}

/// Generate a `{tenant_ulid}/{employee_ulid}/{doc_ulid}` object key.
///
/// The three-segment namespace means a key from another tenant is both
/// unauthorized (RLS blocks the metadata lookup) and unguessable (ULID entropy).
pub fn document_object_key(tenant_ulid: &str, employee_ulid: &str, doc_ulid: &str) -> String {
    format!("{tenant_ulid}/{employee_ulid}/{doc_ulid}")
}

/// Upload document bytes. Returns the object key on success.
///
/// The caller is responsible for computing `sha256` and passing it through for
/// storage in `wf_documents.sha256` — we do not recompute it here.
pub async fn put_document(
    client: &Client,
    bucket: &str,
    object_key: &str,
    bytes: Vec<u8>,
    mime_type: &str,
) -> AosResult<()> {
    client
        .put_object()
        .bucket(bucket)
        .key(object_key)
        .content_type(mime_type)
        .body(bytes.into())
        .send()
        .await
        .map_err(|e| AosError::Internal(format!("s3 put_object failed: {e}")))?;
    Ok(())
}

/// Generate a presigned GET URL. TTL comes from `S3Config.presign_ttl_secs`.
///
/// Never return a public URL. This function is the only path to fetching bytes.
pub async fn presign_get(
    client: &Client,
    bucket: &str,
    object_key: &str,
    ttl_secs: u64,
) -> AosResult<String> {
    let presigning = PresigningConfig::expires_in(Duration::from_secs(ttl_secs))
        .map_err(|e| AosError::Internal(format!("presign config error: {e}")))?;

    let req = client
        .get_object()
        .bucket(bucket)
        .key(object_key)
        .presigned(presigning)
        .await
        .map_err(|e| AosError::Internal(format!("s3 presign failed: {e}")))?;

    Ok(req.uri().to_string())
}

/// Delete a document from object storage. Used when a document is deactivated
/// and the tenant requests data erasure (GDPR). Soft-delete first (`is_active=false`),
/// hard-delete only after compliance review.
pub async fn delete_document(client: &Client, bucket: &str, object_key: &str) -> AosResult<()> {
    client
        .delete_object()
        .bucket(bucket)
        .key(object_key)
        .send()
        .await
        .map_err(|e| AosError::Internal(format!("s3 delete_object failed: {e}")))?;
    Ok(())
}
