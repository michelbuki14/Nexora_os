import subprocess, os
base = r"C:\Users\miche\OneDrive\Documents\GitHub\Nexora_os"
out = base + "/ws_build.txt"
env = dict(os.environ)
env["CARGO_TARGET_DIR"] = r"C:\Users\miche\nexora-target"
print("CARGO_TARGET_DIR set to:", env.get("CARGO_TARGET_DIR"))
with open(out, "w") as f:
    p = subprocess.run(
        ["cargo", "build", "--workspace"],
        cwd=base, capture_output=True, text=True, env=env, timeout=1800,
    )
    f.write("RC=%d\n" % p.returncode)
    f.write("=== STDOUT (tail 9000) ===\n" + p.stdout[-9000:])
    f.write("=== STDERR (tail 9000) ===\n" + p.stderr[-9000:])
print("RC", p.returncode)
