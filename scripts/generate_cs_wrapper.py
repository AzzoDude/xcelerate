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

    print("\n--- 4. Generating High-Level C# Wrappers ---")
    generate_high_level_wrappers(csharp_dir)

    print("\n--- 5. Building C# Wrapper ---")
    run_command(["dotnet", "build", "-c", "Release"], cwd=csharp_dir)
    
    print("\n--- 6. Packaging NuGet (.nupkg) ---")
    run_command(["dotnet", "pack", "-c", "Release"], cwd=csharp_dir)
    
    print("\n--- DONE ---")
    print(f"NuGet package is ready in {os.path.join(csharp_dir, 'bin', 'Release')}")

def generate_high_level_wrappers(target_dir):
    """Generates the high-level C# classes: Browser, Page, Element."""
    
    # Browser.cs
    browser_cs = """using System;
using System.Threading.Tasks;
using System.Runtime.InteropServices;
using System.Text;

namespace Xcelerate
{
    public class Browser : IDisposable
    {
        internal unsafe void* Handle;
        private bool _disposed;

        internal unsafe Browser(void* handle)
        {
            Handle = handle;
        }

        public static Browser Launch(bool headless = true)
        {
            unsafe
            {
                void* handle = NativeMethods.xcel_browser_launch(headless);
                if (handle == null) throw new Exception("Failed to launch browser");
                return new Browser(handle);
            }
        }

        public static Task<Browser> LaunchAsync(bool headless = true) => Task.Run(() => Launch(headless));

        public Page NewPage(string url)
        {
            unsafe
            {
                byte[] urlBytes = Encoding.UTF8.GetBytes(url + "\\0");
                fixed (byte* p = urlBytes)
                {
                    void* pageHandle = NativeMethods.xcel_browser_new_page(Handle, p);
                    if (pageHandle == null) throw new Exception("Failed to create new page");
                    return new Page(pageHandle);
                }
            }
        }

        public Task<Page> NewPageAsync(string url) => Task.Run(() => NewPage(url));

        public void Close() => Dispose();

        public void Dispose()
        {
            if (_disposed) return;
            unsafe
            {
                if (Handle != null) NativeMethods.xcel_free_browser(Handle);
            }
            _disposed = true;
            GC.SuppressFinalize(this);
        }

        ~Browser() => Dispose();
    }
}
"""
    
    # Page.cs
    page_cs = """using System;
using System.Threading.Tasks;
using System.Runtime.InteropServices;
using System.Text;

namespace Xcelerate
{
    public class Page : IDisposable
    {
        internal unsafe void* Handle;
        private bool _disposed;

        internal unsafe Page(void* handle)
        {
            Handle = handle;
        }

        public void Navigate(string url)
        {
            unsafe
            {
                byte[] urlBytes = Encoding.UTF8.GetBytes(url + "\\0");
                fixed (byte* p = urlBytes)
                {
                    if (!NativeMethods.xcel_page_navigate(Handle, p))
                        throw new Exception("Navigation failed");
                }
            }
        }

        public Task NavigateAsync(string url) => Task.Run(() => Navigate(url));

        public string GetTitle()
        {
            unsafe
            {
                IntPtr ptr = (IntPtr)NativeMethods.xcel_page_title(Handle);
                if (ptr == IntPtr.Zero) return string.Empty;
                string title = Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
                NativeMethods.xcel_free_string((byte*)ptr);
                return title;
            }
        }

        public Task<string> GetTitleAsync() => Task.Run(() => GetTitle());

        public byte[] Screenshot()
        {
            unsafe
            {
                ByteBuffer buffer = NativeMethods.xcel_page_screenshot(Handle);
                if (buffer.ptr == null) return Array.Empty<byte>();

                byte[] result = new byte[(int)buffer.len];
                Marshal.Copy((IntPtr)buffer.ptr, result, 0, (int)buffer.len);
                NativeMethods.xcel_free_buffer(buffer);
                return result;
            }
        }

        public Task<byte[]> ScreenshotAsync() => Task.Run(() => Screenshot());

        public Element WaitForSelector(string selector)
        {
            unsafe
            {
                byte[] selBytes = Encoding.UTF8.GetBytes(selector + "\\0");
                fixed (byte* p = selBytes)
                {
                    void* elHandle = NativeMethods.xcel_page_wait_for_selector(Handle, p);
                    if (elHandle == null) throw new Exception($"Timeout waiting for selector: {selector}");
                    return new Element(elHandle);
                }
            }
        }

        public Task<Element> WaitForSelectorAsync(string selector) => Task.Run(() => WaitForSelector(selector));

        public void Dispose()
        {
            if (_disposed) return;
            unsafe
            {
                if (Handle != null) NativeMethods.xcel_free_page(Handle);
            }
            _disposed = true;
            GC.SuppressFinalize(this);
        }

        ~Page() => Dispose();
    }
}
"""

    # Element.cs
    element_cs = """using System;
using System.Threading.Tasks;
using System.Runtime.InteropServices;
using System.Text;

namespace Xcelerate
{
    public class Element : IDisposable
    {
        internal unsafe void* Handle;
        private bool _disposed;

        internal unsafe Element(void* handle)
        {
            Handle = handle;
        }

        public void Click()
        {
            unsafe
            {
                if (!NativeMethods.xcel_element_click(Handle))
                    throw new Exception("Click failed");
            }
        }

        public Task ClickAsync() => Task.Run(() => Click());

        public void TypeText(string text)
        {
            unsafe
            {
                byte[] textBytes = Encoding.UTF8.GetBytes(text + "\\0");
                fixed (byte* p = textBytes)
                {
                    if (!NativeMethods.xcel_element_type_text(Handle, p))
                        throw new Exception("Typing failed");
                }
            }
        }

        public Task TypeTextAsync(string text) => Task.Run(() => TypeText(text));

        public string GetText()
        {
            unsafe
            {
                IntPtr ptr = (IntPtr)NativeMethods.xcel_element_text(Handle);
                if (ptr == IntPtr.Zero) return string.Empty;
                string text = Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
                NativeMethods.xcel_free_string((byte*)ptr);
                return text;
            }
        }

        public Task<string> GetTextAsync() => Task.Run(() => GetText());

        public void Dispose()
        {
            if (_disposed) return;
            unsafe
            {
                if (Handle != null) NativeMethods.xcel_free_element(Handle);
            }
            _disposed = true;
            GC.SuppressFinalize(this);
        }

        ~Element() => Dispose();
    }
}
"""

    with open(os.path.join(target_dir, "Browser.cs"), "w") as f: f.write(browser_cs)
    with open(os.path.join(target_dir, "Page.cs"), "w") as f: f.write(page_cs)
    with open(os.path.join(target_dir, "Element.cs"), "w") as f: f.write(element_cs)
    print(f"[GEN] Generated Browser, Page, Element classes in {target_dir}")

if __name__ == "__main__":
    main()
