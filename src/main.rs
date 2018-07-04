use std::cmp::min;
use std::io::{self, Read, Write};
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

fn isdelim(c: u8) -> bool {
    c.is_ascii_whitespace() || c.is_ascii_punctuation()
}

fn count_words(buf: &[u8]) -> BTreeMap<&[u8], usize> {
    let mut bt = BTreeMap::new();
    for word in buf.split(|&c| isdelim(c)) {
        if word.len() > 0 {
            *bt.entry(word).or_insert(0) += 1;
        }
    }
    bt
}

struct Job<'a> {
    buf: &'a [u8],
    nthreads: usize,
    pos: usize,
}

impl<'a> Iterator for Job<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        // Split buf in chunks of roughly buf.len()/nthreads bytes,
        // making sure not split inside a word
        let oldpos = self.pos;
        self.pos = min(oldpos + self.buf.len() / self.nthreads, self.buf.len());
        match self.buf[self.pos..].iter().position(|&c| isdelim(c)) {
            Some(d) => {
                self.pos += d;
                Some(&self.buf[oldpos..self.pos])
            }
            None if oldpos < self.buf.len() => Some(&self.buf[oldpos..]),
            None => None,
        }
    }
}

impl<'a> Job<'a> {
    fn iter(&mut self) -> &mut Job<'a> {
        self.pos = 0;
        self
    }
}

fn job(buf: &[u8], nthreads: usize) {
    let mut job = Job {
        buf,
        nthreads,
        pos: 0,
    };
    for chunk in job.iter() {
        println!("{:?}", chunk);
    }
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
    job(&buf, 2);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    for (word, count) in count_words(&buf).iter() {
        stdout.write(word).unwrap();
        println!("={}", count);
    }
}
