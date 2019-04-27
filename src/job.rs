use std::cmp::min;
use std::collections::BTreeMap;
use std::convert::AsRef;
use std::ops;
use std::sync::Arc;
use std::thread;

use merge::*;

pub struct Job<A> {
    buf: Arc<A>,
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

impl<A> Job<A>
where
    A: AsRef<[u8]> + Send + Sync + 'static,
{
    pub fn new(buf: A, nthreads: usize, isdelim: fn(&u8) -> bool) -> Job<A> {
        let len = buf.as_ref().len();
        Job {
            buf: Arc::new(buf),
            approx_chunk_size: len / nthreads,
            isdelim,
        }
    }

    fn iter(&self) -> JobChunkIter<A> {
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
                thread::spawn(move || count_words(&(*buf).as_ref()[range], isdelim))
            }).collect();
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

struct JobChunkIter<'a, A: 'a> {
    job: &'a Job<A>,
    pos: usize,
}

impl<'a, A: AsRef<[u8]>> Iterator for JobChunkIter<'a, A> {
    type Item = ops::Range<usize>;
    fn next(&mut self) -> Option<(ops::Range<usize>)> {
        // Split buf in chunks of roughly buf.len()/nthreads bytes,
        // making sure to split on word boundary only
        let job = &self.job;
        let buf = (*job.buf).as_ref();
        let oldpos = self.pos;
        self.pos = min(oldpos + job.approx_chunk_size, buf.len());
        match buf[self.pos..].iter().position(job.isdelim) {
            Some(d) => {
                self.pos += d;
                Some(oldpos..self.pos)
            }
            None if oldpos < buf.len() => {
                self.pos = buf.len();
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
        let v = b"hello world";
        let job = Job::new(v.clone(), 2, |&c| c.is_ascii_whitespace());
        let mut it = job.iter();
        assert_eq!(&v[it.next().unwrap()], b"hello");
        assert_eq!(&v[it.next().unwrap()], b" world");
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mapping_8_words_on_3_threads() {
        let v = b"a b c d e f g h";
        let job = Job::new(v.clone(), 3, |&c| c.is_ascii_whitespace());
        let mut it = job.iter();
        assert_eq!(&v[it.next().unwrap()], b"a b c");
        assert_eq!(&v[it.next().unwrap()], b" d e f");
        assert_eq!(&v[it.next().unwrap()], b" g h");
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mapping_4_uneven_words_on_3_threads() {
        let v = b"a b c ef";
        let job = Job::new(v.clone(), 3, |&c| c.is_ascii_whitespace());
        let mut it = job.iter();
        assert_eq!(&v[it.next().unwrap()], b"a b");
        assert_eq!(&v[it.next().unwrap()], b" c");
        assert_eq!(&v[it.next().unwrap()], b" ef");
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mapping_one_word_on_two_threads() {
        let v = b"bouh";
        let job = Job::new(v.clone(), 2, |&c| c.is_ascii_whitespace());
        let mut it = job.iter();
        assert_eq!(&v[it.next().unwrap()], b"bouh");
        assert_eq!(it.next(), None);
    }
}
