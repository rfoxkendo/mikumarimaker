use mikumarimaker::mikumari_format;
use rust_ringitem_format::{RingItem, BodyHeader, PHYSICS_EVENT};
use frib_datasource::{data_source_factory, DataSource, data_sink_factory, DataSink};
use std::env;
use std::process::exit;
use std::io;
use std::io::Write;
use std::fs::File;
use std::mem::size_of;
// Ring items generated will  be PHYSICS_EVENT 
// Output will be 
// | Absolute frame number | 64 bits.
// zero or more repetitions of hits of the form:
// | r/f channel           | 16 bits, top bit is 1 for trailing edge.
// | absolute-time         | 64 bits. computed by adding the timestamp to the hit time.
//
//   If I've done arithmetic properly, it's 213 days before the absolute time should
//   wrap.

fn main() {
    // We'll use two parameters:
    // First is the ring datasource URI
    // second the data sink object file. "-" for the data sink
    // will mean stdout.

    // Process the command line arguments.

    let argv : Vec<String> = env::args().collect();
    if argv.len() != 3 {
        usage();
    }
    let ring_uri = argv[1].clone();
    let out_path = argv[2].clone();

    // open the source:

    let mut source = data_source_factory(&ring_uri).expect("Could not open ring item source");
    let mut sink   = data_sink_factory(&out_path).expect("Could not open ring item sink");
    

    // For mikumari data, each frame -> a defenestrated frame.
    while let Some(item) = source.read() {
        convert_item(&item, &mut sink);
    }

}


fn  usage() -> ! {
    eprintln!("Usage:");
    eprintln!("   defenestrator  in-uri out-file");
    eprintln!("Where");
    eprintln!("   in-uri is the URI of the data source '-' means stdin");
    eprintln!("   out-uri is the URI for the output");
    

    exit(-1);
}

fn convert_item(item : &RingItem, sink : &mut Box<dyn DataSink> ) {
    let bh = item.get_bodyheader().unwrap();
    let t0 = bh.timestamp;
    let payload = item.payload();    // Vec<u8>


    let mut  output = RingItem::new_with_body_header(
        PHYSICS_EVENT,
        bh.timestamp, bh.source_id, bh.barrier_type
    );
    // We are assured there's an absolute frame number (64 bits)
    // Payload includes the body header.

    let mut cursor = size_of::<u64>() + 2 * size_of::<u32>(); // skip body header.
    let absolute_fno = u64::from_ne_bytes(payload[cursor..cursor+size_of::<u64>()].try_into().unwrap());
    output.add(absolute_fno);
    cursor += size_of::<u64>();   // First (if any) data item:
    while cursor < payload.len() {
        let raw = u64::from_ne_bytes(payload[cursor..cursor+size_of::<u64>()].try_into().unwrap());
        
        match mikumari_format::MikumariDatum::from_u64(raw) {
            mikumari_format::MikumariDatum::LeadingEdge(le)  => {
                let byte : u16 = le.channel() as u16;     // No top bit.
                output.add(byte);
                let t : u64 = le.Time() as u64 + t0;
                output.add(t);
            },

            mikumari_format::MikumariDatum::TrailingEdge(te) => {
                let byte : u16 = te.channel() as u16 | 0x8000;     // Top bit for falling edge.
                output.add(byte);
                let t : u64 = te.Time() as u64 + t0;
                output.add(t);
            },
            _ => {},
        }

        cursor += size_of::<u64>();
    }
    sink.write(&output).expect("Unable to write physics event ring item");
    
}