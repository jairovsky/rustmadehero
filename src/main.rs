#![windows_subsystem = "windows"]

use bindings::{
    Windows::Win32::Graphics::Gdi::{HBRUSH, PAINTSTRUCT, BeginPaint, EndPaint, PatBlt, WHITENESS, BLACKNESS,},
    Windows::Win32::System::Diagnostics::Debug::GetLastError,
    Windows::Win32::System::SystemServices::{GetModuleHandleW, LRESULT, PWSTR},
    Windows::Win32::UI::MenusAndResources::{HCURSOR, HICON},
    Windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassExW,
        TranslateMessage, GetWindowRect, CW_USEDEFAULT, HWND, LPARAM, MSG, WINDOW_EX_STYLE, WINDOW_STYLE,
        WNDCLASSEXW, WNDCLASS_STYLES, WNDPROC, WPARAM, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WM_ACTIVATEAPP, WM_PAINT
    },
};

// learn how to use
use widestring::WideCString;

trait PWSTRCreator {
    fn from_str(text: &'static str) -> PWSTR;
}
impl PWSTRCreator for PWSTR {
    fn from_str(text: &'static str) -> PWSTR {
        Self(
            text.encode_utf16()
                .chain(::std::iter::once(0))
                .collect::<Vec<u16>>()
                .as_mut_ptr(),
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
        },
        WM_PAINT => {
            unsafe {
                let mut paint = PAINTSTRUCT::default();
                let mut hdc = BeginPaint(window, &mut paint);
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
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }
}

use log::{debug};

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
