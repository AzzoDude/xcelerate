using System;
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
                byte[] urlBytes = Encoding.UTF8.GetBytes(url + "\0");
                fixed (byte* p = urlBytes)
                {
                    void* pageHandle = NativeMethods.xcel_browser_new_page(Handle, p);
                    if (pageHandle == null) throw new Exception("Failed to create new page");
                    return new Page(pageHandle);
                }
            }
        }

        public Task<Page> NewPageAsync(string url) => Task.Run(() => NewPage(url));

        public string GetVersion()
        {
            unsafe
            {
                IntPtr ptr = (IntPtr)NativeMethods.xcel_browser_version(Handle);
                if (ptr == IntPtr.Zero) return string.Empty;
                string json = Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
                NativeMethods.xcel_free_string((byte*)ptr);
                return json;
            }
        }

        public Task<string> GetVersionAsync() => Task.Run(() => GetVersion());

        public string GetTargets()
        {
            unsafe
            {
                IntPtr ptr = (IntPtr)NativeMethods.xcel_browser_targets(Handle);
                if (ptr == IntPtr.Zero) return string.Empty;
                string json = Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
                NativeMethods.xcel_free_string((byte*)ptr);
                return json;
            }
        }

        public Task<string> GetTargetsAsync() => Task.Run(() => GetTargets());

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
