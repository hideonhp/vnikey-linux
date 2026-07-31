cat << 'INNER_EOF' > src/buffer.rs
#[derive(Debug, Clone, Copy)]
pub struct CharBuffer {
    data: [char; 16],
    len: usize,
}

impl Default for CharBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl CharBuffer {
    pub const MAX_CAPACITY: usize = 16;

    pub fn new() -> Self {
        Self {
            data: ['\0'; Self::MAX_CAPACITY],
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == Self::MAX_CAPACITY
    }

    pub fn push(&mut self, c: char) -> bool {
        if self.is_full() {
            return false;
        }
        self.data[self.len] = c;
        self.len += 1;
        true
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.is_empty() {
            return None;
        }
        self.len -= 1;
        let c = self.data[self.len];
        self.data[self.len] = '\0';
        Some(c)
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_slice(&self) -> &[char] {
        &self.data[..self.len]
    }

    pub fn last(&self) -> Option<char> {
        if self.is_empty() {
            None
        } else {
            Some(self.data[self.len - 1])
        }
    }

    pub fn replace_last(&mut self, c: char) {
        if !self.is_empty() {
            self.data[self.len - 1] = c;
        }
    }

    pub fn replace_at(&mut self, index: usize, c: char) {
        if index < self.len {
            self.data[index] = c;
        }
    }
}

impl PartialEq for CharBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl Eq for CharBuffer {}
INNER_EOF
