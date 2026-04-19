import sys
import re
import os

def update_file(file_path, pattern, replacement):
    if not os.path.exists(file_path):
        print(f"[WARNING] File not found: {file_path}")
        return
    
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()
    
    new_content = re.sub(pattern, replacement, content)
    
    if new_content != content:
        with open(file_path, "w", encoding="utf-8", newline="") as f:
            f.write(new_content)
        print(f"[UPDATE] {file_path}")
    else:
        print(f"[SKIP] {file_path} (No change or pattern not found)")

def main():
    if len(sys.argv) < 2:
        print("Usage: python scripts/bump_version.py <new_version>")
        print("Example: python scripts/bump_version.py 0.1.5")
        sys.exit(1)

    new_ver = sys.argv[1]
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    print(f"=== Bumping Xcelerate to version {new_ver} ===")

    # 1. Cargo.toml (Rust)
    update_file(
        os.path.join(root_dir, "Cargo.toml"),
        r'^version = "[^"]+"',
        f'version = "{new_ver}"'
    )

    # 2. bindings/javascript/package.json (Node.js)
    update_file(
        os.path.join(root_dir, "bindings", "javascript", "package.json"),
        r'"version": "[^"]+"',
        f'"version": "{new_ver}"'
    )

    # 3. bindings/python/pyproject.toml (Python)
    update_file(
        os.path.join(root_dir, "bindings", "python", "pyproject.toml"),
        r'^version = "[^"]+"',
        f'version = "{new_ver}"'
    )

    # 4. bindings/csharp/Xcelerate.CSharp.csproj (C#)
    update_file(
        os.path.join(root_dir, "bindings", "csharp", "Xcelerate.CSharp.csproj"),
        r'<Version>[^<]+</Version>',
        f'<Version>{new_ver}</Version>'
    )

    # 5. scripts/generate_python_bindings.py (Hardcoded version in __init__.py generation)
    update_file(
        os.path.join(root_dir, "scripts", "generate_python_bindings.py"),
        r'__version__ = "[^"]+"',
        f'__version__ = "{new_ver}"'
    )

    print("=== Done ===")

if __name__ == "__main__":
    main()
