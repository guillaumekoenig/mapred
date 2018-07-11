use std::cmp::min;
use std::collections::BTreeMap;
use std::thread;
use std::sync::Arc;
use std::mem;

use merge::*;

pub struct Job {
    buf: Arc<Vec<u8>>,
    approx_chunk_size: usize,
    isdelim: fn(&u8) -> bool,
}

fn count_words(chunk: &[u8], isdelim: fn(&u8) -> bool) -> BTreeMap<Vec<u8>, usize> {
    let mut bt = BTreeMap::new();
    for word in chunk.split(isdelim) {
        if word.len() > 0 {
            *bt.entry(Vec::from(word)).or_insert(0) += 1;
        }
    }
    bt
}

impl Job {
    fn iter(&self) -> JobChunkIter {
        JobChunkIter { job: self, pos: 0 }
    }

    pub fn run(buf: Vec<u8>, nthreads: usize, isdelim: fn(&u8) -> bool) -> Vec<(Vec<u8>, usize)> {
        // let chunks = self.iter();
        let buf = Arc::new(buf);
        let mut handles = Vec::new();
        for _ in 0..nthreads {
            let buf = Arc::clone(&buf);
            let h = thread::spawn(move || count_words(&*buf, isdelim));
            handles.push(h);
        }
        let mut acc = Vec::<(Vec<u8>, usize)>::new();
        // handles.iter().fold(Vec::new(), |acc, h| {
        for h in handles {
            let bt = h.join().unwrap();
            let mut new = {
                let merge = MergeSortIter {
                    i1: acc.iter().map(|&(ref k, v)| (k.clone(), v)).peekable(),
                    i2: bt.iter().map(|(&ref k, &v)| (k.clone(), v)).peekable(),
                };
                merge.collect()
            };
            mem::swap(&mut new, &mut acc);
        } //)
        acc
    }
}

struct JobChunkIter<'a> {
    job: &'a Job,
    pos: usize,
}

impl<'a> Iterator for JobChunkIter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        // Split buf in chunks of roughly buf.len()/nthreads bytes,
        // making sure to split on word boundary only
        let job = &self.job;
        let oldpos = self.pos;
        self.pos = min(oldpos + job.approx_chunk_size, job.buf.len());
        match job.buf[self.pos..].iter().position(job.isdelim) {
            Some(d) => {
                self.pos += d;
                Some(&job.buf[oldpos..self.pos])
            }
            None if oldpos < job.buf.len() => {
                self.pos = job.buf.len();
                Some(&job.buf[oldpos..])
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Job;

    #[test]
    fn mapping_empty_input_on_one_thread() {
        let job = Job::new(b"", 1, |&c| c.is_ascii_whitespace());
        assert_eq!(job.iter().next(), None);
    }

    #[test]
    fn mapping_empty_input_on_10_threads() {
        let job = Job::new(b"", 10, |&c| c.is_ascii_whitespace());
        assert_eq!(job.iter().next(), None);
    }

    #[test]
    fn mapping_two_words_on_two_threads() {
        let job = Job::new(b"hello world", 2, |&c| c.is_ascii_whitespace());
        let mut it = job.iter();
        assert_eq!(it.next(), Some(&b"hello"[..]));
        assert_eq!(it.next(), Some(&b" world"[..]));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mapping_8_words_on_3_threads() {
        let job = Job::new(b"a b c d e f g h", 3, |&c| c.is_ascii_whitespace());
        let mut it = job.iter();
        assert_eq!(it.next(), Some(&b"a b c"[..]));
        assert_eq!(it.next(), Some(&b" d e f"[..]));
        assert_eq!(it.next(), Some(&b" g h"[..]));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mapping_4_uneven_words_on_3_threads() {
        let job = Job::new(b"a b c ef", 3, |&c| c.is_ascii_whitespace());
        let mut it = job.iter();
        assert_eq!(it.next(), Some(&b"a b"[..]));
        assert_eq!(it.next(), Some(&b" c"[..]));
        assert_eq!(it.next(), Some(&b" ef"[..]));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mapping_one_word_on_two_threads() {
        let job = Job::new(b"bouh", 2, |&c| c.is_ascii_whitespace());
        let mut it = job.iter();
        assert_eq!(it.next(), Some(&b"bouh"[..]));
        assert_eq!(it.next(), None);
    }
}
