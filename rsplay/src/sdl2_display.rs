use sdl2::{self, Sdl};

pub struct DisplayProp {
    pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub event_pump: sdl2::EventPump,
    width: u32,
    height: u32,
}

impl DisplayProp {
    pub fn update_yuv(
        &mut self,
        y_plane: &[u8],
        y_pitch: usize,
        u_plane: &[u8],
        u_pitch: usize,
        v_plane: &[u8],
        v_pitch: usize,
    ) {
        let binding = self.canvas.texture_creator();

        let mut texture: sdl2::render::Texture<'_> = binding
            .create_texture_streaming(
                sdl2::pixels::PixelFormatEnum::IYUV,
                self.width as u32,
                self.height as u32,
            )
            .map_err(|e| e.to_string())
            .unwrap()
            .into();

        texture
            .update_yuv(None, y_plane, y_pitch, u_plane, u_pitch, v_plane, v_pitch)
            .unwrap();

        self.canvas.clear();

        self.canvas
            .copy(
                &texture,
                None,
                sdl2::rect::Rect::new(0, 0, self.width as u32, self.height as u32),
            )
            .unwrap();
        self.canvas.present();
    }
}

pub fn display_init(sdl_context: Sdl, width: u32, height: u32) -> DisplayProp {
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

    DisplayProp {
        canvas,
        event_pump,
        width,
        height,
    }
}
