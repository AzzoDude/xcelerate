use crate::browser::{Browser, BrowserConfig};
use crate::page::Page;
use crate::element::Element;
use std::os::raw::{c_char, c_void};
use std::ffi::{CStr, CString};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

#[repr(C)]
pub struct ByteBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

// Memory Helpers
#[unsafe(no_mangle)]
pub extern "C" fn xcel_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { let _ = CString::from_raw(ptr); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_free_buffer(buf: ByteBuffer) {
    if !buf.ptr.is_null() {
        unsafe { let _ = Vec::from_raw_parts(buf.ptr, buf.len, buf.len); }
    }
}

// Browser
#[unsafe(no_mangle)]
pub extern "C" fn xcel_browser_launch(headless: bool) -> *mut c_void {
    let config = BrowserConfig::builder().headless(headless).build().unwrap();
    let result = RUNTIME.block_on(async { Browser::launch(config).await });

    match result {
        Ok((browser, handler)) => {
            RUNTIME.spawn(handler.run());
            Box::into_raw(Box::new(browser)) as *mut c_void
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_browser_new_page(browser_ptr: *mut c_void, url: *const c_char) -> *mut c_void {
    if browser_ptr.is_null() { return std::ptr::null_mut(); }
    let browser = unsafe { &*(browser_ptr as *mut Browser) };
    let url_str = unsafe { CStr::from_ptr(url).to_string_lossy().to_string() };
    
    let result = RUNTIME.block_on(async { browser.new_page(url_str).await });
    match result {
        Ok(page) => Box::into_raw(Box::new(page)) as *mut c_void,
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_browser_version(browser_ptr: *mut c_void) -> *mut c_char {
    if browser_ptr.is_null() { return std::ptr::null_mut(); }
    let browser = unsafe { &*(browser_ptr as *mut Browser) };
    let result = RUNTIME.block_on(async { browser.version().await });
    
    match result {
        Ok(v) => {
            let json = serde_json::to_string(&v).unwrap_or_default();
            CString::new(json).unwrap().into_raw()
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_browser_targets(browser_ptr: *mut c_void) -> *mut c_char {
    if browser_ptr.is_null() { return std::ptr::null_mut(); }
    let browser = unsafe { &*(browser_ptr as *mut Browser) };
    let result = RUNTIME.block_on(async { browser.targets().await });
    
    match result {
        Ok(t) => {
            let json = serde_json::to_string(&t).unwrap_or_default();
            CString::new(json).unwrap().into_raw()
        }
        Err(_) => std::ptr::null_mut(),
    }
}

// Page Properties & Actions
#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_navigate(page_ptr: *mut c_void, url: *const c_char) -> bool {
    if page_ptr.is_null() { return false; }
    let page = unsafe { &*(page_ptr as *mut Page) };
    let url_str = unsafe { CStr::from_ptr(url).to_string_lossy().to_string() };
    RUNTIME.block_on(async { page.navigate(url_str).await.is_ok() })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_title(page_ptr: *mut c_void) -> *mut c_char {
    if page_ptr.is_null() { return std::ptr::null_mut(); }
    let page = unsafe { &*(page_ptr as *mut Page) };
    let title = RUNTIME.block_on(async { page.title().await.unwrap_or_default() });
    CString::new(title).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_content(page_ptr: *mut c_void) -> *mut c_char {
    if page_ptr.is_null() { return std::ptr::null_mut(); }
    let page = unsafe { &*(page_ptr as *mut Page) };
    let content = RUNTIME.block_on(async { page.content().await.unwrap_or_default() });
    CString::new(content).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_screenshot(page_ptr: *mut c_void) -> ByteBuffer {
    if page_ptr.is_null() { return ByteBuffer { ptr: std::ptr::null_mut(), len: 0 }; }
    let page = unsafe { &*(page_ptr as *mut Page) };
    let res = RUNTIME.block_on(async { page.screenshot().await });
    
    match res {
        Ok(mut data) => {
            let buf = ByteBuffer { ptr: data.as_mut_ptr(), len: data.len() };
            std::mem::forget(data);
            buf
        }
        Err(_) => ByteBuffer { ptr: std::ptr::null_mut(), len: 0 },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_pdf(page_ptr: *mut c_void) -> ByteBuffer {
    if page_ptr.is_null() { return ByteBuffer { ptr: std::ptr::null_mut(), len: 0 }; }
    let page = unsafe { &*(page_ptr as *mut Page) };
    let res = RUNTIME.block_on(async { page.pdf().await });
    
    match res {
        Ok(mut data) => {
            let buf = ByteBuffer { ptr: data.as_mut_ptr(), len: data.len() };
            std::mem::forget(data);
            buf
        }
        Err(_) => ByteBuffer { ptr: std::ptr::null_mut(), len: 0 },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_wait_for_selector(page_ptr: *mut c_void, selector: *const c_char) -> *mut c_void {
    if page_ptr.is_null() { return std::ptr::null_mut(); }
    let page = unsafe { &*(page_ptr as *mut Page) };
    let sel_str = unsafe { CStr::from_ptr(selector).to_string_lossy().to_string() };
    
    let result = RUNTIME.block_on(async { page.wait_for_selector(&sel_str).await });
    match result {
        Ok(el) => Box::into_raw(Box::new(el)) as *mut c_void,
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_wait_for_navigation(page_ptr: *mut c_void) -> bool {
    if page_ptr.is_null() { return false; }
    let page = unsafe { &*(page_ptr as *mut Page) };
    RUNTIME.block_on(async { page.wait_for_navigation().await.is_ok() })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_evaluate(page_ptr: *mut c_void, expression: *const c_char) -> *mut c_char {
    if page_ptr.is_null() { return std::ptr::null_mut(); }
    let page = unsafe { &*(page_ptr as *mut Page) };
    let expr_str = unsafe { CStr::from_ptr(expression).to_string_lossy().to_string() };
    let result = RUNTIME.block_on(async { page.evaluate(expr_str).await });
    
    match result {
        Ok(res) => {
            let json = serde_json::to_string(&res).unwrap_or_default();
            CString::new(json).unwrap().into_raw()
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_reload(page_ptr: *mut c_void) -> bool {
    if page_ptr.is_null() { return false; }
    let page = unsafe { &*(page_ptr as *mut Page) };
    RUNTIME.block_on(async { page.reload().await.is_ok() })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_go_back(page_ptr: *mut c_void) -> bool {
    if page_ptr.is_null() { return false; }
    let page = unsafe { &*(page_ptr as *mut Page) };
    RUNTIME.block_on(async { page.go_back().await.is_ok() })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_page_add_script_to_evaluate_on_new_document(page_ptr: *mut c_void, source: *const c_char) -> *mut c_char {
    if page_ptr.is_null() { return std::ptr::null_mut(); }
    let page = unsafe { &*(page_ptr as *mut Page) };
    let src_str = unsafe { CStr::from_ptr(source).to_string_lossy().to_string() };
    let result = RUNTIME.block_on(async { page.add_script_to_evaluate_on_new_document(src_str).await });
    
    match result {
        Ok(id) => CString::new(id).unwrap().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// Element Actions
#[unsafe(no_mangle)]
pub extern "C" fn xcel_element_click(el_ptr: *mut c_void) -> bool {
    if el_ptr.is_null() { return false; }
    let el = unsafe { &*(el_ptr as *mut Element) };
    RUNTIME.block_on(async { el.click().await.is_ok() })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_element_type_text(el_ptr: *mut c_void, text: *const c_char) -> bool {
    if el_ptr.is_null() { return false; }
    let el = unsafe { &*(el_ptr as *mut Element) };
    let text_str = unsafe { CStr::from_ptr(text).to_string_lossy().to_string() };
    RUNTIME.block_on(async { el.type_text(&text_str).await.is_ok() })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_element_text(el_ptr: *mut c_void) -> *mut c_char {
    if el_ptr.is_null() { return std::ptr::null_mut(); }
    let el = unsafe { &*(el_ptr as *mut Element) };
    let text = RUNTIME.block_on(async { el.text().await.unwrap_or_default() });
    CString::new(text).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_element_hover(el_ptr: *mut c_void) -> bool {
    if el_ptr.is_null() { return false; }
    let el = unsafe { &*(el_ptr as *mut Element) };
    RUNTIME.block_on(async { el.hover().await.is_ok() })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_element_focus(el_ptr: *mut c_void) -> bool {
    if el_ptr.is_null() { return false; }
    let el = unsafe { &*(el_ptr as *mut Element) };
    RUNTIME.block_on(async { el.focus().await.is_ok() })
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_element_inner_html(el_ptr: *mut c_void) -> *mut c_char {
    if el_ptr.is_null() { return std::ptr::null_mut(); }
    let el = unsafe { &*(el_ptr as *mut Element) };
    let html = RUNTIME.block_on(async { el.inner_html().await.unwrap_or_default() });
    CString::new(html).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_element_attribute(el_ptr: *mut c_void, name: *const c_char) -> *mut c_char {
    if el_ptr.is_null() { return std::ptr::null_mut(); }
    let el = unsafe { &*(el_ptr as *mut Element) };
    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy().to_string() };
    let attr = RUNTIME.block_on(async { el.attribute(&name_str).await.unwrap_or_default() });
    match attr {
        Some(a) => CString::new(a).unwrap().into_raw(),
        None => std::ptr::null_mut(),
    }
}

// Freeing
#[unsafe(no_mangle)]
pub extern "C" fn xcel_free_browser(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe { 
            let mut browser = Box::from_raw(ptr as *mut Browser);
            let _ = RUNTIME.block_on(async { browser.close().await });
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_free_page(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe { let _ = Box::from_raw(ptr as *mut Page); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn xcel_free_element(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe { let _ = Box::from_raw(ptr as *mut Element); }
    }
}
