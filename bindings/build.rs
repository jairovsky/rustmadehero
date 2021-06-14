fn main() {
    windows::build!(
        Windows::Win32::Graphics::Gdi::{
            BeginPaint, CreateDIBSection, EndPaint, PatBlt, BLACKNESS, HBRUSH, PAINTSTRUCT, WHITENESS, HDC,
            StretchDIBits, DeleteObject, CreateCompatibleDC, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
            DIB_RGB_COLORS, HBITMAP, RGBQUAD, SRCCOPY,
        },
        Windows::Win32::System::Diagnostics::Debug::GetLastError,
        Windows::Win32::System::SystemServices::{GetModuleHandleW, LoadLibraryW, GetProcAddress, LRESULT, PWSTR, HANDLE, HINSTANCE},
        Windows::Win32::UI::MenusAndResources::{HCURSOR, HICON},
        Windows::Win32::UI::XInput::*,
        Windows::Win32::Media::Audio::DirectMusic::*,
        Windows::Win32::Media::Multimedia::{WAVE_FORMAT_PCM},
        Windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
            GetWindowRect, PostQuitMessage, RegisterClassExW, TranslateMessage, SetWindowLongW, GetWindowLongW,
            SetWindowLongPtrW, GetWindowLongPtrW, PeekMessageW,
            CW_USEDEFAULT, HWND, LPARAM, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_ACTIVATEAPP, WM_CLOSE, WM_PAINT,
            WM_SIZE, WNDCLASSEXW, WNDCLASS_STYLES, WNDPROC, WPARAM, WS_OVERLAPPEDWINDOW, WS_VISIBLE, GWLP_USERDATA,
            WM_CREATE, CREATESTRUCTW, WM_DESTROY, PM_REMOVE, CS_VREDRAW, CS_HREDRAW, WM_KEYDOWN, WM_KEYUP,
            WM_SYSKEYDOWN, WM_SYSKEYUP
        },
        Windows::Win32::UI::DisplayDevices::RECT,
    );
}
