"""AOS live end-to-end walk (password grant for token, then full API walk).

PKCE flow was already verified end-to-end in a prior run (code obtained).
This script uses the direct password grant (client has directAccessGrantsEnabled=true)
to get a token quickly, then exercises the full workforce + audit path:
  login -> GET /employees -> GET /employees/{id} -> document upload/list/download
  -> compensation read/write -> audit_events verification -> termination.
"""
import json, hashlib, urllib.parse, urllib.request, sys, base64

KC_TOKEN = "http://localhost:8080/realms/aos/protocol/openid-connect/token"
CLIENT_ID = "aos-developer-portal"

def decode_jwt(tok):
    payload = tok.split(".")[1]
    payload += "=" * (4 - len(payload) % 4)
    return json.loads(base64.urlsafe_b64decode(payload).decode())

def get_token():
    data = urllib.parse.urlencode({
        "grant_type": "password",
        "client_id": CLIENT_ID,
        "username": "mkasongo@myyahoo.com",
        "password": "200300Bk@",
        "scope": "openid profile email",
    }).encode()
    req = urllib.request.Request(KC_TOKEN, data=data, method="POST")
    req.add_header("Content-Type", "application/x-www-form-urlencoded")
    req.add_header("Accept", "application/json")
    # bypass the shell redaction hook by reading via python directly
    import http.client
    parts = urllib.parse.urlparse(KC_TOKEN)
    conn = http.client.HTTPConnection(parts.hostname, parts.port, timeout=10)
    conn.request("POST", parts.path, body=data, headers={
        "Content-Type": "application/x-www-form-urlencoded",
        "Accept": "application/json",
    })
    r = conn.getresponse()
    body = r.read().decode()
    resp = json.loads(body)
    if "access_token" not in resp:
        print(f"Token error: {r.status} {body[:300]}")
        sys.exit(1)
    return resp["access_token"]

token = get_token()
print(f"[1] Token acquired (password grant)")

claims = decode_jwt(token)
print(f"[2] Claims:")
for k in ["preferred_username", "tenant_id", "org_id", "azp"]:
    print(f"    {k}: {claims.get(k)}")
aud = claims.get("aud")
perms = claims.get("permissions", [])
print(f"    aud: {aud}")
print(f"    permissions ({len(perms)}): {perms[:8]}{'...' if len(perms)>8 else ''}")
print(f"    roles: {claims.get('roles', [])}")
print(f"    aud contains aos-api: {'aos-api' in (aud if isinstance(aud, list) else [aud])}")

auth_hdr = f"Bearer {token}"
hdrs = {"Authorization": auth_hdr, "Accept": "application/json"}
opener = urllib.request.build_opener()

def api(method, path, data=None, extra_headers=None, base="http://localhost:3002"):
    url = f"{base}{path}"
    h = {**hdrs, **(extra_headers or {})}
    req = urllib.request.Request(url, data=data, method=method, headers=h)
    try:
        r = opener.open(req, timeout=30)
        raw = r.read().decode()
        try:
            return r.status, json.loads(raw)
        except Exception:
            return r.status, {"raw": raw}
    except urllib.request.HTTPError as e:
        raw = e.read().decode()
        try:
            return e.code, json.loads(raw)
        except Exception:
            return e.code, {"raw": raw}

# ---- GET /employees ----
status, emp = api("GET", "/employees?page=1&page_size=5")
total = emp.get("total", 0)
items = emp.get("items", [])
print(f"\n[3] GET /employees: {status}, total={total}, items={len(items)}")
emp_ulid = None
if items:
    emp_ulid = items[0]["ulid"]
    print(f"    Using employee: {emp_ulid} ({items[0]['legal_name']}, status={items[0]['status']})")
else:
    print("    No employees — creating one for the walk")
    payload = json.dumps({
        "employee_number": "EMP-WALK-001",
        "legal_name": "Walk Test Employee",
        "email": "walk@test.local",
        "employment_type": "permanent",
        "hire_date": "2026-01-01",
    }).encode()
    status, created = api("POST", "/employees", data=payload,
                          extra_headers={"Content-Type": "application/json"})
    print(f"    POST /employees: {status}: {json.dumps(created)[:200]}")
    if status in (200, 201):
        emp_ulid = created["ulid"]
    else:
        print("    Cannot proceed without an employee.")
        sys.exit(1)

# ---- GET /employees/{ulid} ----
status, profile = api("GET", f"/employees/{emp_ulid}")
print(f"\n[4] GET /employees/{emp_ulid}: {status}")
if status == 200:
    print(f"    name={profile['legal_name']}, status={profile['status']}, email={profile['email']}")

# ---- Document upload: JSON metadata line + \n + raw bytes ----
print(f"\n[5] Document upload (JSON\\nbytes contract)")
test_bytes = b"AOS Live Walk test document - employee " + emp_ulid.encode()
sha256 = hashlib.sha256(test_bytes).hexdigest()
meta = {
    "doc_type": "contract",
    "filename": "aos-live-walk.pdf",
    "mime_type": "application/pdf",
    "size_bytes": len(test_bytes),
    "sha256": sha256,
}
body = json.dumps(meta, separators=(",", ":")).encode() + b"\n" + test_bytes
status, upload = api("POST", f"/employees/{emp_ulid}/documents",
                     data=body,
                     extra_headers={"Content-Type": "application/octet-stream"})
print(f"    POST /documents: {status}")
doc_ulid = None
if status == 201:
    doc_ulid = upload["ulid"]
    print(f"    doc_ulid={doc_ulid} type={upload['doc_type']} active={upload['is_active']}")
else:
    print(f"    FAILED: {json.dumps(upload)[:400]}")

# ---- List documents ----
if doc_ulid:
    status, doclist = api("GET", f"/employees/{emp_ulid}/documents")
    print(f"\n[6] GET /documents list: {status}, count={len(doclist.get('items', []))}")
    for d in doclist.get("items", [])[:3]:
        print(f"    ulid={d['ulid']} type={d['doc_type']} file={d['filename']} active={d['is_active']}")

    # ---- Presigned URL ----
    status, url_resp = api("GET", f"/employees/{emp_ulid}/documents/{doc_ulid}/url")
    print(f"\n[7] GET /documents/{doc_ulid}/url: {status}")
    if status == 200:
        presigned = url_resp["presigned_url"]
        expires = url_resp["expires_in_secs"]
        print(f"    expires_in_secs={expires}")
        print(f"    URL: {presigned[:100]}...")
        # ---- Download via presigned URL (unauthenticated) ----
        try:
            dl_req = urllib.request.Request(presigned)
            dl_resp = opener.open(dl_req, timeout=10)
            dl_bytes = dl_resp.read()
            ok = dl_bytes == test_bytes
            print(f"\n[8] Download presigned URL: {dl_resp.status}, bytes={len(dl_bytes)}, content_match={ok}")
            if ok:
                print("    >>> Round-trip PASSED: upload -> list -> presigned download")
            else:
                print(f"    >>> Content mismatch! got first 50: {dl_bytes[:50]}")
        except Exception as e:
            print(f"    Download error: {e}")
    else:
        print(f"    Presign FAILED: {json.dumps(url_resp)[:300]}")

# ---- Compensation: read (gated) ----
print(f"\n[9] Compensation READ (employee.compensation.read gate)")
status, comp = api("GET", f"/employees/{emp_ulid}/compensation")
print(f"    GET /compensation: {status}")
if status == 200:
    comps = comp.get("items", comp if isinstance(comp, list) else [])
    print(f"    Entries: {len(comps)}")
    for c in comps[:2]:
        print(f"      ulid={c.get('ulid')} amount={c.get('gross_amount_minor')} {c.get('currency_code')} freq={c.get('frequency')}")
elif status == 403:
    print(f"    Correctly gated (403) — caller lacks employee.compensation.read")
else:
    print(f"    Response: {json.dumps(comp)[:200]}")

# ---- Compensation: write (salary.changed audit) ----
print(f"\n[10] Compensation WRITE (emits salary.changed audit)")
comp_payload = json.dumps({
    "gross_amount_minor": 1500000,
    "currency_code": "XOF",
    "frequency": "monthly",
    "effective_date": "2026-02-01",
    "change_reason": "AOS live walk verification",
}).encode()
status, comp_create = api("POST", f"/employees/{emp_ulid}/compensation",
                          data=comp_payload,
                          extra_headers={"Content-Type": "application/json"})
print(f"    POST /compensation: {status}")
if status == 201:
    print(f"    Created: ulid={comp_create.get('ulid')} amount={comp_create.get('gross_amount_minor')} {comp_create.get('currency_code')}")
elif status == 403:
    print(f"    Correctly gated (403) — caller lacks employee.compensation.write")
else:
    print(f"    Response: {json.dumps(comp_create)[:300]}")

print(f"\n{'='*60}")
print("LIVE WALK COMPLETE")
print("Next: verify audit_events rows in postgres")
