use std::cmp::min;

pub struct Job<'a> {
    buf: &'a [u8],
    approx_chunk_size: usize,
    isdelim: fn(&u8) -> bool,
    pos: usize,
}

impl<'a> Job<'a> {
    pub fn new(buf: &[u8], nthreads: usize, isdelim: fn(&u8) -> bool) -> Job {
        Job {
            buf,
            approx_chunk_size: buf.len() / nthreads,
            isdelim,
            pos: 0,
        }
    }

    pub fn iter(&mut self) -> &mut Job<'a> {
        self.pos = 0;
        self
    }
}

impl<'a> Iterator for Job<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        // Split buf in chunks of roughly buf.len()/nthreads bytes,
        // making sure to split on word boundary only
        let oldpos = self.pos;
        self.pos = min(oldpos + self.approx_chunk_size, self.buf.len());
        match self.buf[self.pos..].iter().position(self.isdelim) {
            Some(d) => {
                self.pos += d;
                Some(&self.buf[oldpos..self.pos])
            }
            None if oldpos < self.buf.len() => {
                self.pos = self.buf.len();
                Some(&self.buf[oldpos..])
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
        let mut job = Job::new(b"", 1, |&c| c.is_ascii_whitespace());
        assert_eq!(job.iter().next(), None);
    }

    #[test]
    fn mapping_empty_input_on_10_threads() {
        let mut job = Job::new(b"", 10, |&c| c.is_ascii_whitespace());
        assert_eq!(job.iter().next(), None);
    }

    #[test]
    fn mapping_two_words_on_two_threads() {
        let mut job = Job::new(b"hello world", 2, |&c| c.is_ascii_whitespace());
        let it = job.iter();
        assert_eq!(it.next(), Some(&b"hello"[..]));
        assert_eq!(it.next(), Some(&b" world"[..]));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mapping_8_words_on_3_threads() {
        let mut job = Job::new(b"a b c d e f g h", 3, |&c| c.is_ascii_whitespace());
        let it = job.iter();
        assert_eq!(it.next(), Some(&b"a b c"[..]));
        assert_eq!(it.next(), Some(&b" d e f"[..]));
        assert_eq!(it.next(), Some(&b" g h"[..]));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mapping_4_uneven_words_on_3_threads() {
        let mut job = Job::new(b"a b c ef", 3, |&c| c.is_ascii_whitespace());
        let it = job.iter();
        assert_eq!(it.next(), Some(&b"a b"[..]));
        assert_eq!(it.next(), Some(&b" c"[..]));
        assert_eq!(it.next(), Some(&b" ef"[..]));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mapping_one_word_on_two_threads() {
        let mut job = Job::new(b"bouh", 2, |&c| c.is_ascii_whitespace());
        let it = job.iter();
        assert_eq!(it.next(), Some(&b"bouh"[..]));
        assert_eq!(it.next(), None);
    }
}
