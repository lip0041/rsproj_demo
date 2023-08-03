#![allow(dead_code)]
#![allow(unused)]
use rsmpeg::avcodec::{AVCodec, AVCodecContext, AVCodecParserContext, AVPacket};
use rsmpeg::avformat::AVFormatContextInput;
use rsmpeg::avutil::{AVFrame, AVFrameWithImage, AVImage};
use rsmpeg::error::RsmpegError;
use rsmpeg::ffi::{self, AV_INPUT_BUFFER_PADDING_SIZE};
use rsmpeg::swscale::SwsContext;
use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::{
    error::Error,
    ffi::{CStr, CString},
    fs::{self, File},
    io::prelude::*,
    slice,
};

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
    let file = CString::new("assets/360p.mp4").unwrap();

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
        decode_context.apply_codecpar(&video_stream.codecpar())?;
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

    // let mut file = File::create("assets/decode/out.h264").unwrap();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("assets/decode/out.h264")
        .unwrap();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let window = video_subsystem
        .window("rsplay", 640, 360)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::IYUV, 640, 360)
        .map_err(|e| e.to_string())
        .unwrap();

    let start_code0 = &[0u8, 0, 0, 1];
    let start_code1 = &[0u8, 0, 1];

    // let out_header: &[u8] = h264_extradata_to_annexb(extra_data, extra_data_size);
    // println!("{:?}, len: {}", out_header, out_header.len());
    // return Ok(());
    let mut i = 0;
    'running: while let Some(packet) = input_format_context.read_packet().unwrap() {
        if packet.stream_index != video_stream_index as i32 {
            continue;
        } else {
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
                    let h264_data: &[u8] = unsafe {
                        slice::from_raw_parts(
                            // packet.data.wrapping_add(4 as usize),
                            nal_header,
                            nalu_size as usize,
                            // (packet.size - 4) as usize,
                        )
                    };
                    file.write_all(h264_data).unwrap();
                    // return Ok(());
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

                    file.write_all(start_code1).unwrap();
                    let h264_data: &[u8] =
                        unsafe { slice::from_raw_parts(pdata, nalu_size as usize) };
                    file.write_all(h264_data).unwrap();
                } else if nal_type == 1 {
                    file.write_all(start_code1).unwrap();
                    let h264_data: &[u8] =
                        unsafe { slice::from_raw_parts(pdata, nalu_size as usize) };
                    file.write_all(h264_data).unwrap();
                }
                pdata = pdata.wrapping_add(nalu_size as usize);
                cursize += nalu_size;
            }

            // let h264 =  unsafe { slice::from_raw_parts(packet.data, packet.size as usize)} ;
            // file.write_all(&h264[0..]).unwrap();
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
            let (y, u, v) = (frame_rgb.data[0], frame_rgb.data[1], frame_rgb.data[2]);
            let size: usize = (frame_rgb.width * frame_rgb.height).try_into().unwrap();
            let (y_buf, u_buf, v_buf) = (
                unsafe { slice::from_raw_parts(y, size as usize) },
                unsafe { slice::from_raw_parts(u, size / 4 as usize) },
                unsafe { slice::from_raw_parts(v, size / 4 as usize) },
            );
            texture
                .update_yuv(None, y_buf, 640, u_buf, 320, v_buf, 320)
                .unwrap();

            canvas.clear();

            canvas
                .copy(&texture, None, sdl2::rect::Rect::new(0, 0, 640, 360))
                .unwrap();
            canvas.present();
            // if i >= 2 {
            //     break;
            // }
            // file_save(&frame_rgb, &mut file);
            let time = std::time::Duration::from_millis(30);
            std::thread::sleep(time);

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
        }

    }
    Ok(())
}

fn h264_extradata_to_annexb(extra_data: *mut u8, extra_data_size: i32) -> &'static [u8] {
    let mut len = 0;
    let start_code0: [u8; 4] = [0, 0, 0, 1];
    let mut total_size = 0;
    let mut unit_size = 0;
    let padding = AV_INPUT_BUFFER_PADDING_SIZE;
    let mut pextra_data = extra_data.wrapping_add(4 as usize); // skip the fixed header_data
    let pout: *mut u8 = [0u8; 64].as_mut_ptr();
    pextra_data = pextra_data.wrapping_add(1 as usize); // skip the version info ? skip ff

    let mut sps_uint_num = unsafe { pextra_data.read() } & 0x1f;
    pextra_data = pextra_data.wrapping_add(1 as usize);

    while sps_uint_num != 0 {
        unit_size = unsafe {
            (pextra_data.read() as i32) << 8 | pextra_data.wrapping_add(1 as usize).read() as i32
        } as i32;
        // println!("sps_num: {sps_uint_num}, sps_length: {unit_size}");
        pextra_data = pextra_data.wrapping_add(2); // get the sps length
        total_size += unit_size + start_code0.len() as i32;

        // if total_size > INT_MAX - padding
        //     || unsafe {
        //         pextra_data
        //             .wrapping_add(unit_size as usize)
        //             .offset_from(extra_data.wrapping_add(extra_data_size as usize))
        //             < 0
        //     }
        // {
        //     return;
        // }

        unsafe { pout.copy_from(start_code0.as_ptr(), 4 as usize) };

        let ptout = pout.wrapping_add(4 as usize);
        unsafe { ptout.copy_from(pextra_data, unit_size as usize) };
        pextra_data = pextra_data.wrapping_add(unit_size as usize);
        sps_uint_num -= 1;
    }
    len = unit_size + start_code0.len() as i32;

    let mut pps_uint_num = unsafe { pextra_data.read() } & 0x1f;
    pextra_data = pextra_data.wrapping_add(1 as usize);
    let ppout = pout.wrapping_add(len as usize);
    while pps_uint_num != 0 {
        unit_size = unsafe {
            (pextra_data.read() as i32) << 8 | pextra_data.wrapping_add(1 as usize).read() as i32
        } as i32;
        pextra_data = pextra_data.wrapping_add(2); // get the pps length
        total_size += unit_size + start_code0.len() as i32;

        // if total_size > INT_MAX - padding
        //     || unsafe {
        //         pextra_data
        //             .wrapping_add(unit_size as usize)
        //             .offset_from(extra_data.wrapping_add(extra_data_size as usize))
        //             < 0
        //     }
        // {
        //     return;
        // }

        // return &[0u8; 2];
        unsafe { ppout.copy_from(start_code0.as_ptr(), 4 as usize) };
        let ptout = ppout.wrapping_add(4 as usize);
        unsafe { ptout.copy_from(pextra_data, unit_size as usize) };
        pextra_data = pextra_data.wrapping_add(unit_size as usize);

        pps_uint_num -= 1;
    }

    len += unit_size + 4;
    // let oot = unsafe { slice::from_raw_parts(pout, len as usize)};
    // println!("{:?}, len: {}", oot, oot.len());
    unsafe { slice::from_raw_parts(pout, total_size as usize) }
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
