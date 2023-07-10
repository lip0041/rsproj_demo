#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused)]
use rsmpeg::avcodec::{AVCodec, AVCodecContext, AVCodecParserContext, AVPacket};
use rsmpeg::avformat::AVFormatContextInput;
use rsmpeg::avutil::{AVFrame, AVFrameWithImage, AVImage};
use rsmpeg::error::RsmpegError;
use rsmpeg::ffi;
use rsmpeg::swscale::SwsContext;
use sdl2;
use sdl2::video::Window;
use std::{
    error::Error,
    ffi::{CStr, CString},
    fs::{self, File},
    io::prelude::*,
    slice,
};
// use rsmpeg::avutil;

fn dump_av_info(path: &CStr) -> Result<(), Box<dyn Error>> {
    let mut input_format_context = AVFormatContextInput::open(path)?;
    input_format_context.dump(0, path)?;
    Ok(())
}

fn decode_mp4_file(path: &str) -> Result<(), Box<dyn Error>> {
    // let input_context = AVFormatContextInput::open(path)?;
    // let media_type = ffi::AVMediaType_AVMEDIA_TYPE_VIDEO;
    // let (video_index, video_codec) = input_context.find_best_stream(media_type).unwrap().unwrap();
    // println!("{:?}", video_index);
    // decode mp4 to yuv
    let decoder = AVCodec::find_decoder(ffi::AVCodecID_AV_CODEC_ID_H264).unwrap();
    let mut decode_context = AVCodecContext::new(&decoder);
    decode_context.open(None).unwrap();

    let video_data = fs::read(path).unwrap();
    fs::create_dir_all("assets/decode").unwrap();

    let mut parse_offset = 0;
    let mut parse_context = AVCodecParserContext::find(decoder.id).unwrap();
    let mut packet = AVPacket::new();
    println!("video_data len: {:?}", video_data.len());
    while parse_offset < video_data.len() {
        let (get_packet, offset) = parse_context
            .parse_packet(
                &mut decode_context,
                &mut packet,
                &video_data[parse_offset..],
            )
            .unwrap();

        if get_packet {
            decode(&mut decode_context, Some(&packet), "assets/decode").unwrap();
        } else {
            println!("didn't get a packet, offset: {:?}", offset);
        }
        parse_offset += offset;
    }
    Ok(())
}

fn decode(
    decode_context: &mut AVCodecContext,
    packet: Option<&AVPacket>,
    out_dir: &str,
) -> Result<(), RsmpegError> {
    decode_context.send_packet(packet)?;
    println!("decode in");
    loop {
        let frame = match decode_context.receive_frame() {
            Ok(frame) => frame,
            Err(RsmpegError::DecoderDrainError) | Err(RsmpegError::DecoderFlushedError) => break,
            Err(e) => return Err(e.into()),
        };
        println!("{:?}, {:?}", frame, decode_context.frame_number);
    }
    Ok(())
}

fn main() -> Result<(), RsmpegError> {
    // dump_av_info(&CString::new("assets/bunny_1080p.mp4").unwrap()).unwrap();
    // decode_mp4_file("assets/bunny_1080p.mp4").unwrap();
    let file = CString::new("assets/bunny_1080p.mp4").unwrap();

    let mut input_format_context = AVFormatContextInput::open(&file)?;

    // input_format_context.dump(0, &file)?;

    let video_stream_index = input_format_context
        .streams()
        .into_iter()
        .position(|stream| stream.codecpar().codec_type().is_video())
        .unwrap();

    let mut decode_context = {
        let video_stream = input_format_context
            .streams()
            .get(video_stream_index)
            .unwrap();

        let decoder = AVCodec::find_decoder(video_stream.codecpar().codec_id).unwrap();

        let mut decode_context = AVCodecContext::new(&decoder);
        decode_context.apply_codecpar(&video_stream.codecpar());
        decode_context.open(None)?;
        decode_context
    };

    let image_buffer = AVImage::new(
        ffi::AVPixelFormat_AV_PIX_FMT_YUV420P,
        decode_context.width,
        decode_context.height,
        1,
    )
    .unwrap();

    let mut frame_rgb = AVFrameWithImage::new(image_buffer);

    let mut frame_buffer = AVFrame::new();

    let mut sws_context = SwsContext::get_context(
        decode_context.width,
        decode_context.height,
        decode_context.pix_fmt,
        decode_context.width,
        decode_context.height,
        ffi::AVPixelFormat_AV_PIX_FMT_YUV420P,
        ffi::SWS_FAST_BILINEAR,
    )
    .unwrap();

    let mut file = File::create("assets/decode/out.yuv").unwrap();

    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();
    // let audio_subsystem = sdl_context.audio().unwrap();
    // let event_pump = sdl_context.event_pump().unwrap();

    // let window = video_subsystem
    //     .window("rsplay", 960, 540)
    //     .position_centered()
    //     .opengl()
    //     .build()
    //     .map_err(|e| e.to_string())
    //     .unwrap();

    // let mut canvas = window
    //     .into_canvas()
    //     .build()
    //     .map_err(|e| e.to_string())
    //     .unwrap();

    // let texture_creator = canvas.texture_creator();
    // let mut texture = texture_creator
    //     .create_texture_streaming(sdl2::pixels::PixelFormatEnum::IYUV, 1920, 1080)
    //     .map_err(|e| e.to_string())
    //     .unwrap();

    // texture
    //     .with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //         let w = 256;
    //         let h = 256;

    //         for y in 0..h {
    //             for x in 0..w {
    //                 let offset = y * pitch + x;
    //                 buffer[offset] = 128;
    //             }
    //         }

    //         let y_size = pitch * h;

    //         for y in 0..h / 2 {
    //             for x in 0..w / 2 {
    //                 let u_offset = y_size + y * pitch / 2 + x;
    //                 let v_offset = y_size + (pitch / 2 * h / 2) + y * pitch / 2 + x;
    //                 buffer[u_offset] = (x * 2) as u8;
    //                 buffer[v_offset] = (y * 2) as u8;
    //             }
    //         }
    //     })
    //     .unwrap();

    // canvas.clear();

    // canvas
    //     .copy(&texture, None, sdl2::rect::Rect::new(100, 100, 1280, 720))
    //     .unwrap();
    // canvas.present();

    let mut i = 0;
    while let Some(packet) = input_format_context.read_packet().unwrap() {
        if packet.stream_index != video_stream_index as i32 {
            continue;
        }
        decode_context.send_packet(Some(&packet))?;

        loop {
            let frame = match decode_context.receive_frame() {
                Ok(frame) => frame,
                Err(RsmpegError::DecoderDrainError) | Err(RsmpegError::DecoderFlushedError) => {
                    break;
                }
                Err(e) => return Err(e.into()),
            };

            sws_context.scale_frame(&frame, 0, decode_context.height, &mut frame_rgb)?;

            i += 1;
            // file_save(&frame_rgb, &mut file);
            // let (y, u, v) = (frame_rgb.data[0], frame_rgb.data[1], frame_rgb.data[2]);
            // let size: usize = (frame_rgb.width * frame_rgb.height).try_into().unwrap();
            // let (y_buf, u_buf, v_buf) = (
            //     unsafe { slice::from_raw_parts(y, size as usize) },
            //     unsafe { slice::from_raw_parts(u, size / 4 as usize) },
            //     unsafe { slice::from_raw_parts(v, size / 4 as usize) },
            // );
            if i >= 2 {
                break;
            }
            file_save(&frame_rgb, &mut file);
        }
    }

    Ok(())
}

fn file_save(frame: &AVFrame, file: &mut File) {
    // Here we only capture the first layer of frame.
    let (y, u, v) = (frame.data[0], frame.data[1], frame.data[2]);
    let size: usize = (frame.width * frame.height).try_into().unwrap();
    let (y_buf, u_buf, v_buf) = (
        unsafe { slice::from_raw_parts(y, size as usize) },
        unsafe { slice::from_raw_parts(u, size / 4 as usize) },
        unsafe { slice::from_raw_parts(v, size / 4 as usize) },
    );

    file.write_all(&y_buf[0..]).unwrap();
    file.write_all(&u_buf[0..]).unwrap();
    file.write_all(&v_buf[0..]).unwrap();
}
