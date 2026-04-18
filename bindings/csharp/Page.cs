using System;
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
