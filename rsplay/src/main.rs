#![allow(dead_code)]
#![allow(unused)]
mod sdl2_display;
mod utils;

use rsmpeg::avcodec::{AVCodec, AVCodecContext, AVCodecParserContext, AVPacket};
use rsmpeg::avformat::AVFormatContextInput;
use rsmpeg::avutil::{AVFrame, AVFrameWithImage, AVImage};
use rsmpeg::error::RsmpegError;
use rsmpeg::ffi::{self, fileno, AV_INPUT_BUFFER_PADDING_SIZE};
use rsmpeg::swscale::SwsContext;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::sys::PlaceOnTop;
use std::env;
use std::{
    error::Error,
    ffi::{CStr, CString},
    fs::{self, File},
    io::prelude::*,
    slice,
};

use sdl2_display::{display_init, AudioParams, VideoParams};
use utils::dump_av_info;
use utils::file_save_yuv420p;
use utils::h264_extradata_to_annexb;

fn main() -> Result<(), RsmpegError> {
    let args: Vec<String> = env::args().collect();
    let mut file_name = String::from("assets/luca_720p.mp4");
    if args.len() == 2 {
        file_name = "assets/".to_string();
        file_name += &args[1];
    }

    // dump_av_info(&CString::new(file_name.clone()).unwrap()).unwrap();

    let file = CString::new(file_name).unwrap();

    let mut input_format_context = AVFormatContextInput::open(&file)?;

    let video_stream_index = input_format_context
        .streams()
        .into_iter()
        .position(|stream| stream.codecpar().codec_type().is_video())
        .unwrap();

    let audio_stream_index = input_format_context
        .streams()
        .into_iter()
        .position(|stream| stream.codecpar().codec_type().is_audio())
        .unwrap();

    let mut video_decode_context = {
        let video_stream = input_format_context
            .streams()
            .get(video_stream_index)
            .unwrap();

        let decoder = AVCodec::find_decoder(video_stream.codecpar().codec_id).unwrap();

        let mut video_decode_context = AVCodecContext::new(&decoder);
        video_decode_context.apply_codecpar(&video_stream.codecpar())?;
        video_decode_context.open(None)?;
        video_decode_context
    };

    let mut audio_decode_context = {
        let audio_stream = input_format_context
            .streams()
            .get(audio_stream_index)
            .unwrap();
        let decoder = AVCodec::find_decoder(audio_stream.codecpar().codec_id).unwrap();

        let mut audio_decode_context = AVCodecContext::new(&decoder);
        audio_decode_context.apply_codecpar(&audio_stream.codecpar())?;
        audio_decode_context.open(None);
        audio_decode_context
    };

    let frame_rate = input_format_context
        .streams()
        .get(video_stream_index)
        .unwrap()
        .avg_frame_rate;

    let video_fps = (frame_rate.num as f64 / frame_rate.den as f64).ceil() as u64;
    println!("fps: {video_fps}");
    let image_buffer = AVImage::new(
        ffi::AVPixelFormat_AV_PIX_FMT_YUV420P,
        video_decode_context.width,
        video_decode_context.height,
        1,
    )
    .unwrap();

    let mut frame_rgb = AVFrameWithImage::new(image_buffer);

    let mut sws_context = SwsContext::get_context(
        video_decode_context.width,
        video_decode_context.height,
        video_decode_context.pix_fmt,
        video_decode_context.width,
        video_decode_context.height,
        ffi::AVPixelFormat_AV_PIX_FMT_YUV420P,
        ffi::SWS_FAST_BILINEAR,
    )
    .unwrap();

    // let mut file = File::create("assets/decode/out.h264").unwrap();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("assets/decode/out.h264")
        .unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_params: VideoParams = VideoParams::new(
        video_decode_context.width as u32,
        video_decode_context.height as u32,
    );

    let audio_params: AudioParams = AudioParams::new(
        audio_decode_context.sample_rate,
        audio_decode_context.ch_layout.nb_channels as u8,
        audio_decode_context.frame_size,
    );

    let mut display_prop = display_init(sdl_context, video_params, audio_params);

    let start_code0 = &[0u8, 0, 0, 1];
    let start_code1 = &[0u8, 0, 1];

    let mut i = 0;
    let extract_h264 = false;

    let start = std::time::Instant::now();
    'running: while let Some(packet) = input_format_context.read_packet().unwrap() {
        if packet.stream_index == audio_stream_index as i32 {
            // audio_decode_context.send_packet(Some(&packet))?;
        } else if packet.stream_index == video_stream_index as i32 {
            if extract_h264 {
                let mut pdata: *mut u8 = packet.data;
                let psize = packet.size;
                let pend: *mut u8 = pdata.wrapping_add(psize as usize);
                let mut cursize: i32 = 0;
                while cursize < psize {
                    let mut nalu_size: i32 = 0;
                    if unsafe { pend.offset_from(pdata) } < 4 {
                        break;
                    }
                    for i in 0..4 {
                        nalu_size <<= 8;
                        nalu_size |= unsafe { pdata.wrapping_add(i).read() } as i32;
                    }
                    pdata = pdata.wrapping_add(4 as usize);
                    cursize += 4;

                    let nal_header = pdata.wrapping_add(0 as usize);
                    let nal_type = unsafe { nal_header.read() } & 0x1f;
                    // println!("naltype: {}, nalsize: {}", nal_type, nalu_size);
                    if nal_type == 6 {
                        file.write_all(start_code0).unwrap();
                        let h264_data: &[u8] =
                            unsafe { slice::from_raw_parts(nal_header, nalu_size as usize) };
                        file.write_all(h264_data).unwrap();
                    } else if nal_type == 5 {
                        let extra_data = input_format_context
                            .streams()
                            .get(video_stream_index)
                            .unwrap()
                            .codecpar()
                            .extradata;
                        let extra_data_size = input_format_context
                            .streams()
                            .get(video_stream_index)
                            .unwrap()
                            .codecpar()
                            .extradata_size;

                        file.write_all(h264_extradata_to_annexb(extra_data, extra_data_size))
                            .unwrap();
                    }
                    file.write_all(start_code1).unwrap();
                    let h264_data: &[u8] =
                        unsafe { slice::from_raw_parts(pdata, nalu_size as usize) };
                    file.write_all(h264_data).unwrap();

                    pdata = pdata.wrapping_add(nalu_size as usize);
                    cursize += nalu_size;
                }
            }

            video_decode_context.send_packet(Some(&packet))?;

            loop {
                let frame = match video_decode_context.receive_frame() {
                    Ok(frame) => frame,
                    Err(RsmpegError::DecoderDrainError) | Err(RsmpegError::DecoderFlushedError) => {
                        break;
                    }
                    Err(e) => return Err(e.into()),
                };

                sws_context.scale_frame(&frame, 0, video_decode_context.height, &mut frame_rgb)?;

                i += 1;
                let (y, u, v) = (frame_rgb.data[0], frame_rgb.data[1], frame_rgb.data[2]);
                let size: usize = (frame_rgb.width * frame_rgb.height).try_into().unwrap();
                let (y_buf, u_buf, v_buf) = (
                    unsafe { slice::from_raw_parts(y, size as usize) },
                    unsafe { slice::from_raw_parts(u, size / 4 as usize) },
                    unsafe { slice::from_raw_parts(v, size / 4 as usize) },
                );

                display_prop.update_yuv(
                    y_buf,
                    video_decode_context.width as usize,
                    u_buf,
                    (video_decode_context.width / 2) as usize,
                    v_buf,
                    (video_decode_context.width / 2) as usize,
                );

                let time: std::time::Duration = std::time::Duration::from_millis(video_fps);
                std::thread::sleep(time);

                for event in display_prop.event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'running,
                        _ => {}
                    }
                }
            }
        }
    }
    println!("play time: {:?} ms", start.elapsed().as_millis());
    Ok(())
}
