use mikumarimaker::{mikumari_format, glom};
use rust_ringitem_format::{RingItem, PHYSICS_EVENT};
use frib_datasource::{data_source_factory, DataSource, data_sink_factory, DataSink};
use std::process::exit;
use std::mem::size_of;
use clap::{value_parser, Arg, ArgAction, Command, ArgMatches};


// Ring items generated will  be PHYSICS_EVENT 
// Output will be 
// | Absolute frame number | 64 bits.
// zero or more repetitions of hits of the form:
// | r/f channel           | 16 bits, top bit is 1 for trailing edge.
// | absolute-time         | 64 bits. computed by adding the timestamp to the hit time.
//
//   If I've done arithmetic properly, it's 213 days before the absolute time should
//   wrap.
// Usage:
//    defenestrator --dt coincidence-interval sourced sink.
//
// Source and sink are URI's --dt is in tdc units.
//
fn main() {
    // Define the command line parameter for clap:

    let parser = Command::new("defenestrator")
        .version("0.2.0").about("Defenestrates mikumari time data (AMANEQ)")
        .arg(Arg::new("dt")
            .short('t').long("dt").required(true).help("Coincidence interval")
            .value_parser(value_parser!(u64))
        )
        .arg(Arg::new("source").required(true).help("Data Source URI"))
        .arg(Arg::new("sink").required(true).help("Data Sink URI"));

    let matches = parser.get_matches();

    
    // Process the command line arguments.

    let ring_uri = matches.get_one::<String>("source").expect("No data source given");
    let out_path = matches.get_one::<String>("sink").expect("No data sink given");
    let glom_dt = matches.get_one::<u64>("dt").expect("No --dt given for gloming");

    // open the source:

    let mut source = data_source_factory(&ring_uri).expect("Could not open ring item source");
    let mut sink   = data_sink_factory(&out_path).expect("Could not open ring item sink");
    
    // Create the glommer:

    let mut glom = glom::Glom::new(sink, 0, *glom_dt);


    // For mikumari data, each frame -> a defenestrated frame.
    //while let Some(item) = source.read() {
    //    convert_item(&item, &mut sink);
    //}

}


fn convert_item(item : &RingItem, sink : &mut Box<dyn DataSink> ) {
    // if the ring item is not a MIKUMARI frame, just pass it unaltered.

    if item.type_id() != mikumari_format::MIKUMARI_FRAME_ITEM_TYPE {
        sink.write(item).expect("Unable to pass through a non-frame item.")
    } else {
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
    sink.flush();  
        
}