import subprocess
import os
import sys

def run_command(cmd, cwd=None):
    print(f"\n[PHASE] Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, shell=True)
    if result.returncode != 0:
        print(f"[ERROR] Command failed with code {result.returncode}")
        return False
    return True

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    scripts_dir = os.path.join(root_dir, "scripts")

    print("=== Xcelerate Universal Bindgen Pipeline ===")

    # 1. Build Rust
    print("\n--- Phase 1: Building Rust Core (Release) ---")
    if not run_command(["cargo", "build", "--release"], cwd=root_dir):
        sys.exit(1)

    # 2. C#
    run_command(["python", os.path.join(scripts_dir, "generate_csharp_bindings.py")])

    # 3. Python
    run_command(["python", os.path.join(scripts_dir, "generate_python_bindings.py")])

    # 4. JavaScript
    run_command(["python", os.path.join(scripts_dir, "generate_javascript_bindings.py")])

    print("\n=== Universal Pipeline Finished Successfully ===")

if __name__ == "__main__":
    main()
