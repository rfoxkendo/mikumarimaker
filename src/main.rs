use mikumarimaker::mikumari_format;
use std::env::args;
use std::process::exit;

use std::io::{stdin, BufReader, Read};
use std::fs::File;
use rust_ringitem_format::{RingItem};
use frib_datasource::{data_sink_factory, DataSink};


const HEART_BEAT_MICROSECONDS : f64 = 524.288; // Time between heart beats.
const TDC_TICK_PS : f64 = 0.9765625;           // LSB value for tdc.
fn main() ->std::io::Result<()> {
    let argv : Vec<String> = args().collect();
    if argv.len() != 3 {
        usage();
        exit(-1);
    }
    let fname = argv[1].clone();
    let ring_name = argv[2].clone();

    // Open the file, attach a buffered reader to it and box it to create
    // a MikumariReader:

    let source : Box<dyn Read> = 
    if fname == "-" {
        let inf = stdin();
        Box::new(inf)
    } else {
        let f = File::open(&fname)?;
        let reader = BufReader::new(f);
        Box::new(reader)
    };

    let mut data_source = mikumari_format::MikumariReader::new(source);
    
    // Open the output ring item - or ring buffer.

    let mut ring_file = data_sink_factory(&ring_name).expect("Unable to open data sink");   

    // Mikumari data has a partial frame at the front. We _could_
    // figure out how to timestamp it, but, instead, we'll just skip
    // that data as that seems to be standard.

    let hb = skip_partial_frame(&mut data_source);
    println!("Found first hb: {}", hb.frame());

    let hb_t0 = hb.frame();        // our t0 frame.
    dump_data(&mut data_source, hb_t0, &mut ring_file);

    Ok(())
}
fn skip_partial_frame(src : &mut mikumari_format::MikumariReader) ->
    mikumari_format::Delimeter1
{
    let mut n=0;                      // Count dropped values:
    while let Ok(data) = src.read() {
        if let mikumari_format::MikumariDatum::Heartbeat0(d1) = data {
            println!("Skipped {} u64 before finding a heartbeat", n);
            return d1;
        }
        n += 1;
    }
    // We had an error before finding a heartbeat.

    eprintln!("Did not find the first heartbeat before eof or read error");
    exit(-1);

}
// t0 - the frame # of t0.
// We're going to try to make the times into absolutes as well.
// Ring items we make:
//   These consist of raw hit values.
//   the timestamp comes from the relative frame_no, but the first
//   u64 bit item is the absolute frame number.
//
fn dump_data(src : &mut mikumari_format::MikumariReader, t0 : u64, rf : &mut Box<dyn DataSink>) {
    let mut frame_no = 0;                       // THe current frame number.
    let mut absolute_frame = t0;

    // start a ring item for the first frame:

    let mut ring_item = RingItem::new_with_body_header(
        mikumari_format::MIKUMARI_FRAME_ITEM_TYPE,
        hb_frame_to_ts(frame_no) as u64,
        0, 0
     );
     ring_item.add(absolute_frame);
    while let Ok(data) = src.read() {
        match data {
            mikumari_format::MikumariDatum::LeadingEdge(le) => {
                ring_item.add(le.get());
            },
            mikumari_format::MikumariDatum::TrailingEdge(te) => {
                ring_item.add(te.get());
            }
            mikumari_format::MikumariDatum::Heartbeat0(_d) => {
                // Heart beat means we write the item and 
                // start a new one:
                rf.write(&ring_item).expect("Failed to write a ring item to data sink.");
            
                frame_no += 1;                   // Next frame.
                absolute_frame += 1;
                // Start the new ring item:

                ring_item = RingItem::new_with_body_header(
                    mikumari_format::MIKUMARI_FRAME_ITEM_TYPE,
                    hb_frame_to_ts(frame_no) as u64,
                    0,0
                );
                ring_item.add(absolute_frame);
            }
            mikumari_format::MikumariDatum::Heartbeat1(_d) => (),
            mikumari_format::MikumariDatum::Other(_d) => (),
        }
    }
    // Flush the last ring item out:
  
    rf.write(&ring_item).expect("Failed to write ring item to data sink.");
    
}

// Convert a frame number to a mikumari timestamp:

fn hb_frame_to_ts(frame: u64) -> f64 {
    let frame_t : f64 = frame as f64 * HEART_BEAT_MICROSECONDS; // frame_time in usec.
    (frame_t * (1.0e6)) / TDC_TICK_PS
}

fn usage() {
    eprintln!("Usage:");
    eprintln!("  mikumarimaker  source sink");
    eprintln!("Where:");
    eprintln!("   source - is a source of mikumaridata either a file or '-' for stdin");
    eprintln!("    sink - is a data sink URI These are of the form:");
    eprintln!("        * file:///absolute-file-path e.g. file:///`pwd`/somefile.evt");
    eprintln!("        * file:///- for stdout");
    eprintln!("        * tcp://localhost/somering to output to a ring buffer.");
}