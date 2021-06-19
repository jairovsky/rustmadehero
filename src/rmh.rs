use log::debug;

type BuildPixelFn<'a> = &'a dyn Fn(u32,u32,u32,u32,) -> u32;

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