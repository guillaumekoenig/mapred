extern crate memmap;

mod job;
mod merge;

use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::{env, process};

use job::*;

use memmap::Mmap;

fn mmap_file(filename: &str) -> io::Result<Mmap> {
    let f = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&f)? };
    Ok(mmap)
}

fn parse_args(args: &[String]) -> Result<(&str, usize), Box<Error>> {
    if args.len() != 3 {
        Err(Box::from("missing filename argument or number of threads"))
    } else {
        Ok((&args[1], str::parse(&args[2])?))
    }
}

fn isdelim(c: &u8) -> bool {
    // Match C's isspace() || ispunct()
    c.is_ascii_whitespace() || c.is_ascii_punctuation() || *c == 11
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (filename, nthreads) = parse_args(&args).unwrap_or_else(|err| {
        eprintln!("Error parsing args: {}", err);
        process::exit(1);
    });
    let buf = mmap_file(&filename).unwrap_or_else(|err| {
        eprintln!("Error mmapping file: {}", err);
        process::exit(2);
    });
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let job = Job::new(buf, nthreads, isdelim);
    for (word, count) in job.run() {
        stdout.write(&word).unwrap();
        println!("={}", count);
    }
}
