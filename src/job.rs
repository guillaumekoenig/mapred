use std::cmp::min;
use std::collections::BTreeMap;
use std::{iter, slice};
use std::ops::Add;

pub struct Job<'a> {
    buf: &'a [u8],
    approx_chunk_size: usize,
    isdelim: fn(&u8) -> bool,
}

fn count_words(chunk: &[u8], isdelim: fn(&u8) -> bool) -> BTreeMap<&[u8], usize> {
    let mut bt = BTreeMap::new();
    for word in chunk.split(isdelim) {
        if word.len() > 0 {
            *bt.entry(word).or_insert(0) += 1;
        }
    }
    bt
}

impl<'a> Job<'a> {
    pub fn new(buf: &[u8], nthreads: usize, isdelim: fn(&u8) -> bool) -> Job {
        Job {
            buf,
            approx_chunk_size: buf.len() / nthreads,
            isdelim,
        }
    }

    fn iter(&self) -> JobChunkIter {
        JobChunkIter { job: self, pos: 0 }
    }

    pub fn run(&self) -> Vec<(&[u8], usize)> {
        // for chunk in self.iter() {
        count_words(self.buf, self.isdelim)
            .iter()
            .map(|(&k, &v)| (k, v))
            .collect()
    }
}

struct JobChunkIter<'a> {
    job: &'a Job<'a>,
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

struct MergeSortIter<'a, K: 'a, V: 'a> {
    i1: iter::Peekable<slice::Iter<'a, (K, V)>>,
    i2: iter::Peekable<slice::Iter<'a, (K, V)>>,
}

impl<'a, K: 'a + Ord, V: Copy + Add<Output = V>> Iterator for MergeSortIter<'a, K, V> {
    type Item = V;
    fn next(&mut self) -> Option<V> {
        match (self.i1.peek(), self.i2.peek()) {
            (None, None) => None,
            (Some(&&(_, v)), None) => {
                self.i1.next();
                Some(v)
            }
            (None, Some(&&(_, v))) => {
                self.i2.next();
                Some(v)
            }
            (Some(&&(ref k1, v1)), Some(&&(ref k2, v2))) if k1 == k2 => {
                self.i1.next();
                self.i2.next();
                Some(v1 + v2)
            }
            (Some(&&(ref k1, v1)), Some(&&(ref k2, v2))) => {
                if k1 < k2 {
                    self.i1.next();
                    Some(v1)
                } else {
                    self.i2.next();
                    Some(v2)
                }
            }
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
