use rsmpeg::avformat::AVFormatContextInput;
use rsmpeg::avutil::AVFrame;
use std::error::Error;
use std::ffi::CStr;
use std::fs::{self, File};
use std::slice;
use std::io::prelude::*;

pub fn dump_av_info(path: &CStr) -> Result<(), Box<dyn Error>> {
    let mut input_format_context = AVFormatContextInput::open(path)?;
    input_format_context.dump(0, path)?;
    Ok(())
}

pub fn file_save_yuv420p(frame: &AVFrame, file: &mut File) {
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

pub fn h264_extradata_to_annexb(extra_data: *mut u8, extra_data_size: i32) -> &'static [u8] {
    let mut len = 0;
    let start_code0: [u8; 4] = [0, 0, 0, 1];
    let mut total_size = 0;
    let mut unit_size = 0;
    // let padding = AV_INPUT_BUFFER_PADDING_SIZE;
    let mut pextra_data = extra_data.wrapping_add(4 as usize); // skip the fixed header_data
    let pout: *mut u8 = [0u8; 64].as_mut_ptr();
    pextra_data = pextra_data.wrapping_add(1 as usize); // skip ff

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
