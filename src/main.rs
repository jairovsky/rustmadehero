#![windows_subsystem = "windows"]

use log::debug;

use bindings::{
    Windows::Win32::Graphics::Gdi::{
        BeginPaint, CreateDIBSection, EndPaint, PatBlt, BLACKNESS, HBRUSH, PAINTSTRUCT, WHITENESS, HDC,
        StretchDIBits, DeleteObject, GetDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, RGBQUAD,
            SRCCOPY,
    },
    Windows::Win32::System::Diagnostics::Debug::GetLastError,
    Windows::Win32::System::SystemServices::{GetModuleHandleW, LRESULT, PWSTR, HANDLE},
    Windows::Win32::UI::MenusAndResources::{HCURSOR, HICON},
    Windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
        RegisterClassExW, TranslateMessage, SetWindowLongW, GetWindowLongW, CW_USEDEFAULT, HWND,
        LPARAM, MSG, WINDOW_EX_STYLE, WM_ACTIVATEAPP, WM_CLOSE, WM_PAINT, WM_SIZE,
        WNDCLASSEXW, WNDCLASS_STYLES, WPARAM, WS_OVERLAPPEDWINDOW, WS_VISIBLE, GWLP_USERDATA, WM_CREATE,
        CREATESTRUCTW, WM_DESTROY,
    },
    Windows::Win32::UI::DisplayDevices::RECT,
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

#[derive(Debug)]
struct Win32Game {
    running: bool,
    bitmap_info: BITMAPINFO,
    bitmap_handle: HBITMAP,
    bitmap_mem: *mut core::ffi::c_void,
    device_ctx: HDC,
}

fn win32_get_game(window: HWND) -> &'static mut Win32Game {
    unsafe { 
        let ptr = GetWindowLongW(window, GWLP_USERDATA) as *mut Win32Game;
        debug_assert!(!ptr.is_null());
        &mut *ptr
    }
}

macro_rules! u32_rgba {
    ( $r:expr, $g: expr, $b: expr, $a: expr ) => { 
        ($a << 24) + ($r << 16) + ($g << 8) + $b
    }
}

extern "system" fn window_event_handler(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE=> {
            unsafe {
                let create_struct: &CREATESTRUCTW = std::mem::transmute(lparam);
                SetWindowLongW(window, GWLP_USERDATA, create_struct.lpCreateParams as _);
            }
            LRESULT::default()
        }
        WM_ACTIVATEAPP => {
            debug!("window activated");

            LRESULT::default()
        }
        WM_SIZE => {
            debug!("resizing GDI buffer");

            let game = win32_get_game(window);
            if !game.bitmap_handle.is_null() {
                unsafe {DeleteObject(game.bitmap_handle)};
            }

            let mut rect = RECT::default();
            unsafe {
                GetClientRect(window, &mut rect);
            }

            game.bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: rect.right - rect.left,
                    biHeight: rect.bottom - rect.top,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB as u32,
                    biSizeImage: 0,
                    biClrImportant: 0,
                    biClrUsed: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                },
                bmiColors: [RGBQUAD::default()],
            };

            unsafe {
                game.bitmap_handle = CreateDIBSection(
                    game.device_ctx,
                    &game.bitmap_info,
                    DIB_RGB_COLORS,
                    &mut game.bitmap_mem,
                    HANDLE::default(),
                    0,
                );
                debug_assert!(!game.bitmap_handle.is_null());
            }
            LRESULT::default()
        }
        WM_CLOSE => {
            let game = win32_get_game(window);
            game.running = false;

            LRESULT::default()
        }
        WM_DESTROY => {
            let game = win32_get_game(window);
            game.running = false;

            LRESULT::default()
        }
        WM_PAINT => {
            let game=  win32_get_game(window);

            unsafe {
                let mut ptr = game.bitmap_mem as *mut u32;
                let n_pixels = game.bitmap_info.bmiHeader.biWidth * game.bitmap_info.bmiHeader.biHeight;
                for _ in 1..n_pixels {
                    *(ptr) = u32_rgba!(0x50, 0, 0, 0);
                    ptr = ptr.add(1);
                }
            }

            unsafe {
                let mut paint = PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut paint);
                let x = paint.rcPaint.left;
                let y = paint.rcPaint.top;
                let w = paint.rcPaint.right - x;
                let h = paint.rcPaint.bottom - y;
                debug_assert!(PatBlt(hdc, x, y, w, h, BLACKNESS).as_bool());
                let r = StretchDIBits(
                    game.device_ctx,
                    x,
                    y,
                    w,
                    h,
                    x,
                    y,
                    w,
                    h,
                    game.bitmap_mem,
                    &game.bitmap_info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
                debug_assert!(r != 0);
                debug!("blitted {} scanlines", r);
                EndPaint(window, &mut paint);
            }

            LRESULT::default()
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

        let mut game = Win32Game {
            running: true,
            bitmap_info: BITMAPINFO::default(),
            bitmap_handle: HBITMAP::default(),
            bitmap_mem: std::ptr::null_mut(),
            device_ctx: HDC::default(),
        };

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
            &mut game as *mut _ as _,
            // std::ptr::null_mut(),
        );

        debug_assert!(hwnd.0 != 0);

        game.device_ctx = GetDC(hwnd);

        let mut msg = MSG::default();
        while game.running {
            if GetMessageW(&mut msg, hwnd, 0, 0).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    Ok(())
}
