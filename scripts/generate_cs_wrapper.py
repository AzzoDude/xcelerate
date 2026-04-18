import subprocess
import os
import shutil
import sys

def run_command(cmd, cwd=None):
    print(f"[EXECUTING] {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, shell=True)
    if result.returncode != 0:
        print(f"[ERROR] Command failed with code {result.returncode}")
        sys.exit(result.returncode)

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    rust_dir = root_dir
    csharp_dir = os.path.join(root_dir, "bindings", "csharp")
    
    print("--- 1. Building Rust Library (cdylib) ---")
    run_command(["cargo", "build", "--release"], cwd=rust_dir)
    
    print("\n--- 2. Preparing C# Project ---")
    # Identify the DLL path (Windows assumption based on OS information)
    dll_name = "xcelerate.dll"
    built_dll = os.path.join(rust_dir, "target", "release", dll_name)
    target_dll = os.path.join(csharp_dir, dll_name)
    
    if os.path.exists(built_dll):
        shutil.copy2(built_dll, target_dll)
        print(f"[COPY] Copied {dll_name} to {csharp_dir}")
    else:
        print(f"[ERROR] Could not find {built_dll}. Only Windows (dll) is currently supported in this script.")
        sys.exit(1)

    print("\n--- 3. Running CsBindGen (via build.rs) ---")
    # This is actually done during cargo build, but we ensure output existence
    generated_cs = os.path.join(rust_dir, "src", "bindings", "csharp", "NativeMethods.g.cs")
    final_cs = os.path.join(csharp_dir, "NativeMethods.g.cs")
    
    if os.path.exists(generated_cs):
        shutil.move(generated_cs, final_cs)
        print(f"[MOVE] Moved generated bindings to {csharp_dir}")
    else:
        print(f"[WARNING] Generated bindings not found at {generated_cs}. Ensure cargo build ran build.rs.")

    print("\n--- 4. Building C# Wrapper ---")
    run_command(["dotnet", "build", "-c", "Release"], cwd=csharp_dir)
    
    print("\n--- 5. Packaging NuGet (.nupkg) ---")
    run_command(["dotnet", "pack", "-c", "Release"], cwd=csharp_dir)
    
    print("\n--- DONE ---")
    print(f"NuGet package is ready in {os.path.join(csharp_dir, 'bin', 'Release')}")

if __name__ == "__main__":
    main()
