use mikumarimaker::mikumari_format;
use std::env::args;
use std::process::exit;
fn main() {
    let argv : Vec<String> = args().collect();
    if argv.len() != 2 {
        eprintln!("This program rerquires the name of a mmikumari input file ");
        exit(-1);
    }
    let fname = argv[0].clone();
}