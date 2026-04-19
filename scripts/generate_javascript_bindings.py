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
    
    # Use uniffi-bindgen-node-js (installed via Cargo in CI)
    success = run_command([
        "uniffi-bindgen-node-js",
        "generate",
        "--out-dir", js_dir,
        built_dll,
    ], cwd=root_dir)
    
    if success:
        # Copy the DLL to the JS folder
        shutil.copy2(built_dll, os.path.join(js_dir, dll_name))
        
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

if __name__ == "__main__":
    main()
