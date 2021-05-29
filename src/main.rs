#![windows_subsystem = "windows"]

use log::debug;

use bindings::{
    Windows::Win32::Graphics::Gdi::{
        BeginPaint, EndPaint, PatBlt, BLACKNESS, HBRUSH, PAINTSTRUCT, WHITENESS,
    },
    Windows::Win32::System::Diagnostics::Debug::GetLastError,
    Windows::Win32::System::SystemServices::{GetModuleHandleW, LRESULT, PWSTR},
    Windows::Win32::UI::MenusAndResources::{HCURSOR, HICON},
    Windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowRect,
        PostQuitMessage, RegisterClassExW, TranslateMessage, CW_USEDEFAULT, HWND, LPARAM, MSG,
        WINDOW_EX_STYLE, WINDOW_STYLE, WM_ACTIVATEAPP, WM_CLOSE, WM_PAINT, WNDCLASSEXW,
        WNDCLASS_STYLES, WNDPROC, WPARAM, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    },
};

use widestring::WideCString;

trait PWSTRCreator {
    fn from_str(text: &'static str) -> PWSTR;
}
impl PWSTRCreator for PWSTR {
    fn from_str(text: &'static str) -> Self {
        Self(
            WideCString::from_str(text)
                .expect("convesion to wide string")
                .into_raw(),
        )
    }
}

fn debug_last_err() {
    unsafe { debug!("{:?}", GetLastError()) }
}

extern "system" fn window_event_handler(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_ACTIVATEAPP => {
            debug!("window activated");

            LRESULT(0)
        }
        WM_CLOSE => {
            unsafe {
                PostQuitMessage(0);
            }

            LRESULT(0)
        }
        WM_PAINT => {
            unsafe {
                let mut paint = PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut paint);
                //GetWindowRect(window, &mut rect);
                let x = paint.rcPaint.left;
                let y = paint.rcPaint.top;
                let w = paint.rcPaint.right - x;
                let h = paint.rcPaint.bottom - y;
                PatBlt(hdc, x, y, w, h, BLACKNESS);
                EndPaint(window, &mut paint);
            }

            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn main() -> windows::Result<()> {
    log::set_logger(&win_dbg_logger::DEBUGGER_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    let h_instance = unsafe { GetModuleHandleW(None) };

    let window_template = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: WNDCLASS_STYLES::default(),
        lpfnWndProc: Some(window_event_handler),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: HICON::default(),
        hCursor: HCURSOR::default(),
        hbrBackground: HBRUSH::default(),
        lpszMenuName: PWSTR::default(),
        lpszClassName: PWSTR::from_str("RMHClass"),
        hIconSm: HICON::default(),
    };

    unsafe {
        let success = RegisterClassExW(&window_template);
        debug_assert!(success != 0);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PWSTR::from_str("RMHClass"),
            "Rust made hero",
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            h_instance,
            std::ptr::null_mut(),
        );

        debug_assert!(hwnd.0 != 0);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, hwnd, 0, 0).as_bool() {
            TranslateMessage(&mut msg);
            DispatchMessageW(&mut msg);
        }
    }

    Ok(())
}
