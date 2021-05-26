fn main() {
    windows::build!(
        Windows::Win32::UI::MenusAndResources::{HCURSOR, HICON},
        Windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW,
            RegisterClassExW,
            GetMessageW,
            DefWindowProcW ,
            TranslateMessage,
            DispatchMessageW,
            WNDCLASSEXW,
            WNDCLASS_STYLES,
            CW_USEDEFAULT,
            WINDOW_EX_STYLE,
            WINDOW_STYLE,
            WS_OVERLAPPEDWINDOW,
            WS_VISIBLE,
            WNDPROC,
            HWND,
            LPARAM,
            WPARAM,
            MSG,
        },
        Windows::Win32::Graphics::Gdi::{HBRUSH},
        Windows::Win32::System::SystemServices::{GetModuleHandleW, PWSTR, LRESULT},
        Windows::Win32::System::Diagnostics::Debug::GetLastError,
    );
}
