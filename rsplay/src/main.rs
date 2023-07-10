use rsmpeg::avcodec::{AVCodec, AVCodecContext, AVCodecParserContext, AVPacket};
use rsmpeg::avformat::AVFormatContextInput;
use rsmpeg::avutil::AVFrame;
use rsmpeg::error::RsmpegError;
use rsmpeg::ffi;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fs;
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

fn decode(decode_context: &mut AVCodecContext, packet: Option<&AVPacket>, out_dir: &str) -> Result<(), RsmpegError> {
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

fn main() {
    // dump_av_info(&CString::new("assets/bunny_1080p.mp4").unwrap()).unwrap();
    decode_mp4_file("assets/bunny_1080p.mp4").unwrap();
    println!("Hello, world!");
}
