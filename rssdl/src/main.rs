use std::os::unix::prelude::FileExt;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::fs;
pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 640, 360)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    const WIDTH: usize = 640;
    const HEIGHT: usize = 360;
    const Y_SIZE: usize = WIDTH * HEIGHT;
    const U_SIZE: usize = WIDTH * HEIGHT / 4;
    const V_SIZE: usize = WIDTH * HEIGHT / 4;
    let file: fs::File = fs::File::open("assets/360p_frame.yuv").unwrap();
    let mut y_buf = [0u8; Y_SIZE];
    let mut u_buf = [0u8; U_SIZE];
    let mut v_buf = [0u8; V_SIZE];

    let y_bytes_read = file.read_at(&mut y_buf, 0).unwrap();
    let u_bytes_read = file.read_at(&mut u_buf, Y_SIZE as u64).unwrap();
    let v_bytes_read = file.read_at(&mut v_buf, (Y_SIZE + U_SIZE) as u64).unwrap();
    println!("{y_bytes_read}, {u_bytes_read}, {v_bytes_read}");
    // let texture = texture_creator.crea
    let rect = Rect::new(0, 0, 640, 360);
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::IYUV, 640, 360)
        .map_err(|e| e.to_string())?;

    texture
        .update_yuv(rect, &y_buf, WIDTH, &u_buf, WIDTH / 2, &v_buf, WIDTH / 2)
        .unwrap();

    // Create a U-V gradient
    // texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //     // `pitch` is the width of the Y component
    //     // The U and V components are half the width and height of Y

    //     let w = 640;
    //     let h = 360;

    //     // Set Y (constant)
    //     for y in 0..h {
    //         for x in 0..w {
    //             let offset = y * pitch + x;
    //             buffer[offset] = 128;
    //         }
    //     }

    //     let y_size = pitch * h;

    //     // Set U and V (X and Y)
    //     for y in 0..h / 2 {
    //         for x in 0..w / 2 {
    //             let u_offset = y_size + y * pitch / 2 + x;
    //             let v_offset = y_size + (pitch / 2 * h / 2) + y * pitch / 2 + x;
    //             buffer[u_offset] = (x * 2) as u8;
    //             buffer[v_offset] = (y * 2) as u8;
    //         }
    //     }
    // })?;

    canvas.clear();
    canvas.copy(&texture, None, Some(Rect::new(0, 0, 640, 360)))?;
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...
    }

    Ok(())
}
