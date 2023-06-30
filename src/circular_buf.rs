pub(crate) struct CircularBuf<T> {
    buffer: Vec<T>,
    start: usize,
    length: usize,
}

impl<T> CircularBuf<T> {
    pub fn with_capacity(size: usize) -> Self {
        CircularBuf {
            buffer: Vec::with_capacity(size),
            start: 0,
            length: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        let n = self.buffer.len();
        let idx = (self.start + self.length) % n;
        self.buffer[idx] = item;

        if self.length < n {
            self.length += 1;
        } else {
            self.start = (self.start + 1) % n;
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length {
            return None;
        }

        let idx = (self.start + index) % self.buffer.len();
        Some(&self.buffer[idx])
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    pub fn full(&self) -> bool {
        self.length == self.buffer.len()
    }

    pub fn clear(&mut self) {
        self.start = 0;
        self.length = 0;
    }

    pub fn empty(&self) -> bool {
        self.length == 0
    }

    pub fn buffer(&self) -> &[T] {
        &self.buffer
    }
}