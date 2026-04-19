using System;
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
                byte[] textBytes = Encoding.UTF8.GetBytes(text + "\0");
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
