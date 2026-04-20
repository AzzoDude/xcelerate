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
    js_dir = os.path.join(root_dir, "bindings", "javascript") # Updated name
    
    print("--- Phase: Generating JavaScript Bindings ---")
    
    # Clean up stale files
    if os.path.exists(js_dir):
        for f in os.listdir(js_dir):
            if f.endswith(".ts") or f.endswith(".js") or f.endswith(".d.ts"):
                try:
                    path = os.path.join(js_dir, f)
                    if os.path.isfile(path) and not f == "package.json": # don't delete package.json
                        os.remove(path)
                except:
                    pass
    os.makedirs(js_dir, exist_ok=True)
    
    # Get version from Cargo.toml
    version = "0.0.0"
    cargo_toml = os.path.join(root_dir, "Cargo.toml")
    if os.path.exists(cargo_toml):
        with open(cargo_toml, "r") as f:
            for line in f:
                if line.startswith("version ="):
                    version = line.split("=")[1].strip().strip('"')
                    break

    # Find uniffi-bindgen-node-js
    tool_cmd = "uniffi-bindgen-node-js"
    if shutil.which(tool_cmd) is None:
        possible_paths = [
            os.path.join(os.path.expanduser("~"), ".cargo", "bin"),
            os.path.join(os.environ.get("USERPROFILE", ""), ".cargo", "bin"),
        ]
        for dp in possible_paths:
            if os.path.exists(dp):
                for f in os.listdir(dp):
                    if "uniffi-bindgen-node-js" in f.lower() and f.endswith(".exe"):
                        tool_cmd = os.path.join(dp, f)
                        break
                if tool_cmd != "uniffi-bindgen-node-js": break

    # Use uniffi-bindgen-node-js
    success = run_command([
        tool_cmd,
        "generate",
        "--out-dir", js_dir,
        "--package-name", "xcelerate",
        built_dll,
    ], cwd=root_dir)
    
    if success:
        # Patch package.json with the correct version and metadata
        package_json_path = os.path.join(js_dir, "package.json")
        if os.path.exists(package_json_path):
            import json
            with open(package_json_path, "r") as f:
                pj = json.load(f)
            
            pj["version"] = version
            pj["description"] = "A high-performance, lightweight Chrome DevTools Protocol (CDP) client for Node.js"
            pj["author"] = "AzzoDude"
            pj["license"] = "MIT"
            pj["engines"] = { "node": ">=12" }
            pj["repository"] = {
                "type": "git",
                "url": "git+https://github.com/AzzoDude/xcelerate.git"
            }
            
            with open(package_json_path, "w") as f:
                json.dump(pj, f, indent=2)
            print(f"[PATCH] Updated package.json to version {version}")

        # Copy all native libraries to the JS folder
        native_libs = ["xcelerate_core.dll", "libxcelerate_core.so", "libxcelerate_core.dylib"]
        for lib in native_libs:
            src = os.path.join(rust_dir, "target", "release", lib)
            if os.path.exists(src):
                shutil.copy2(src, os.path.join(js_dir, lib))
                print(f"[COPY] {lib} -> {js_dir}")
        
        # POST-PROCESS: Optional args and camelCase renames
        js_file = os.path.join(js_dir, "xcelerate_core.js")
        if os.path.exists(js_file):
            with open(js_file, "r") as f:
                content = f.read()
            
            # Inject defaults into Browser.launch
            pattern = r"static async launch\(config\) \{"
            replacement = """static async launch(config = {}) {
    const finalConfig = {
      headless: true,
      stealth: true,
      detached: true,
      executable_path: null,
      ...config
    };
    config = finalConfig;"""
            content = re.sub(pattern, replacement, content)
            
            # Renames
            content = content.replace("async new_page(", "async newPage(")
            content = content.replace("async inner_html(", "async innerHtml(")
            content = content.replace("async screenshot_full(", "async screenshotFull(")
            content = content.replace("async find_element(", "async findElement(")
            content = content.replace("async wait_for_navigation(", "async waitForNavigation(")
            content = content.replace("async wait_for_selector(", "async waitForSelector(")
            content = content.replace("async type_text(", "async typeText(")
            content = content.replace("async add_script_to_evaluate_on_new_document(", "async addScriptToEvaluateOnNewDocument(")

            with open(js_file, "w") as f:
                f.write(content)

        print(f"[SUCCESS] JavaScript bindings ready in {js_dir}")
        
        # Create the .tgz package for artifacts
        run_command(["npm", "pack"], cwd=js_dir)

if __name__ == "__main__":
    main()
