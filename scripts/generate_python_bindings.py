import subprocess
import os
import shutil
import sys
import re

def run_command(cmd, cwd=None):
    print(f"[EXECUTING] {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, shell=True)
    if result.returncode != 0:
        print(f"[ERROR] Command failed with code {result.returncode}")
        return False
    return True

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    rust_dir = root_dir
    dll_name = "xcelerate_core.dll"
    built_dll = os.path.join(rust_dir, "target", "release", dll_name)
    python_dir = os.path.join(root_dir, "bindings", "python")
    
    print("--- Phase: Generating Python Bindings ---")
    os.makedirs(python_dir, exist_ok=True)
    
    # 1. Generate Python code using UniFFI
    success = run_command([
        "cargo", "run", "--features=uniffi/cli", "--bin", "uniffi-bindgen",
        "generate", "--library", built_dll,
        "--language", "python",
        "--out-dir", python_dir
    ], cwd=rust_dir)
    
    if success:
        # Move the native libraries to the python package folder
        package_dir = os.path.join(python_dir, "xcelerate")
        os.makedirs(package_dir, exist_ok=True)
        
        # Look for all platform binaries in target/release
        native_libs = ["xcelerate_core.dll", "libxcelerate_core.so", "libxcelerate_core.dylib"]
        for lib in native_libs:
            src = os.path.join(rust_dir, "target", "release", lib)
            if os.path.exists(src):
                shutil.copy2(src, os.path.join(package_dir, lib))
                print(f"[COPY] {lib} -> {package_dir}")
        
        # Ensure __init__.py exists with version
        init_file = os.path.join(package_dir, "__init__.py")
        with open(init_file, "w") as f:
            f.write("__version__ = \"0.1.4\"\n")
            f.write("from .xcelerate_core import Browser, BrowserConfig, Page, Element, XcelerateError\n")
            f.write("__all__ = [\"Browser\", \"BrowserConfig\", \"Page\", \"Element\", \"XcelerateError\"]\n")

        # Move the generated py file to the package folder
        generated_py = os.path.join(python_dir, "xcelerate_core.py")
        if os.path.exists(generated_py):
            # POST-PROCESS: Make BrowserConfig arguments optional
            with open(generated_py, "r") as f:
                content = f.read()
            
            pattern = r"def __init__\(self, \*, headless: \"bool\", stealth: \"bool\", detached: \"bool\", executable_path: \"typing\.Optional\[str\]\"\):"
            replacement = r'def __init__(self, *, headless: "bool" = True, stealth: "bool" = True, detached: "bool" = True, executable_path: "typing.Optional[str]" = None):'
            content = re.sub(pattern, replacement, content)
            
            with open(os.path.join(package_dir, "xcelerate_core.py"), "w") as f:
                f.write(content)
            os.remove(generated_py)

        print(f"[PACKAGING] Building Python wheel...")
        run_command(["python", "-m", "build"], cwd=python_dir)
        print(f"[SUCCESS] Python package ready in {os.path.join(python_dir, 'dist')}")

if __name__ == "__main__":
    main()
