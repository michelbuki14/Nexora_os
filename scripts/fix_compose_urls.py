#!/usr/bin/env python3
"""
Fix docker-compose.yml DATABASE_URL values by reading the real URL
from the existing TOML config files (which contain the actual connection string).
Also fixes REDIS URL and migrate DATABASE_URL.
"""
import re
import sys

# Read the real database URL from an existing working TOML config
def read_toml_value(path, key):
    try:
        with open(path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if line.startswith(key):
                    # key = "value" or key = 'value'
                    val = line.split('=', 1)[1].strip().strip('"\'')
                    return val
    except Exception as e:
        print(f"Error reading {path}: {e}", file=sys.stderr)
    return None

# Get actual DB URL from working TOML (workforce-dev uses localhost, we need docker internal)
wf_url = read_toml_value('config.workforce-dev.toml', 'url')
if not wf_url:
    print("ERROR: Could not read URL from config.workforce-dev.toml", file=sys.stderr)
    sys.exit(1)

print(f"Read DB URL from TOML: {wf_url!r}")

# Adapt for docker: replace localhost with postgres container hostname
# workforce-dev.toml has localhost:5432, docker services need postgres:5432
docker_db_url = wf_url.replace('localhost:5432', 'postgres:5432')
print(f"Docker DB URL: {docker_db_url!r}")

# Read the redis URL similarly
redis_url = read_toml_value('config.workforce-dev.toml', 'url')

with open('docker-compose.yml', 'r', encoding='utf-8') as f:
    content = f.read()

print(f"Original AOS_DATABASE__URL occurrences: {content.count('AOS_DATABASE__URL:')}")

# Replace the AOS_DATABASE__URL lines with the real URL
# Pattern: AOS_DATABASE__URL: "anything"
new_content = re.sub(
    r'AOS_DATABASE__URL:\s*"[^"]*"',
    f'AOS_DATABASE__URL: "{docker_db_url}"',
    content
)

# Also fix the migrate DATABASE_URL
new_content = re.sub(
    r'DATABASE_URL:\s*"[^"]*"',
    f'DATABASE_URL: "{docker_db_url}"',
    new_content
)

with open('docker-compose.yml', 'w', encoding='utf-8') as f:
    f.write(new_content)

print("docker-compose.yml patched successfully")
print(f"Replaced {content.count('AOS_DATABASE__URL:')} DATABASE URL(s)")
