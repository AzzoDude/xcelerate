//! Windows **UI Automation** backend for xcelerate.
//!
//! Gives native windows the same shape the CDP browser already has: an indexed
//! `snapshot` of the element tree and `click <index>` on any of it. This is what
//! replaces hand-rolled PowerShell UIA when a native OS dialog (class `#32770`)
//! appears in front of an Electron/CEF/WebView2 app.

use std::fmt;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::POINT;
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation8, IUIAutomation, IUIAutomationElement, IUIAutomationInvokePattern,
    IUIAutomationScrollPattern, IUIAutomationSelectionItemPattern, IUIAutomationValuePattern,
    ScrollAmount_LargeDecrement, ScrollAmount_LargeIncrement, ScrollAmount_NoAmount,
    TreeScope_Children, TreeScope_Descendants, UIA_ControlTypePropertyId, UIA_InvokePatternId,
    UIA_ScrollPatternId, UIA_SelectionItemPatternId, UIA_ValuePatternId, UIA_WindowControlTypeId,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT, SendInput,
    VIRTUAL_KEY, VK_DOWN, VK_NEXT, VK_PRIOR, VK_RETURN, VK_SPACE, VK_UP, mouse_event,
};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos, SetForegroundWindow};
use windows::core::BSTR;
use windows::core::VARIANT;

#[derive(Debug)]
pub enum UiaError {
    Win(windows::core::Error),
    NotFound(String),
    NoElement(usize),
}

impl fmt::Display for UiaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiaError::Win(error) => write!(f, "uia: {error}"),
            UiaError::NotFound(name) => write!(f, "no window matching {name:?}"),
            UiaError::NoElement(index) => {
                write!(f, "no element at index {index}; run `tree` first")
            }
        }
    }
}

impl std::error::Error for UiaError {}

impl From<windows::core::Error> for UiaError {
    fn from(error: windows::core::Error) -> Self {
        UiaError::Win(error)
    }
}

pub type Result<T> = std::result::Result<T, UiaError>;

/// One element of the last snapshot, `snapshot`-shaped like the browser's.
#[derive(Debug, Clone)]
pub struct Element {
    pub index: usize,
    pub role: String,
    pub name: String,
    pub value: String,
    /// Screen rectangle `(x, y, width, height)`, physical pixels.
    pub rect: (i32, i32, i32, i32),
}

pub struct Uia {
    automation: IUIAutomation,
    /// Elements backing the indices of the last [`Uia::snapshot`].
    elements: Vec<IUIAutomationElement>,
}

/// A top-level window.
#[derive(Debug, Clone)]
pub struct Window {
    pub name: String,
    pub class: String,
    pub pid: i32,
}

impl Uia {
    pub fn new() -> Result<Self> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let automation: IUIAutomation = CoCreateInstance(&CUIAutomation8, None, CLSCTX_ALL)?;
            Ok(Self {
                automation,
                elements: Vec::new(),
            })
        }
    }

    /// Top-level windows, fastest path (one `FindAll`, no per-window walking).
    pub fn windows(&self) -> Result<Vec<Window>> {
        let root = unsafe { self.automation.GetRootElement()? };
        let found = unsafe { root.FindAll(TreeScope_Children, &self.window_condition()?) }?;
        let mut out = Vec::new();
        for i in 0..unsafe { found.Length()? } {
            let el = unsafe { found.GetElement(i)? };
            out.push(Window {
                name: text(unsafe { el.CurrentName() }),
                class: text(unsafe { el.CurrentClassName() }),
                pid: unsafe { el.CurrentProcessId() }.unwrap_or(0),
            });
        }
        Ok(out)
    }

    /// Indexed element tree of the first window whose title contains `window`.
    pub fn snapshot(&mut self, window: &str, limit: usize) -> Result<Vec<Element>> {
        let root = self.find_window(window)?;
        self.elements.clear();
        let mut out = Vec::new();
        unsafe {
            let all = root.FindAll(
                TreeScope_Descendants,
                &self.automation.CreateTrueCondition()?,
            )?;
            let count = all.Length()?.min(limit as i32);
            for i in 0..count {
                let el = all.GetElement(i)?;
                let rect = el.CurrentBoundingRectangle().unwrap_or_default();
                out.push(Element {
                    index: out.len(),
                    role: text(el.CurrentLocalizedControlType()),
                    name: text(el.CurrentName()),
                    value: String::new(),
                    rect: (
                        rect.left,
                        rect.top,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                    ),
                });
                self.elements.push(el);
            }
        }
        Ok(out)
    }

    /// Trigger element `index` via its `InvokePattern` - works even when the
    /// element reports no bounds (UIA-virtualized, as Store/app-list cards do).
    pub fn invoke(&self, index: usize) -> Result<()> {
        let el = self.elements.get(index).ok_or(UiaError::NoElement(index))?;
        let pattern: IUIAutomationInvokePattern =
            unsafe { el.GetCurrentPatternAs(UIA_InvokePatternId)? };
        unsafe { pattern.Invoke()? };
        Ok(())
    }

    /// Find the first element whose name contains `needle` and click it
    /// (pattern-first). Returns the `(index, method)` it used. More robust than
    /// an index when the tree animates (a rotating hero), since a name is stable
    /// across frames; prefers a `button`.
    pub fn click_name(
        &mut self,
        window: &str,
        needle: &str,
        limit: usize,
    ) -> Result<(usize, &'static str)> {
        let infos = self.snapshot(window, limit)?;
        let index = find_element(&infos, needle)
            .ok_or_else(|| UiaError::NotFound(needle.to_ascii_lowercase()))?;
        Ok((index, self.click(index)?))
    }

    /// Trigger element `index`, preferring a **pattern** (`InvokePattern`) so the
    /// real cursor never moves; only falls back to a coordinate click when the
    /// element exposes no pattern. Returns `"invoke"` or `"mouse"`.
    pub fn click(&self, index: usize) -> Result<&'static str> {
        if self.invoke(index).is_ok() {
            return Ok("invoke");
        }
        let el = self.elements.get(index).ok_or(UiaError::NoElement(index))?;
        // A list/nav item: select it via pattern - still no cursor.
        if let Ok(pattern) = unsafe {
            el.GetCurrentPatternAs::<IUIAutomationSelectionItemPattern>(UIA_SelectionItemPatternId)
        } {
            unsafe { pattern.Select()? };
            return Ok("select");
        }
        let rect = unsafe { el.CurrentBoundingRectangle().unwrap_or_default() };
        let cx = (rect.left + rect.right) / 2;
        let cy = (rect.top + rect.bottom) / 2;
        unsafe {
            let _ = SetCursorPos(cx, cy);
            std::thread::sleep(Duration::from_millis(80));
            mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
            mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
        }
        Ok("mouse")
    }

    /// Scroll the window with the real **wheel**, which is the only thing a
    /// Chromium/WebView2 surface honours (it needs the pointer over the content).
    /// The cursor is parked on the window's centre for the wheel, then restored.
    pub fn wheel(&self, window: &str, notches: i32) -> Result<()> {
        let element = self.find_window(window)?;
        let hwnd = unsafe { element.CurrentNativeWindowHandle()? };
        let rect = unsafe { element.CurrentBoundingRectangle().unwrap_or_default() };
        let (cx, cy) = ((rect.left + rect.right) / 2, (rect.top + rect.bottom) / 2);
        let mut previous = POINT::default();
        unsafe {
            let _ = SetForegroundWindow(hwnd);
            let had_previous = GetCursorPos(&mut previous).is_ok();
            let _ = SetCursorPos(cx, cy);
            std::thread::sleep(Duration::from_millis(200));
            let delta = 120 * notches.signum();
            let inputs: Vec<INPUT> = (0..notches.abs()).map(|_| wheel_input(delta)).collect();
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            std::thread::sleep(Duration::from_millis(150));
            if had_previous {
                let _ = SetCursorPos(previous.x, previous.y);
            }
        }
        Ok(())
    }

    /// Send one key to the window. Keyboard scroll (`Next`/`Prior`) is the
    /// cursor-free way to scroll a Chromium/WebView2 surface, where the wheel
    /// needs the pointer over the content.
    pub fn key(&self, window: &str, key: VIRTUAL_KEY) -> Result<()> {
        let element = self.find_window(window)?;
        let hwnd = unsafe { element.CurrentNativeWindowHandle()? };
        unsafe {
            let _ = SetForegroundWindow(hwnd);
        }
        std::thread::sleep(Duration::from_millis(150));
        let inputs = [
            key_input(key, KEYBD_EVENT_FLAGS(0)),
            key_input(key, KEYEVENTF_KEYUP),
        ];
        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }
        std::thread::sleep(Duration::from_millis(120));
        Ok(())
    }

    /// Send a named key: `next`, `prior`, `down`, `up`, `space`, `enter`.
    pub fn key_name(&self, window: &str, name: &str) -> Result<()> {
        let key = match name.to_ascii_lowercase().as_str() {
            "next" => VK_NEXT,
            "prior" => VK_PRIOR,
            "down" => VK_DOWN,
            "up" => VK_UP,
            "space" => VK_SPACE,
            "enter" => VK_RETURN,
            other => return Err(UiaError::NotFound(format!("key `{other}`"))),
        };
        self.key(window, key)
    }

    /// Scroll the window via the UIA `ScrollPattern` - no wheel, no cursor.
    /// `notches` < 0 scrolls down, > 0 scrolls up. Returns the method used, or
    /// `"none"` when no scrollable element exposes the pattern.
    pub fn scroll(&mut self, window: &str, notches: i32) -> Result<&'static str> {
        self.snapshot(window, 400)?;
        let (horizontal, vertical) = if notches < 0 {
            (ScrollAmount_NoAmount, ScrollAmount_LargeIncrement)
        } else {
            (ScrollAmount_NoAmount, ScrollAmount_LargeDecrement)
        };
        for element in &self.elements {
            if let Ok(pattern) = unsafe {
                element.GetCurrentPatternAs::<IUIAutomationScrollPattern>(UIA_ScrollPatternId)
            } {
                for _ in 0..notches.abs() {
                    unsafe { pattern.Scroll(horizontal, vertical)? };
                }
                return Ok("pattern");
            }
        }
        Ok("none")
    }

    /// Poll until an element whose name contains `text` appears (built-in
    /// waiting, so the caller never polls). Returns the matched name.
    pub fn wait_for(&mut self, window: &str, text: &str, timeout_ms: u64) -> Result<String> {
        let needle = text.to_ascii_lowercase();
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            let infos = self.snapshot(window, 400)?;
            if let Some(hit) = infos
                .iter()
                .find(|e| e.name.to_ascii_lowercase().contains(&needle))
            {
                return Ok(hit.name.clone());
            }
            if Instant::now() >= deadline {
                return Err(UiaError::NotFound(format!("wait for {text:?} timed out")));
            }
            std::thread::sleep(Duration::from_millis(300));
        }
    }

    /// Set element `index`'s value via `ValuePattern` - types into a field with no
    /// cursor and no synthetic keystrokes.
    pub fn set_value(&self, index: usize, text: &str) -> Result<()> {
        let el = self.elements.get(index).ok_or(UiaError::NoElement(index))?;
        let pattern: IUIAutomationValuePattern =
            unsafe { el.GetCurrentPatternAs(UIA_ValuePatternId)? };
        unsafe { pattern.SetValue(&BSTR::from(text))? };
        Ok(())
    }

    fn window_condition(
        &self,
    ) -> Result<windows::Win32::UI::Accessibility::IUIAutomationCondition> {
        unsafe {
            Ok(self.automation.CreatePropertyCondition(
                UIA_ControlTypePropertyId,
                &VARIANT::from(UIA_WindowControlTypeId.0),
            )?)
        }
    }

    fn find_window(&self, needle: &str) -> Result<IUIAutomationElement> {
        let root = unsafe { self.automation.GetRootElement()? };
        let found = unsafe { root.FindAll(TreeScope_Children, &self.window_condition()?) }?;
        let needle = needle.to_ascii_lowercase();
        for i in 0..unsafe { found.Length()? } {
            let el = unsafe { found.GetElement(i)? };
            if text(unsafe { el.CurrentName() })
                .to_ascii_lowercase()
                .contains(&needle)
            {
                return Ok(el);
            }
        }
        Err(UiaError::NotFound(needle))
    }
}

/// The index of the first element whose name contains `needle`, preferring a
/// `button` (stable across animated frames). The single source of truth for
/// every surface that acts by name.
pub fn find_element(infos: &[Element], needle: &str) -> Option<usize> {
    let needle = needle.to_ascii_lowercase();
    infos
        .iter()
        .find(|e| e.role == "button" && e.name.to_ascii_lowercase().contains(&needle))
        .or_else(|| {
            infos
                .iter()
                .find(|e| e.name.to_ascii_lowercase().contains(&needle))
        })
        .map(|e| e.index)
}

/// One-line element rendering (`[index] <role> "name"  WxH@X,Y`), shared by the
/// CLI and the MCP server so the layouts never drift apart.
pub fn format_element(el: &Element) -> String {
    let (x, y, w, h) = el.rect;
    format!(
        "[{}] <{}> {:?}  {}x{}@{},{}",
        el.index, el.role, el.name, w, h, x, y
    )
}

/// One-line window rendering (`name [class] pid N`), shared by every surface.
pub fn format_window(window: &Window) -> String {
    format!(
        "{:<44} [{:<26}] pid {}",
        window.name, window.class, window.pid
    )
}

/// A `MOUSEINPUT` wheel delta inside an `INPUT`, for `SendInput`.
fn wheel_input(delta: i32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: delta as u32,
                dwFlags: MOUSEEVENTF_WHEEL,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// A `KEYBDINPUT` inside an `INPUT`, for `SendInput`.
fn key_input(key: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// A `BSTR` property, or `""` when the property is absent.
fn text(value: windows::core::Result<BSTR>) -> String {
    value.map(|b| b.to_string()).unwrap_or_default()
}
