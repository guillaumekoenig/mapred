use std::cmp::min;
use std::collections::BTreeMap;
use std::thread;
use std::sync::Arc;
use std::ops;

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
    pub fn new(buf: Vec<u8>, nthreads: usize, isdelim: fn(&u8) -> bool) -> Job {
        let len = buf.len();
        Job {
            buf: Arc::new(buf),
            approx_chunk_size: len / nthreads,
            isdelim,
        }
    }

    fn iter(&self) -> JobChunkIter {
        JobChunkIter { job: self, pos: 0 }
    }

    pub fn run(self) -> Vec<(Vec<u8>, usize)> {
        let chunks = self.iter();
        // We must collect into vector to force spawning all threads,
        // otherwise the threads will run in turn (because iterators
        // are evaluated lazily)
        let handles: Vec<_> = chunks
            .map(|range| {
                let buf = Arc::clone(&self.buf);
                let isdelim = self.isdelim;
                thread::spawn(move || count_words(&(*buf)[range], isdelim))
            })
            .collect();
        handles.into_iter().fold(Vec::new(), |acc, h| {
            let bt = h.join().unwrap();
            let merge = MergeSortIter {
                // Use into_iter so we take ownership and don't need
                // to make a copy of key
                i1: acc.into_iter().peekable(),
                i2: bt.into_iter().peekable(),
            };
            merge.collect()
        })
    }
}

struct JobChunkIter<'a> {
    job: &'a Job,
    pos: usize,
}

impl<'a> Iterator for JobChunkIter<'a> {
    type Item = ops::Range<usize>;
    fn next(&mut self) -> Option<(ops::Range<usize>)> {
        // Split buf in chunks of roughly buf.len()/nthreads bytes,
        // making sure to split on word boundary only
        let job = &self.job;
        let oldpos = self.pos;
        self.pos = min(oldpos + job.approx_chunk_size, job.buf.len());
        match job.buf[self.pos..].iter().position(job.isdelim) {
            Some(d) => {
                self.pos += d;
                Some(oldpos..self.pos)
            }
            None if oldpos < job.buf.len() => {
                self.pos = job.buf.len();
                Some(oldpos..self.pos)
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
