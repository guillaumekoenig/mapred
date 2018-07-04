use std::io::{self, Read, Write};
use std::fs::File;
use std::{env, process};
use std::collections::BTreeMap;

mod job;
use job::Job;

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

fn isdelim(c: &u8) -> bool {
    c.is_ascii_whitespace() || c.is_ascii_punctuation()
}

fn count_words(buf: &[u8]) -> BTreeMap<&[u8], usize> {
    let mut bt = BTreeMap::new();
    for word in buf.split(isdelim) {
        if word.len() > 0 {
            *bt.entry(word).or_insert(0) += 1;
        }
    }
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
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut job = Job::new(&buf, 2, isdelim);
    for chunk in job.iter() {
        print!("START:");
        stdout.write(chunk).unwrap();
    }
    for (word, count) in count_words(&buf).iter() {
        stdout.write(word).unwrap();
        println!("={}", count);
    }
}
