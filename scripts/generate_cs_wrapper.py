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
    
    print("\n--- 2. Generating UniFFI C# Bindings ---")
    dll_name = "xcelerate_core.dll"
    built_dll = os.path.join(rust_dir, "target", "release", dll_name)
    
    # Run uniffi-bindgen-cs
    run_command([
        "uniffi-bindgen-cs", 
        "--library", 
        "--out-dir", csharp_dir, 
        built_dll
    ], cwd=rust_dir)
    
    print("\n--- 3. Fixing Visibility (Internal -> Public) ---")
    # UniFFI C# generator often defaults to internal. We make it public for easier consumption.
    generated_cs = os.path.join(csharp_dir, "xcelerate_core.cs")
    ps_command = (
        f"(Get-Content '{generated_cs}') "
        "-replace 'internal class', 'public class' "
        "-replace 'internal interface', 'public interface' "
        "-replace 'internal record', 'public record' "
        "-replace 'internal enum', 'public enum' "
        "-replace 'internal struct', 'public struct' "
        "-replace 'internal abstract class', 'public abstract class' "
        "-replace '^class ', 'public class ' "
        "-replace '^struct ', 'public struct ' "
        "-replace '^interface ', 'public interface ' "
        "-replace '^enum ', 'public enum ' "
        f"| Set-Content '{generated_cs}'"
    )
    run_command(["powershell", "-Command", ps_command])

    print("\n--- 4. Distributing Native Library ---")
    target_dll = os.path.join(csharp_dir, dll_name)
    shutil.copy2(built_dll, target_dll)
    print(f"[COPY] Copied {dll_name} to {csharp_dir}")
    
    # Also copy to TestApp bin directory for immediate testing
    test_app_bin = os.path.join(csharp_dir, "Xcelerate.TestApp", "bin", "Debug", "net10.0")
    if os.path.exists(test_app_bin):
        shutil.copy2(built_dll, os.path.join(test_app_bin, dll_name))
        print(f"[COPY] Copied {dll_name} to {test_app_bin}")

    print("\n--- 5. Building C# Wrapper ---")
    run_command(["dotnet", "build", "-c", "Release"], cwd=csharp_dir)
    
    print("\n--- 6. Packaging NuGet (.nupkg) ---")
    run_command(["dotnet", "pack", "-c", "Release"], cwd=csharp_dir)
    
    print("\n--- DONE ---")
    print(f"Xcelerate UniFFI SDK is ready in {os.path.join(csharp_dir, 'bin', 'Release')}")

if __name__ == "__main__":
    main()
