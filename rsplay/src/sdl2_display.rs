use std::fs::{self, File};
use std::io::Write;
use std::sync::{Arc, Mutex};

use rand::{self, Rng};
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
    device: sdl2::audio::AudioDevice<Sound>,
    audio_data: Arc<Mutex<Vec<u8>>>,
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
    data: Arc<Mutex<Vec<u8>>>,
    volume: f32,
    samples: usize,
    audio_file: File,
}

impl AudioCallback for Sound {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        let mut binding = self.data.lock().unwrap();
        let audio_data = binding.as_slice();
        println!("after callback {:?}, {:?}", out.len(), audio_data.len());
        if (audio_data.len() != 32768) {
            return;
        }
        println!("audio_data len: {:?}", audio_data.len());
        let mut f32_data = unsafe {
            std::mem::transmute::<[u8; 32768], [f32; 8192]>(audio_data.try_into().unwrap())
        };

        // self.audio_file.write_all(audio_data).unwrap();
        let mut index = 0;
        for dst in out.iter_mut() {
            let pre_scale = f32_data.get(index).unwrap();
            *dst = *pre_scale * self.volume;
            index += 1;
        }
        // out.as_mut().swap_with_slice(f32_data.as_mut());

        // use self::rand::{thread_rng, Rng};
        // let mut rng = thread_rng();

        // // Generate white noise
        // for x in out.iter_mut() {
        //     let y: f32 = rng.gen();
        //     *x = (y * self.volume);
        // }
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

    pub fn updata_pcm(&mut self, data: &[u8]) {
        // self.device.pause();
        {
            let mut lock = self.device.lock();
            if (*lock).data.lock().unwrap().len() >= 32768 {
                (*lock).data.lock().unwrap().clear();
            }
            (*lock).data.lock().unwrap().extend_from_slice(data);
        }
        // self.device.resume();
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
    let desired_spec: AudioSpecDesired = AudioSpecDesired {
        freq: Some(audio_params.freq),
        channels: Some(audio_params.channels),
        samples: Some(4096),
    };

    let audio_data: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    println!(
        "after new {:?}, {:?}, {:?}",
        desired_spec.freq, desired_spec.channels, desired_spec.samples
    );
    let device: sdl2::audio::AudioDevice<Sound> = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            println!("{:?}", spec);

            let mut audio_file = fs::File::create("assets/decode/sdlpcm.h264").unwrap();
            Sound {
                data: audio_data.clone(),
                volume: 0.8,
                samples: spec.samples as usize,
                audio_file,
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
        device,
        audio_data,
    }
}
