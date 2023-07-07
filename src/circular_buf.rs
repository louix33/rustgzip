use std::ops::Index;

pub(crate) struct CircularBuf<T: Default+Clone> {
    buffer: Vec<T>,
    start: usize,
    length: usize,
}

impl<T: Default+Clone> Index<usize> for CircularBuf<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[(self.start + index) % self.buffer.len()]
    }
}

impl<T: Default+Clone> CircularBuf<T> {
    pub fn with_capacity(size: usize) -> Self {
        CircularBuf {
            buffer: vec![Default::default(); size],
            start: 0,
            length: 0,
        }
    }

    /// Push an item to the end of the buffer. If the buffer is full, the oldest item will be replaced and returned.
    pub fn push_back(&mut self, item: T) -> Option<T> {
        let capacity = self.buffer.len();
        let idx = (self.start + self.length) % capacity;

        if self.length < capacity {
            self.length += 1;
            self.buffer[idx] = item;
            None
        } else {
            self.start = (self.start + 1) % capacity;
            Some(std::mem::replace(&mut self.buffer[idx], item))
        }
    }

    /// Pop an item from the front of the buffer. If the buffer is empty, None will be returned.
    pub fn pop_front(&mut self) -> Option<T> {
        if self.length == 0 {
            return None;
        }

        let idx = self.start;
        self.start = (self.start + 1) % self.buffer.len();
        self.length -= 1;
        Some(std::mem::replace(&mut self.buffer[idx], Default::default()))
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.length == 0 {
            None
        } else {
            Some(&mut self.buffer[self.start])
        }
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.length == 0 {
            None
        } else {
            let idx = (self.start + self.length - 1) % self.buffer.len();
            Some(&mut self.buffer[idx])
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_full(&self) -> bool {
        self.length == self.buffer.len()
    }

    pub fn clear(&mut self) {
        self.length = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}