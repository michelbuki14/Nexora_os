"""AOS live walk part 2: compensation write + termination + audit verification."""
import json, urllib.parse, urllib.request, sys, base64, http.client

KC_TOKEN = "http://localhost:8080/realms/aos/protocol/openid-connect/token"
CLIENT_ID = "aos-developer-portal"
EMP_ULID = "01ARZ3NDEKTSV4RRFFQ69G5FB5"  # Amina Mukendi, from part 1

def decode_jwt(tok):
    p = tok.split(".")[1]
    p += "=" * (4 - len(p) % 4)
    return json.loads(base64.urlsafe_b64decode(p).decode())

def get_token():
    data = urllib.parse.urlencode({
        "grant_type": "password",
        "client_id": CLIENT_ID,
        "username": "mkasongo@myyahoo.com",
        "password": "200300Bk@",
        "scope": "openid profile email",
    }).encode()
    parts = urllib.parse.urlparse(KC_TOKEN)
    conn = http.client.HTTPConnection(parts.hostname, parts.port, timeout=10)
    conn.request("POST", parts.path, body=data, headers={
        "Content-Type": "application/x-www-form-urlencoded",
        "Accept": "application/json",
    })
    r = conn.getresponse()
    return json.loads(r.read().decode())["access_token"]

token = get_token()
claims = decode_jwt(token)
print(f"[1] Token OK; perms include comp.write={('employee.compensation.write' in claims.get('permissions',[]))}, terminate={('employee.terminate' in claims.get('permissions',[]))}")

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

# ---- Compensation READ before write ----
status, comp = api("GET", f"/employees/{EMP_ULID}/compensation")
print(f"\n[2] GET /compensation (before write): {status}")
if status == 200:
    items = comp.get("items", [])
    print(f"    Existing entries: {len(items)}")
    for c in items[:3]:
        print(f"      {c.get('ulid')} {c.get('gross_amount_minor')} {c.get('currency_code')} {c.get('frequency')} eff={c.get('effective_date')}")

# ---- Compensation WRITE (CDF, salary.changed audit) ----
print(f"\n[3] POST /compensation (CDF — emits salary.changed audit)")
comp_payload = json.dumps({
    "gross_amount_minor": 1500000,
    "currency_code": "CDF",
    "frequency": "monthly",
    "effective_date": "2026-02-01",
    "change_reason": "AOS live walk verification",
}).encode()
status, created = api("POST", f"/employees/{EMP_ULID}/compensation",
                      data=comp_payload,
                      extra_headers={"Content-Type": "application/json"})
print(f"    POST /compensation: {status}")
if status == 201:
    print(f"    Created: ulid={created.get('ulid')} amount={created.get('gross_amount_minor')} {created.get('currency_code')} freq={created.get('frequency')} eff={created.get('effective_date')}")
elif status == 403:
    print(f"    Correctly gated (403)")
else:
    print(f"    FAILED: {json.dumps(created)[:300]}")

# ---- Compensation READ after write ----
status, comp = api("GET", f"/employees/{EMP_ULID}/compensation")
print(f"\n[4] GET /compensation (after write): {status}, items={len(comp.get('items',[]))}")
for c in comp.get("items", [])[:3]:
    print(f"      {c.get('ulid')} {c.get('gross_amount_minor')} {c.get('currency_code')} {c.get('frequency')} eff={c.get('effective_date')}")

# ---- Termination (terminal confirm) ----
print(f"\n[5] PATCH /employees/{EMP_ULID}/status (termination, employee.terminate)")
term_payload = json.dumps({
    "status": "terminated",
    "termination_date": "2026-02-15",
    "termination_reason": "AOS live walk verification — end of contract",
}).encode()
status, term_resp = api("PATCH", f"/employees/{EMP_ULID}/status",
                       data=term_payload,
                       extra_headers={"Content-Type": "application/json"})
print(f"    PATCH /status: {status}")
if status == 200:
    print(f"    Updated: status={term_resp.get('status')} | name={term_resp.get('legal_name')}")
elif status == 403:
    print(f"    Correctly gated (403)")
else:
    print(f"    Response: {json.dumps(term_resp)[:300]}")

# ---- Verify with a final GET ----
status, profile = api("GET", f"/employees/{EMP_ULID}")
print(f"\n[6] Final GET /employees/{EMP_ULID}: {status}, status={profile.get('status')}")

print(f"\n{'='*60}")
print("PART 2 COMPLETE — now verifying audit_events in postgres")
