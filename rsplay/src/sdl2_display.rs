use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct DisplayProp<'a> {
    pub texture: sdl2::render::Texture<'a>,
    pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub event_pump: sdl2::EventPump,
}

pub fn display_init(width: u32, height: u32) -> DisplayProp<'static> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let event_pump: sdl2::EventPump = sdl_context.event_pump().unwrap();

    let window = video_subsystem
        .window("rsplay", width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let canvas: sdl2::render::Canvas<sdl2::video::Window> = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let texture= texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::IYUV, width, height)
        .map_err(|e| e.to_string())
        .unwrap().into();

    DisplayProp {
        texture,
        canvas,
        event_pump,
    }
}
