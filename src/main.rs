use mikumarimaker::mikumari_format;
use rust_ringitem_format::{RingItem, BodyHeader};
use std::fs::File;
use std::env;
use std::process::exit;
use std::path;

const TDC_FRAME_ITEM_TYPE : u32 = 51;     // Ring item type for TDC frame data.
const WINDOW_WIDTH : u16 =1000;
fn main() {
    // Get the command line argumnents. We need:
    // path to program (always [0]).
    // Number of frames to make.
    // output file path.

    let args : Vec<String> = env::args().collect();
    if args.len() != 3 {
        Usage(&args[0]);    // args[0] is always present.
        exit(-1);
    }
    let num_frames : u32 = match args[1].parse() {
        Ok(num) => num,
        Err(e)  => {
            eprintln!("Unable to convert frame count to integer: {}", e);
            Usage(&args[0]);
            exit(-1);
        }
    };
    let out_path = path::Path::new(&args[2]);
    let mut fd = match File::create(&out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not open the output file {} : {}", out_path.to_str().unwrap(), e);
            Usage(&args[0]);
            exit(-1);
        }
    };

    make_frames(&mut fd, num_frames);
    exit(0);
}

fn Usage(pgm_name : &str) {
    eprintln!("Usage: ");
    eprintln!("  {} num_frames output_file", pgm_name);
    eprintln!("Where:");
    eprintln!("    num_frames is the number of frames to make");
    eprintln!("    output_file is the name of the output file to write");
}
fn make_frames(f : &mut File, num_frames : u32) {
    let mut t = 0;
    for fno in 0..num_frames {
        let frame = make_empty_frame(t, fno);
        write_frame(f, t, frame);

        t += WINDOW_WIDTH;
    }
}
// Make an empty frame.. that is delim1 followed by delim 2
// with data_size = 0 (I guess)?
// 
// time : the timestamp and num the frame number.
// Returns the frame as a vector of u64's.
fn make_empty_frame(time : u16, num : u32) -> Vec<u64> {
    let d1 = mikumari_format::Delimeter1::new(time, num);
    let d2 = mikumari_format::Delimeter2::new(0);

    let result = vec![d1.get(), d2.get()];
    result
}
// Write a frame to the output
// f the file, t, the timestamp for the body header, frame the body contents.
//
fn write_frame(f : &mut File, t : u16, frame : Vec<u64>)  {
    // Make the body header:

    
    let mut item = RingItem::new_with_body_header(
        TDC_FRAME_ITEM_TYPE, t as u64, 0, 0
    );
    for ll in &frame {
        item.add(*ll);
    }
    match item.write_item(f) {
        Ok(_) => return,
        Err(e) => {
            eprintln!("Failed to write a ring item: {}", e);
            exit(-1);
        }
    };
}