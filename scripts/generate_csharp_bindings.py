import subprocess
import os
import shutil
import sys

def run_command(cmd, cwd=None):
    print(f"[EXECUTING] {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, shell=True, capture_output=True, text=True)
    if result.stdout:
        print(result.stdout)
    if result.stderr:
        print(f"[STDERR] {result.stderr}")
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
    
    # Find uniffi-bindgen-cs (with fallback to default install paths)
    tool_cmd = "uniffi-bindgen-cs"
    
    print(f"[DEBUG] Current PATH: {os.environ.get('PATH')}")
    
    if shutil.which(tool_cmd) is None:
        print(f"[DEBUG] Tool '{tool_cmd}' not found in PATH.")
        possible_paths = [
            os.path.join(os.path.expanduser("~"), ".cargo", "bin"),
            os.path.join(os.environ.get("USERPROFILE", ""), ".cargo", "bin"),
            os.path.join(os.path.expanduser("~"), ".dotnet", "tools"),
            "C:\\Users\\runneradmin\\.dotnet\\tools"
        ]
        for dp in possible_paths:
            if os.path.exists(dp):
                print(f"[DEBUG] Checking directory: {dp}")
                # Check for things like uniffi-bindgen-cs.exe
                for f in os.listdir(dp):
                    if "uniffi-bindgen-cs" in f.lower() and f.endswith(".exe"):
                        tool_cmd = os.path.join(dp, f)
                        print(f"[INFO] Found potential tool at fallback path: {tool_cmd}")
                        break
                if tool_cmd != "uniffi-bindgen-cs": break

    # Run bindgen
    run_command([
        tool_cmd, 
        "--library", 
        "--out-dir", csharp_dir, 
        built_dll
    ], cwd=rust_dir)
    
    print("\n--- 3. Fixing Visibility (Internal -> Public) ---")
    # UniFFI C# generator often defaults to internal. We make it public for easier consumption.
    generated_cs = os.path.join(csharp_dir, "xcelerate_core.cs")
    
    with open(generated_cs, "r") as f:
        cs_content = f.read()

    # Replacements list
    replacements = [
        ('internal class', 'public class'),
        ('internal interface', 'public interface'),
        ('internal record', 'public record'),
        ('internal enum', 'public enum'),
        ('internal struct', 'public struct'),
        ('internal abstract class', 'public abstract class'),
    ]
    
    for old, new in replacements:
        cs_content = cs_content.replace(old, new)
    
    # Handle start of line replacements for classes/structs/etc without modifiers
    import re
    cs_content = re.sub(r'^class ', 'public class ', cs_content, flags=re.MULTILINE)
    cs_content = re.sub(r'^struct ', 'public struct ', cs_content, flags=re.MULTILINE)
    cs_content = re.sub(r'^interface ', 'public interface ', cs_content, flags=re.MULTILINE)
    cs_content = re.sub(r'^enum ', 'public enum ', cs_content, flags=re.MULTILINE)

    with open(generated_cs, "w") as f:
        f.write(cs_content)


    # POST-PROCESS: Make BrowserConfig arguments optional in C#
    with open(generated_cs, "r") as f:
        content = f.read()
    
    import re
    # We match the specific Record definition and add defaults
    pattern = r"public record BrowserConfig \(\n    bool @headless, \n    bool @stealth, \n    bool @detached, \n    string\? @executablePath\n\)"
    replacement = r"public record BrowserConfig (\n    bool @headless = true, \n    bool @stealth = true, \n    bool @detached = true, \n    string? @executablePath = null\n)"
    content = re.sub(pattern, replacement, content)
    
    with open(generated_cs, "w") as f:
        f.write(content)

    print("\n--- 4. Distributing Native Libraries ---")
    native_libs = ["xcelerate_core.dll", "libxcelerate_core.so", "libxcelerate_core.dylib"]
    for lib in native_libs:
        src = os.path.join(rust_dir, "target", "release", lib)
        if os.path.exists(src):
            shutil.copy2(src, os.path.join(csharp_dir, lib))
            print(f"[COPY] {lib} -> {csharp_dir}")
    
    # Also copy to TestApp bin directory for immediate testing (windows only usually)
    test_app_bin = os.path.join(csharp_dir, "Xcelerate.TestApp", "bin", "Debug", "net10.0")
    if os.path.exists(test_app_bin):
        shutil.copy2(built_dll, os.path.join(test_app_bin, dll_name))
        print(f"[COPY] {dll_name} -> {test_app_bin}")

    print("\n--- 5. Building C# Wrapper ---")
    run_command(["dotnet", "build", "-c", "Release"], cwd=csharp_dir)
    
    print("\n--- 6. Packaging NuGet (.nupkg) ---")
    run_command(["dotnet", "pack", "-c", "Release"], cwd=csharp_dir)
    
    print("\n--- DONE ---")
    print(f"Xcelerate UniFFI SDK is ready in {os.path.join(csharp_dir, 'bin', 'Release')}")

if __name__ == "__main__":
    main()
