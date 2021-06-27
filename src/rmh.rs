use log::debug;

type BuildPixelFn<'a> = &'a dyn Fn(u32,u32,u32,u32,) -> u32;

pub struct Pad {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

pub struct GameState {
    pub x_offset: i32,
    pub y_offset: i32,
    pub sine_wave_half_len: i32,
}

pub fn render_gfx(
    mem: &mut Vec<u32>,
    w: i32,
    h: i32,
    x_offset: i32,
    y_offset: i32,
    build_pixel: BuildPixelFn
) {
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if (x-x_offset) % 100 == 0 || (y-y_offset) % 100 == 0 {
                mem[idx]= build_pixel(255, 0, 255, 0);
            } else {
                mem[idx]= build_pixel(255, 0, 0, 0);
            }
        }
    }
}

pub fn render_audio(
    buf: &mut Vec<i16>,
    sine_wave_half_len: i32,
    t_sine: &mut i32,
) {
    let amplitude = 2000;
    for i in (0..buf.len()).step_by(2) {
        let radians = (
            (std::f32::consts::PI * 2.0)
            / (sine_wave_half_len * 2) as f32
            * (*t_sine) as f32
        ) as f32;
        let sample = (radians.sin() * amplitude as f32) as i16;
        buf[i] = sample;
        buf[i+1] = sample;
        *t_sine += 1;
        if *t_sine >= sine_wave_half_len * 2 {
            *t_sine = 0;
        }
    }
}

pub fn update_state(
    state: &mut GameState,
    pad: &Pad,
) {
    if pad.up {
        // state.y_offset -= 5;
        state.sine_wave_half_len += 1;
    }
    if pad.down {
        // state.y_offset += 5;
        state.sine_wave_half_len -= 1;
    }
    if pad.left {
        state.x_offset -= 5;
    }
    if pad.right {
        state.x_offset += 5;
    }
}