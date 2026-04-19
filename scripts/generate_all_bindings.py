import subprocess
import os
import shutil
import sys

def run_command(cmd, cwd=None, env=None):
    print(f"[EXECUTING] {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, shell=True, env=env)
    if result.returncode != 0:
        print(f"[ERROR] Command failed with code {result.returncode}")
        return False
    return True

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    rust_dir = root_dir
    dll_name = "xcelerate_core.dll"
    built_dll = os.path.join(rust_dir, "target", "release", dll_name)

    print("=== Xcelerate All-Bindgen Utility ===")
    
    # 1. Build Rust
    print("\n--- Phase 1: Building Rust Core ---")
    if not run_command(["cargo", "build", "--release"], cwd=rust_dir):
        sys.exit(1)

    # 2. C# Bindings
    print("\n--- Phase 2: Generating C# Bindings ---")
    csharp_dir = os.path.join(root_dir, "bindings", "csharp")
    if os.path.exists(csharp_dir):
        run_command(["python", os.path.join(root_dir, "scripts", "generate_cs_wrapper.py")], cwd=rust_dir)
    else:
        print("[SKIP] C# folder not found")

    # 3. Python Bindings
    print("\n--- Phase 3: Generating Python Bindings ---")
    python_dir = os.path.join(root_dir, "bindings", "python")
    os.makedirs(python_dir, exist_ok=True)
    
    # Generate using the built-in UniFFI CLI
    success = run_command([
        "cargo", "run", "--features=uniffi/cli", "--bin", "uniffi-bindgen",
        "generate", "--library", built_dll,
        "--language", "python",
        "--out-dir", python_dir
    ], cwd=rust_dir)
    
    if success:
        # Move the DLL to the python package folder
        package_dir = os.path.join(python_dir, "xcelerate")
        os.makedirs(package_dir, exist_ok=True)
        shutil.copy2(built_dll, os.path.join(package_dir, dll_name))
        
        # Ensure __init__.py exists
        init_file = os.path.join(package_dir, "__init__.py")
        if not os.path.exists(init_file):
            with open(init_file, "w") as f:
                f.write("from .xcelerate_core import Browser, BrowserConfig, Page, Element, XcelerateError\n")
                f.write("__all__ = [\"Browser\", \"BrowserConfig\", \"Page\", \"Element\", \"XcelerateError\"]\n")

        # Move the generated py file to the package folder
        generated_py = os.path.join(python_dir, "xcelerate_core.py")
        if os.path.exists(generated_py):
            # POST-PROCESS: Make BrowserConfig arguments optional by adding defaults to __init__
            with open(generated_py, "r") as f:
                content = f.read()
            
            # Use regex to find the __init__ of BrowserConfig and add defaults
            import re
            pattern = r"def __init__\(self, \*, headless: \"bool\", stealth: \"bool\", detached: \"bool\", executable_path: \"typing\.Optional\[str\]\"\):"
            replacement = r'def __init__(self, *, headless: "bool" = True, stealth: "bool" = True, detached: "bool" = True, executable_path: "typing.Optional[str]" = None):'
            content = re.sub(pattern, replacement, content)
            
            with open(os.path.join(package_dir, "xcelerate_core.py"), "w") as f:
                f.write(content)
            os.remove(generated_py)

        print(f"[PACKAGING] Building Python wheel for xcelerate...")
        run_command(["python", "-m", "build"], cwd=python_dir)
        print(f"[SUCCESS] Python package ready in {os.path.join(python_dir, 'dist')}")

    # 4. JS Bindings
    print("\n--- Phase 4: Generating JavaScript (Node.js) Bindings ---")
    js_dir = os.path.join(root_dir, "bindings", "js")
    os.makedirs(js_dir, exist_ok=True)
    
    # Check for uniffi-bindgen-js
    # Standard UniFFI doesn't have JS. We try to use the community one if available.
    print("[INFO] Attempting to use uniffi-bindgen-node-js...")
    js_success = run_command([
        "uniffi-bindgen-node-js",
        "--library", built_dll,
        "--out-dir", js_dir
    ], cwd=rust_dir)
    
    if not js_success:
        print("[WARNING] uniffi-bindgen-node-js not found in PATH.")
        print("[HINT] Run: cargo install uniffi-bindgen-node-js")
        print("[HINT] Then re-run this script.")
    else:
        shutil.copy2(built_dll, os.path.join(js_dir, dll_name))
        print(f"[COPY] Copied {dll_name} to {js_dir}")

    print("\n=== All Bindings Task Finished ===")

if __name__ == "__main__":
    main()
