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
            None if oldpos < self.buf.len() => Some(&self.buf[oldpos..]),
            None => None,
        }
    }
}
