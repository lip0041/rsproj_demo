use sdl2::{
    self,
    audio::{AudioCallback, AudioSpecDesired},
    Sdl,
};

pub struct DisplayProp {
    pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub event_pump: sdl2::EventPump,
    width: u32,
    height: u32,
}

pub struct VideoParams {
    width: u32,
    height: u32,
}

impl VideoParams {
    pub fn new(width: u32, height: u32) -> Self {
        VideoParams { width, height }
    }
}

pub struct AudioParams {
    freq: i32,
    channels: u8,
    samples: i32,
}

impl AudioParams {
    pub fn new(freq: i32, channels: u8, samples: i32) -> Self {
        AudioParams {
            freq,
            channels,
            samples,
        }
    }
}

struct Sound {
    data: Vec<u8>,
    volume: f32,
    pos: usize,
}

impl AudioCallback for Sound {
    type Channel = u8;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for dst in out.iter_mut() {
            let pre_scale = *self.data.get(self.pos).unwrap_or(&128);
            let scaled_signed_float = (pre_scale as f32 - 128.0) * self.volume;
            let scaled = (scaled_signed_float + 128.0) as u8;
            *dst = scaled;
            self.pos += 1;
        }
    }
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

pub fn display_init(
    sdl_context: Sdl,
    video_params: VideoParams,
    audio_params: AudioParams,
) -> DisplayProp {
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let event_pump: sdl2::EventPump = sdl_context.event_pump().unwrap();
    let width = video_params.width;
    let height = video_params.height;
    let desired_spec = AudioSpecDesired {
        freq: Some(audio_params.freq),
        channels: Some(audio_params.channels),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            let data: Vec<u8> = Vec::new();
            Sound {
                data,
                volume: 0.5,
                pos: 0,
            }
        })
        .unwrap();

    device.resume();

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
