#![windows_subsystem = "windows"]

use log::debug;

use bindings::{Windows::Win32::Graphics::Gdi::{
        BeginPaint, CreateDIBSection, EndPaint, PatBlt, BLACKNESS, HBRUSH, PAINTSTRUCT, WHITENESS, HDC,
        StretchDIBits, DeleteObject, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        HBITMAP, RGBQUAD, SRCCOPY,
    }, Windows::Win32::Media::Audio::DirectMusic::{
        IDirectSound, IDirectSoundBuffer, DSBCAPS_PRIMARYBUFFER, DSBCAPS_GLOBALFOCUS, DSBLOCK_ENTIREBUFFER, DSBLOCK_FROMWRITECURSOR,
        DSBUFFERDESC, DirectSoundCreate, DSBPLAY_LOOPING, DSBSTATUS_LOOPING
    },
    Windows::Win32::Media::Multimedia::{ WAVEFORMATEX, WAVE_FORMAT_PCM }, Windows::Win32::System::Diagnostics::Debug::GetLastError,
    Windows::Win32::{Media::Audio::DirectMusic::DSSCL_PRIORITY, System::SystemServices::{
        GetModuleHandleW, LoadLibraryW, GetProcAddress, LRESULT, PWSTR, HANDLE
    }}, Windows::Win32::UI::DisplayDevices::RECT, Windows::Win32::UI::MenusAndResources::{HCURSOR, HICON}, Windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
        RegisterClassExW, TranslateMessage, SetWindowLongPtrW, GetWindowLongPtrW, PeekMessageW, CW_USEDEFAULT, HWND,
        LPARAM, MSG, WINDOW_EX_STYLE, WM_ACTIVATEAPP, WM_CLOSE, WM_PAINT, WM_SIZE,
        WNDCLASSEXW, WNDCLASS_STYLES, WPARAM, WS_OVERLAPPEDWINDOW, WS_VISIBLE, GWLP_USERDATA, WM_CREATE,
        CREATESTRUCTW, WM_DESTROY, PM_REMOVE, CS_HREDRAW, CS_VREDRAW, WM_KEYDOWN, WM_KEYUP
    }, Windows::Win32::UI::XInput::*};

use windows::{HRESULT, Guid, IUnknown};

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

trait WaveFormatExCreator {
    fn new_pcm(n_channels: u16, n_bits_p_sample: u16, n_samples_p_sec: u16) -> WAVEFORMATEX;
}
impl WaveFormatExCreator for WAVEFORMATEX {
    fn new_pcm(n_channels: u16, n_bits_p_sample: u16, n_samples_p_sec: u16) -> Self {
        let block_align = (n_channels * n_bits_p_sample) / 8;
        Self {
            wFormatTag: WAVE_FORMAT_PCM as u16,
            nChannels : n_channels,
            nSamplesPerSec : n_samples_p_sec as u32,
            wBitsPerSample : n_bits_p_sample,
            nBlockAlign : block_align,
            nAvgBytesPerSec : n_samples_p_sec as u32 * block_align as u32,
            cbSize: 0
        }
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

type DirectSoundCreateFn = extern "C" fn(
    pcguiddevice: *const Guid, 
    ppds: *mut Option<IDirectSound>, 
    punkouter: *const std::ffi::c_void,
) -> HRESULT;

type XInputGetStateFn = extern "C" fn(u32, *mut XINPUT_STATE) -> u32;
struct XInput {
   get_state: XInputGetStateFn
}

struct SoundParams {
    bits_per_sample: u16,
    n_channels: u16,
    n_samples_per_sec: u16,
    buf_size_seconds: u16
}

impl SoundParams {
    fn buf_size_bytes(&self) -> u32 {
        (self.n_channels as u32 * 
        self.bits_per_sample as u32 *
        self.n_samples_per_sec as u32 *
        self.buf_size_seconds as u32) / 8
    }

    fn bytes_per_sample(&self) -> u32 {
        (self.bits_per_sample * self.n_channels / 8) as u32
    }
}

struct Pad {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

struct Win32Game {
    running: bool,
    bitmap_info: BITMAPINFO,
    bitmap_mem: std::vec::Vec::<u32>,
    window: HWND,
    window_width: u32,
    window_height: u32,
    xinput: Option<XInput>,
    pad1: Pad,
    dsound_buffer: Option<IDirectSoundBuffer>,
    dsound: Option<IDirectSound>, //necessary to hold this ref, otherwise the buffer gets deallocated
    sound_params: SoundParams,
    sound_sample_idx: u32,
    sound_playing: bool,
}

fn win32_get_game(window: HWND) -> &'static mut Win32Game {
    unsafe { 
        let ptr = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut Win32Game;
        debug_assert!(!ptr.is_null());
        &mut *ptr
    }
}

fn win32_load_xinput(game: &mut Win32Game) {

    let versions = ["xinput.dll", "xinput1_4.dll", "xinput1_3.dll"];

    for v in &versions {

        let dll = unsafe {LoadLibraryW(*v)};
        if !dll.is_null() {

            debug!("loaded xinput {}", *v);
            unsafe {
                if let Some(addr) = GetProcAddress(dll, "XInputGetState") {
                    game.xinput = Some(XInput {
                        get_state: std::mem::transmute_copy(&addr)
                    });
                }
            }

            return
        }
    }
}

fn win32_init_dsound(game: &mut Win32Game) {

    let dll = unsafe {LoadLibraryW("dsound.dll")};
    if !dll.is_null() {
        debug!("loaded dsound");
        unsafe {
            if let Some(addr) = GetProcAddress(dll, "DirectSoundCreate") {
                let direct_sound_create: DirectSoundCreateFn = std::mem::transmute_copy(&addr);
                debug_assert!(
                    direct_sound_create(std::ptr::null_mut(), &mut game.dsound, std::ptr::null_mut()).is_ok()
                );
                debug_assert!(
                    game.dsound.is_some()
                );

                if let Some(dsound) = &game.dsound {

                    debug_assert!(
                        dsound.SetCooperativeLevel(game.window, DSSCL_PRIORITY).is_ok()
                    );

                    let mut wave_format = WAVEFORMATEX::new_pcm(
                        game.sound_params.n_channels,
                        game.sound_params.bits_per_sample,
                        game.sound_params.n_samples_per_sec,
                    );

                    let buffer_desc = &mut DSBUFFERDESC {
                        dwSize: std::mem::size_of::<DSBUFFERDESC>() as u32,
                        dwFlags: DSBCAPS_PRIMARYBUFFER,
                        ..Default::default()
                    };
                    let mut dsound_buffer: Option<IDirectSoundBuffer> = None;
                    debug_assert!(
                        dsound.CreateSoundBuffer(buffer_desc, &mut dsound_buffer, None).is_ok()
                    );

                    if let Some(dsound_buffer) = dsound_buffer {
                        debug_assert!(
                            dsound_buffer.SetFormat(&mut wave_format).is_ok()
                        );
                    }

                    let sec_buffer_desc = &mut DSBUFFERDESC {
                        dwSize: std::mem::size_of::<DSBUFFERDESC>() as u32,
                        dwBufferBytes: game.sound_params.buf_size_bytes(),
                        lpwfxFormat: &mut wave_format,
                        dwFlags: DSBCAPS_GLOBALFOCUS,
                        ..Default::default()
                    };
                    debug_assert!(
                        dsound.CreateSoundBuffer(sec_buffer_desc, &mut game.dsound_buffer, None).is_ok()
                    );
                    
                    if let Some(buf) = &game.dsound_buffer {

                        let mut part1ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                        let mut part1size = 0u32;

                        debug_assert!(buf.Lock(
                            0,
                            0,
                            &mut part1ptr,
                            &mut part1size,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                            DSBLOCK_ENTIREBUFFER | DSBLOCK_FROMWRITECURSOR
                        ).is_ok());

                        let mut part1ptr_iter = part1ptr as *mut u32;
                        for i in (0..part1size).step_by(std::mem::size_of::<u32>()) {
                            *part1ptr_iter = 0;
                            part1ptr_iter = part1ptr_iter.add(1);
                        }

                        debug_assert!(
                            buf.Unlock(part1ptr, part1size, std::ptr::null_mut(), 0).is_ok()
                        );
                    }
                }
            }
        }
    }
}

fn win32_get_pad_input(game: &mut Win32Game) {
    if let Some(xinput) = &mut game.xinput {
        let mut state = XINPUT_STATE::default();
        (xinput.get_state)(0, &mut state);

        game.pad1.up = (state.Gamepad.wButtons & XINPUT_GAMEPAD_DPAD_UP as u16) != 0;
        game.pad1.down = (state.Gamepad.wButtons & XINPUT_GAMEPAD_DPAD_DOWN as u16) != 0;
        game.pad1.left = (state.Gamepad.wButtons & XINPUT_GAMEPAD_DPAD_LEFT as u16) != 0;
        game.pad1.right = (state.Gamepad.wButtons & XINPUT_GAMEPAD_DPAD_RIGHT as u16) != 0;
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
        debug!("stretchdibits: blitted {} lines", r);
        ReleaseDC(game.window, hdc);
    }
}

fn win32_render_weird_gradient(game: &mut Win32Game, xoffset: i32, yoffset: i32) {

    for y in (0..game.bitmap_info.bmiHeader.biHeight) {
        for x in 0..game.bitmap_info.bmiHeader.biWidth {
            let idx = (y * game.bitmap_info.bmiHeader.biWidth + x) as usize;
            if (x-xoffset) % 100 == 0 || (y-yoffset) % 100 == 0 {
                game.bitmap_mem[idx]= u32_rgba!( 0, 255, 0, 0);
            } else {
                game.bitmap_mem[idx]= u32_rgba!( 0, 0, 0, 0);
            }
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
                SetWindowLongPtrW(window, GWLP_USERDATA, create_struct.lpCreateParams as _);
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
            xinput: None,
            pad1: Pad{
                up: false,
                down: false,
                left: false,
                right: false,
            },
            dsound: None,
            dsound_buffer: None,
            sound_params: SoundParams {
                bits_per_sample: 16,
                n_channels: 2,
                n_samples_per_sec: 48000,
                buf_size_seconds: 1,
            },
            sound_sample_idx: 0,
            sound_playing: false,
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

        let mut x_offset = 10;
        let mut y_offset = 10;
        let mut square_wave_sign = 1;
        let mut square_wave_sample_counter = 0;
        let mut square_wave_len = 30;

        win32_load_xinput(&mut game);

        win32_init_dsound(&mut game);

        let mut frame_timer = std::time::Instant::now();
        let mut frame_timer_diff = 0u128;

        let mut msg = MSG::default();

        while game.running {
            while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            win32_get_pad_input(&mut game);

            if game.pad1.up {
                // y_offset -= 5;
                square_wave_len += 1;
            }
            if game.pad1.down {
                // y_offset += 5;
                square_wave_len -= 1;
            }
            if game.pad1.left {
                x_offset -= 5;
            }
            if game.pad1.right {
                x_offset += 5;
            }

            win32_render_weird_gradient(&mut game, x_offset, y_offset);
            win32_render(&game);

            frame_timer_diff = frame_timer.elapsed().as_millis();
            debug!("loop time {}ms", frame_timer_diff);
            frame_timer = std::time::Instant::now();

            if let Some(buf) = &game.dsound_buffer {
                
                let mut play_cur = 0u32;
                let mut write_cur = 0u32;
                buf.GetCurrentPosition(&mut play_cur, &mut write_cur);

                let mut part1ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                let mut part2ptr: *mut std::ffi::c_void = std::ptr::null_mut();
                let mut part1size = 0u32;
                let mut part2size = 0u32;

                let running_bytes = game.sound_sample_idx * game.sound_params.bytes_per_sample();

                debug!(
                    "diff between write_cur and own byte tracker {} {} {}",
                    write_cur,
                    running_bytes,
                    running_bytes as i32 - write_cur as i32 ,
                );

                let mut byte_to_lock = game.sound_sample_idx * game.sound_params.bytes_per_sample();
                let mut bytes_to_write = game.sound_params.buf_size_bytes()
                                        / (game.sound_params.buf_size_seconds as u32 * 1000)
                                        * frame_timer_diff as u32 ; 

                // in case we suffer some latency, increase the amount of bytes to write
                // to catch up with dsound's write cursor
                let will_wrap = byte_to_lock + bytes_to_write > game.sound_params.buf_size_bytes();
                if !will_wrap && byte_to_lock < write_cur {
                    bytes_to_write += (write_cur - byte_to_lock) * 2;
                }

                // preventing overflow if the game loop hangs for whatever reason,
                // e.g. if some Windows event makes PeekMessage wait for too long.
                if bytes_to_write > game.sound_params.buf_size_bytes() {
                    bytes_to_write = game.sound_params.buf_size_bytes();
                }

                debug_assert!(
                    buf.Lock(
                        byte_to_lock,
                        bytes_to_write,
                        &mut part1ptr,
                        &mut part1size,
                        &mut part2ptr,
                        &mut part2size,
                        0
                    ).is_ok()
                );

                let mut part1ptrwalk = part1ptr as *mut i16;
                for sample_idx in (0..part1size).step_by(game.sound_params.bytes_per_sample() as usize) {
                    *part1ptrwalk = square_wave_sign * 2000;
                    part1ptrwalk=part1ptrwalk.add(1);
                    *part1ptrwalk = square_wave_sign * 2000;
                    part1ptrwalk=part1ptrwalk.add(1);
                    square_wave_sample_counter += 1;
                    game.sound_sample_idx += 1;
                    if square_wave_sample_counter >= square_wave_len {
                        square_wave_sample_counter = 0;
                        square_wave_sign *= -1;
                    }
                    game.sound_sample_idx %= game.sound_params.buf_size_bytes() / game.sound_params.bytes_per_sample();
                }

                let mut part2ptrwalk = part2ptr as *mut i16;
                for sample_idx in (0..part2size).step_by(game.sound_params.bytes_per_sample() as usize) {
                    *part2ptrwalk = square_wave_sign * 2000;
                    part2ptrwalk=part2ptrwalk.add(1);
                    *part2ptrwalk = square_wave_sign * 2000;
                    part2ptrwalk=part2ptrwalk.add(1);
                    square_wave_sample_counter += 1;
                    game.sound_sample_idx += 1;
                    if square_wave_sample_counter >= square_wave_len {
                        square_wave_sample_counter = 0;
                        square_wave_sign *= -1;
                    }
                    game.sound_sample_idx %= game.sound_params.buf_size_bytes() / game.sound_params.bytes_per_sample();
                }

                debug_assert!(
                    buf.Unlock(part1ptr, part1size, part2ptr, part2size).is_ok()
                );

                if !game.sound_playing {
                    debug_assert!(
                        buf.Play(0, 0, DSBPLAY_LOOPING).is_ok()
                    );
                    game.sound_playing = true;
                }
            }
        }
    }

    Ok(())
}
