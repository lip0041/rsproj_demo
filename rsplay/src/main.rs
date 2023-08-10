#![allow(dead_code)]
#![allow(unused)]
mod sdl2_display;
mod utils;

use rsmpeg::avcodec::{AVCodec, AVCodecContext, AVCodecParserContext, AVPacket};
use rsmpeg::avformat::{AVFormatContextInput, AVIOContext, AVIOContextContainer, AVIOContextURL};
use rsmpeg::avutil::{
    self, AVDictionary, AVFrame, AVFrameWithImage, AVImage, AVSampleFormat, AVSamples,
};
use rsmpeg::error::RsmpegError;
use rsmpeg::ffi::{
    self, fileno, AVRational, AVSampleFormat_AV_SAMPLE_FMT_FLT, AV_INPUT_BUFFER_PADDING_SIZE,
    AV_TIME_BASE_Q,
};
use rsmpeg::swresample::{self, SwrContext};
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
    // wait for the registry update
    let rtsp_key = CString::new("rtsp_transport").unwrap();
    let rtsp_value = CString::new("tcp").unwrap();
    let options: AVDictionary = AVDictionary::new(&rtsp_key, &rtsp_value, 0);
    // let url = CString::new("rtsp://192.168.62.152:8554/stream").unwrap();

    let url = CString::new("rtmp://127.0.0.1:1935/live/test").unwrap();

    let mut input_format_context = AVFormatContextInput::open(&file)?;

    let video_stream_index = input_format_context
        .streams()
        .into_iter()
        .position(|stream| stream.codecpar().codec_type().is_video())
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

    let audio_stream_index = input_format_context
        .streams()
        .into_iter()
        .position(|stream| stream.codecpar().codec_type().is_audio())
        .unwrap();

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
    // video config
    let frame_rate = input_format_context
        .streams()
        .get(video_stream_index)
        .unwrap()
        .avg_frame_rate;

    let video_fps = (frame_rate.num as f64 / frame_rate.den as f64).ceil() as u64;
    let video_time_base = input_format_context
        .streams()
        .get(video_stream_index)
        .unwrap()
        .time_base;
    let time_base_q = AV_TIME_BASE_Q;
    // println!("fps: {video_fps}");
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

    // audio config
    let mut swr_context = SwrContext::new(
        audio_decode_context.channel_layout as u64,
        AVSampleFormat_AV_SAMPLE_FMT_FLT,
        audio_decode_context.sample_rate,
        audio_decode_context.channel_layout as u64,
        audio_decode_context.sample_fmt,
        audio_decode_context.sample_rate,
    )
    .unwrap();

    swr_context.init();

    let (_, swr_buffer_size) = AVSamples::get_buffer_size(
        audio_decode_context.ch_layout.nb_channels,
        audio_decode_context.frame_size,
        AVSampleFormat_AV_SAMPLE_FMT_FLT,
        0,
    )
    .unwrap();

    // write file
    let extract_h264 = false;
    let write_pcm = true;

    let mut audio_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("assets/decode/out.pcm")
        .unwrap();
    let mut video_file = fs::OpenOptions::new()
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

    let mut first_frame = true;
    let mut played_time: i64 = 0;
    let mut start = std::time::Instant::now();
    let mut running = true;
    let mut pause = false;
    // 'running: while running {
    // if pause {
    //     println!("play pause, delay 10ms");
    //     std::thread::sleep(std::time::Duration::from_millis(10));
    //     continue;
    // }
    'running: while let Some(packet) = input_format_context.read_packet().unwrap() {
        if packet.stream_index == audio_stream_index as i32 {
            audio_decode_context.send_packet(Some(&packet))?;
            loop {
                let frame = match audio_decode_context.receive_frame() {
                    Ok(frame) => frame,
                    Err(RsmpegError::DecoderDrainError) | Err(RsmpegError::DecoderFlushedError) => {
                        break;
                    }
                    Err(e) => return Err(e.into()),
                };
                let mut pcm_frame: AVFrame = AVFrame::new();
                pcm_frame.set_channel_layout(audio_decode_context.channel_layout as u64);
                pcm_frame.set_format(AVSampleFormat_AV_SAMPLE_FMT_FLT);
                pcm_frame.set_sample_rate(audio_decode_context.sample_rate);
                // println!(
                //     "frame config: {:}, {:}, {:}",
                //     frame.channel_layout, frame.format, frame.sample_rate
                // );

                // println!(
                //     "pcm config: {:}, {:}, {:}",
                //     pcm_frame.channel_layout, pcm_frame.format, pcm_frame.sample_rate
                // );

                swr_context.convert_frame(Some(&frame), &mut pcm_frame)?;
                // packed, just use data[0]
                if write_pcm {
                    audio_file
                        .write_all(unsafe {
                            slice::from_raw_parts(pcm_frame.data[0], swr_buffer_size as usize)
                        })
                        .unwrap();
                }
            }
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
                        video_file.write_all(start_code0).unwrap();
                        let h264_data: &[u8] =
                            unsafe { slice::from_raw_parts(nal_header, nalu_size as usize) };
                        video_file.write_all(h264_data).unwrap();
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

                        video_file
                            .write_all(h264_extradata_to_annexb(extra_data, extra_data_size))
                            .unwrap();
                    }
                    video_file.write_all(start_code1).unwrap();
                    let h264_data: &[u8] =
                        unsafe { slice::from_raw_parts(pdata, nalu_size as usize) };
                    video_file.write_all(h264_data).unwrap();

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
                if first_frame {
                    played_time = avutil::av_rescale_q(frame.pts, video_time_base, time_base_q);
                    let duration =
                        avutil::av_rescale_q(frame.duration, video_time_base, time_base_q);
                    played_time -= duration;
                    first_frame = false;
                }
                let pts_time =
                    rsmpeg::avutil::av_rescale_q(frame.pts, video_time_base, time_base_q);
                let now_time = start.elapsed().as_micros() as i64;
                // println!("{pts_time}, {now_time}");
                if (pts_time - played_time > now_time) {
                    let time: std::time::Duration = std::time::Duration::from_micros(
                        (pts_time - now_time - played_time) as u64,
                    );
                    // println!("need sleep {:?}", time.as_micros());
                    std::thread::sleep(time);
                }

                for event in display_prop.event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'running,
                        // Event::KeyDown {
                        //     keycode: Some(Keycode::Space),
                        //     ..
                        // } => {
                        //     println!("catch pause keydown");
                        //     pause = !pause;
                        //     break 'decoding;
                        // }
                        _ => {}
                    }
                }
            }
        }
        // }
    }
    println!("play time: {:?} ms", start.elapsed().as_millis());
    if !extract_h264 {
        fs::remove_file("assets/decode/out.h264").unwrap();
    }
    if !write_pcm {
        fs::remove_file("assets/decode/out.pcm").unwrap();
    }
    Ok(())
}
