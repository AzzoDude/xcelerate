using System;
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
                byte[] urlBytes = Encoding.UTF8.GetBytes(url + "\0");
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

        public string GetContent()
        {
            unsafe
            {
                IntPtr ptr = (IntPtr)NativeMethods.xcel_page_content(Handle);
                if (ptr == IntPtr.Zero) return string.Empty;
                string content = Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
                NativeMethods.xcel_free_string((byte*)ptr);
                return content;
            }
        }

        public Task<string> GetContentAsync() => Task.Run(() => GetContent());

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

        public byte[] PrintToPdf()
        {
            unsafe
            {
                ByteBuffer buffer = NativeMethods.xcel_page_pdf(Handle);
                if (buffer.ptr == null) return Array.Empty<byte>();

                byte[] result = new byte[(int)buffer.len];
                Marshal.Copy((IntPtr)buffer.ptr, result, 0, (int)buffer.len);
                NativeMethods.xcel_free_buffer(buffer);
                return result;
            }
        }

        public Task<byte[]> PrintToPdfAsync() => Task.Run(() => PrintToPdf());

        public Element WaitForSelector(string selector)
        {
            unsafe
            {
                byte[] selBytes = Encoding.UTF8.GetBytes(selector + "\0");
                fixed (byte* p = selBytes)
                {
                    void* elHandle = NativeMethods.xcel_page_wait_for_selector(Handle, p);
                    if (elHandle == null) throw new Exception($"Timeout waiting for selector: {selector}");
                    return new Element(elHandle);
                }
            }
        }

        public Task<Element> WaitForSelectorAsync(string selector) => Task.Run(() => WaitForSelector(selector));

        public bool WaitForNavigation()
        {
            unsafe
            {
                return NativeMethods.xcel_page_wait_for_navigation(Handle);
            }
        }

        public Task<bool> WaitForNavigationAsync() => Task.Run(() => WaitForNavigation());

        public string Evaluate(string expression)
        {
            unsafe
            {
                byte[] exprBytes = Encoding.UTF8.GetBytes(expression + "\0");
                fixed (byte* p = exprBytes)
                {
                    IntPtr ptr = (IntPtr)NativeMethods.xcel_page_evaluate(Handle, p);
                    if (ptr == IntPtr.Zero) return string.Empty;
                    string result = Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
                    NativeMethods.xcel_free_string((byte*)ptr);
                    return result;
                }
            }
        }

        public Task<string> EvaluateAsync(string expression) => Task.Run(() => Evaluate(expression));

        public bool Reload()
        {
            unsafe
            {
                return NativeMethods.xcel_page_reload(Handle);
            }
        }

        public Task<bool> ReloadAsync() => Task.Run(() => Reload());

        public bool GoBack()
        {
            unsafe
            {
                return NativeMethods.xcel_page_go_back(Handle);
            }
        }

        public Task<bool> GoBackAsync() => Task.Run(() => GoBack());

        public string AddScriptToEvaluateOnNewDocument(string source)
        {
            unsafe
            {
                byte[] srcBytes = Encoding.UTF8.GetBytes(source + "\0");
                fixed (byte* p = srcBytes)
                {
                    IntPtr ptr = (IntPtr)NativeMethods.xcel_page_add_script_to_evaluate_on_new_document(Handle, p);
                    if (ptr == IntPtr.Zero) return string.Empty;
                    string id = Marshal.PtrToStringAnsi(ptr) ?? string.Empty;
                    NativeMethods.xcel_free_string((byte*)ptr);
                    return id;
                }
            }
        }

        public Task<string> AddScriptToEvaluateOnNewDocumentAsync(string source) => Task.Run(() => AddScriptToEvaluateOnNewDocument(source));

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
