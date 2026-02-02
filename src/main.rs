use mikumarimaker::mikumari_format;
use std::env::args;
use std::process::exit;

use std::io::{BufReader, BufRead};
use std::fs::File;

const heart_beat_microseconds : f64 = 524.288; // Time between heart beats.
const tdc_tick_ps : f64 = 0.9765625;           // LSB value for tdc.
fn main() ->std::io::Result<()> {
    let argv : Vec<String> = args().collect();
    if argv.len() != 2 {
        eprintln!("This program rerquires the name of a mmikumari input file ");
        exit(-1);
    }
    let fname = argv[1].clone();

    // Open the file, attache a buffered reader to it and box it to create
    // a MikumariReader:

    let f = File::open(&fname)?;
    let reader = BufReader::new(f);
    let mut source = Box::new(reader);

    let mut data_source = mikumari_format::MikumariReader::new(source);
    
    let hb = skip_partial_frame(&mut data_source);
    println!("Found first hb: {}", hb.frame());
    let hb_t0 = hb.frame();        // our t0 frame.
    dump_data(&mut data_source);

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
fn dump_data(src : &mut mikumari_format::MikumariReader) {
    while let Ok(data) = src.read() {
        match data {
            mikumari_format::MikumariDatum::LeadingEdge(le) => {
                println!("Leading edge time: ");
                println!("Chan: {} time: {:x}", le.channel(), le.Time());
            },
            mikumari_format::MikumariDatum::TrailingEdge(te) => {
                println!("Trailing edge time: ");
                println!("Chan {}, time: {:x}", te.channel(), te.Time());
            }
            mikumari_format::MikumariDatum::Heartbeat0(d) => {
                println!(
                    "Delimeter1 frame {} --------------", 
                    d.frame()
                );
            }
            mikumari_format::MikumariDatum::Heartbeat1(d) => {
                println!("Delimeter2 datasize: {}", d.datasize());
            }
            mikumari_format::MikumariDatum::Other(d) => 
                println!("Other data : {:x}", d)
        }
    }
}