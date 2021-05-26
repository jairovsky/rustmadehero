use bindings::{
    Windows::Win32::Graphics::Gdi::HBRUSH,
    Windows::Win32::System::Diagnostics::Debug::GetLastError,
    Windows::Win32::System::SystemServices::{GetModuleHandleW, LRESULT, PWSTR},
    Windows::Win32::UI::MenusAndResources::{HCURSOR, HICON},
    Windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassExW,
        TranslateMessage, CW_USEDEFAULT, HWND, LPARAM, MSG, WINDOW_EX_STYLE, WINDOW_STYLE,
        WNDCLASSEXW, WNDCLASS_STYLES, WNDPROC, WPARAM, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
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
    unsafe { println!("{:?}", GetLastError()) }
}

extern "system" fn window_event_handler(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

fn main() -> windows::Result<()> {
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
            debug_last_err();
        }
    }

    Ok(())
}
