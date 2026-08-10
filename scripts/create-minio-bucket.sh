#!/usr/bin/env bash
# create-minio-bucket.sh
# One-shot MinIO bucket creation for the demo/workforce service.
# Run after `docker compose up -d minio` (and network is up).

set -euo pipefail

ALIAS="aos"
ENDPOINT="${MINIO_ENDPOINT:-http://minio:9000}"
ACCESS="${MINIO_ROOT_USER:-minioadmin}"
SECRET="${MINIO_ROOT_PASSWORD:-minioadmin}"
BUCKET="${MINIO_BUCKET:-aos-workforce-documents}"

echo "Setting up MinIO alias '$ALIAS' -> $ENDPOINT"
docker run --rm --network aos-network minio/mc:latest \
    alias set "$ALIAS" "$ENDPOINT" "$ACCESS" "$SECRET" >/dev/null

echo "Creating bucket '$BUCKET' (private)"
docker run --rm --network aos-network minio/mc:latest \
    mb --ignore-existing "$ALIAS/$BUCKET"

echo "Verifying bucket exists"
docker run --rm --network aos-network minio/mc:latest \
    ls "$ALIAS/$BUCKET"

echo "Bucket '$BUCKET' ready at $ENDPOINT"
echo "Console: http://localhost:9001 (user: $ACCESS)"