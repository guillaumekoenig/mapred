use std::iter;
use std::ops::Add;
use std::cmp::Ordering;

pub struct MergeSortIter<K, V, I: Iterator<Item = (K, V)>, J: Iterator<Item = (K, V)>> {
    pub i1: iter::Peekable<I>,
    pub i2: iter::Peekable<J>,
}

impl<K: Ord, V: Add<Output = V>, I: Iterator<Item = (K, V)>, J: Iterator<Item = (K, V)>> Iterator
    for MergeSortIter<K, V, I, J> {
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> {
        let ord = match (self.i1.peek(), self.i2.peek()) {
            (None, None) => return None,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(&(ref k1, _)), Some(&(ref k2, _))) => k1.cmp(&k2),
        };
        match ord {
            Ordering::Less => self.i1.next(),
            Ordering::Greater => self.i2.next(),
            Ordering::Equal => {
                let (k, v1) = self.i1.next().unwrap();
                let (_, v2) = self.i2.next().unwrap();
                Some((k, v1 + v2))
            }
        }
    }
}
