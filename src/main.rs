use std::io::{self, Read};
use std::fs::File;
use std::{env, process};
use std::collections::BTreeMap;

fn read_file(filename: &str) -> io::Result<Vec<u8>> {
    let mut f = File::open(filename)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(buf)
}

fn parse_args(args: &[String]) -> Result<&str, &str> {
    if args.len() < 2 {
        Err("missing filename argument")
    } else {
        Ok(&args[1])
    }
}

fn count_words(_buf: &Vec<u8>) -> BTreeMap<String, usize> {
    let mut bt = BTreeMap::new();
    bt.insert(String::from("b"), 1);
    bt.insert(String::from("a"), 2);
    bt
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = parse_args(&args).unwrap_or_else(|err| {
        eprintln!("Error parsing args: {}", err);
        process::exit(1);
    });
    let buf = read_file(&filename).unwrap_or_else(|err| {
        eprintln!("Error reading file: {}", err);
        process::exit(2);
    });
    for (word, count) in count_words(&buf).iter() {
        println!("{}={}", word, count);
    }
}
