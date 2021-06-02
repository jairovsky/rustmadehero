#![windows_subsystem = "windows"]

use log::debug;

use bindings::{
    Windows::Win32::Graphics::Gdi::{
        BeginPaint, CreateDIBSection, EndPaint, PatBlt, BLACKNESS, HBRUSH, PAINTSTRUCT, WHITENESS, HDC,
        StretchDIBits, DeleteObject, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, RGBQUAD,
            SRCCOPY,
    },
    Windows::Win32::System::Diagnostics::Debug::GetLastError,
    Windows::Win32::System::SystemServices::{GetModuleHandleW, LRESULT, PWSTR, HANDLE},
    Windows::Win32::UI::MenusAndResources::{HCURSOR, HICON},
    Windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
        RegisterClassExW, TranslateMessage, SetWindowLongW, GetWindowLongW, PeekMessageW, CW_USEDEFAULT, HWND,
        LPARAM, MSG, WINDOW_EX_STYLE, WM_ACTIVATEAPP, WM_CLOSE, WM_PAINT, WM_SIZE,
        WNDCLASSEXW, WNDCLASS_STYLES, WPARAM, WS_OVERLAPPEDWINDOW, WS_VISIBLE, GWLP_USERDATA, WM_CREATE,
        CREATESTRUCTW, WM_DESTROY, PM_REMOVE, CS_HREDRAW, CS_VREDRAW
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

macro_rules! u32_rgba {
    ( $r:expr, $g: expr, $b: expr, $a: expr ) => { 
        ($a << 24) + ($r << 16) + ($g << 8) + $b
    }
}

#[derive(Debug)]
struct Win32Game {
    running: bool,
    bitmap_info: BITMAPINFO,
    bitmap_mem: std::vec::Vec::<u32>,
    window: HWND,
    window_width: u32,
    window_height: u32
}

fn win32_get_game(window: HWND) -> &'static mut Win32Game {
    unsafe { 
        let ptr = GetWindowLongW(window, GWLP_USERDATA) as *mut Win32Game;
        debug_assert!(!ptr.is_null());
        &mut *ptr
    }
}

fn win32_render(game: &Win32Game) {
    unsafe {
        let hdc = GetDC(game.window);
        let r = StretchDIBits(
            hdc,
            0,
            game.window_height as i32,
            game.window_width as i32,
            -(game.window_height as i32),
            0,
            0,
            game.bitmap_info.bmiHeader.biWidth,
            game.bitmap_info.bmiHeader.biHeight,
            &(game.bitmap_mem[0]) as *const u32 as *const std::ffi::c_void,
            &game.bitmap_info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
        debug_assert!(r > 0);
        ReleaseDC(game.window, hdc);
    }
}

fn win32_render_weird_gradient(game: &mut Win32Game, xoffset: i32, yoffset: i32) {

    for y in (0..game.bitmap_info.bmiHeader.biHeight) {
        for x in 0..game.bitmap_info.bmiHeader.biWidth {
            let idx = (y * game.bitmap_info.bmiHeader.biWidth + x) as usize;
            game.bitmap_mem[idx]= u32_rgba!(
                0,
                ((y - yoffset) as u32 & 0xff),
                ((x - xoffset) as u32 & 0xff),
                0
            );
        }
    }
}

fn win32_resize_bitmap_buffer(game: &mut Win32Game, width: u32, height: u32) {

    game.bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: height as i32,
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

    let bitmap_size_pixels = game.bitmap_info.bmiHeader.biWidth * game.bitmap_info.bmiHeader.biHeight;

    game.bitmap_mem = vec![0; bitmap_size_pixels as usize];
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

            let game = win32_get_game(window);

            let mut rect= RECT::default();
            unsafe { GetClientRect(window, &mut rect) };

            game.window_width = (rect.right - rect.left) as u32;
            game.window_height = (rect.bottom - rect.top) as u32;

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

            let mut paint = PAINTSTRUCT::default();
            unsafe {
                let hdc = BeginPaint(window, &mut paint);
                PatBlt(hdc, 0, 0, game.bitmap_info.bmiHeader.biWidth,game.bitmap_info.bmiHeader.biHeight, BLACKNESS);
                win32_render(game);
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
        style: CS_HREDRAW | CS_VREDRAW,
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
            bitmap_mem: std::vec::Vec::new(),
            window: HWND::default(),
            window_width: 1280,
            window_height: 720,
        };

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PWSTR::from_str("RMHClass"),
            "Rust made hero",
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            game.window_width as i32,
            game.window_height as i32,
            None,
            None,
            h_instance,
            &mut game as *mut _ as _,
        );


        debug_assert!(hwnd.0 != 0);

        game.window = hwnd;

        win32_resize_bitmap_buffer(&mut game, 1280, 720);

        let mut msg = MSG::default();
        let mut x_offset = 10;
        let mut y_offset = 10;
        while game.running {
            while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            x_offset += 1;
            y_offset += 1;
            win32_render_weird_gradient(&mut game, x_offset, y_offset);
            win32_render(&game);
        }
    }

    Ok(())
}
