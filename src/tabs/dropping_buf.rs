use std::collections::RingBuf;

pub struct DroppingBuf<T>(RingBuf<T>);

impl<T> DroppingBuf<T> {
    pub fn with_capacity(capacity: u16) -> DroppingBuf<T> {
        DroppingBuf(RingBuf::with_capacity(capacity as usize))
    }

    pub fn insert(&mut self, elem: T) {
        if self.0.capacity() == self.0.len() {
            self.0.pop_back();
        }
        self.0.push_front(elem);
    }

    pub fn resize<I>(&mut self, new_size: u16, mut fill: I) where I: Iterator<Item = T> + DoubleEndedIterator {
        let new_size = new_size as usize;
        let capacity = self.0.capacity();
        if new_size > capacity {
            self.0.reserve_exact(new_size - capacity);
            let len = self.0.len();
            self.0.extend(fill.rev().skip(len).take(new_size - capacity));
        } else if new_size < capacity {
            self.0.truncate(new_size);
            self.0.shrink_to_fit();
        }
    }

    pub fn iter(&self) -> ::std::collections::ring_buf::Iter<T> {
        self.0.iter()
    }
}
