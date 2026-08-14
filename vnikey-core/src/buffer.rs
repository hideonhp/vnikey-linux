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
            data: ['\x00'; Self::MAX_CAPACITY],
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
        self.data[self.len] = '\x00';
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear() {
        let mut buffer = CharBuffer::new();
        buffer.push('a');
        buffer.push('b');
        assert_eq!(buffer.len(), 2);
        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_last() {
        let mut buffer = CharBuffer::new();
        assert_eq!(buffer.last(), None);
        buffer.push('a');
        assert_eq!(buffer.last(), Some('a'));
        buffer.push('b');
        assert_eq!(buffer.last(), Some('b'));
        buffer.pop();
        assert_eq!(buffer.last(), Some('a'));
        buffer.clear();
        assert_eq!(buffer.last(), None);
    }

    #[test]
    fn test_push_full() {
        let mut buffer = CharBuffer::new();
        for i in 0..16 {
            assert!(buffer.push(char::from_u32('a' as u32 + i as u32).unwrap()));
        }
        assert_eq!(buffer.len(), 16);
        assert!(buffer.is_full());
        assert!(!buffer.push('z'));
        assert_eq!(buffer.len(), 16);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_replace_at() {
        let mut buffer = CharBuffer::new();
        buffer.push('a');
        buffer.push('b');
        buffer.push('c');

        // Replace at valid indices
        buffer.replace_at(0, 'x');
        assert_eq!(buffer.as_slice(), &['x', 'b', 'c']);

        buffer.replace_at(1, 'y');
        assert_eq!(buffer.as_slice(), &['x', 'y', 'c']);

        buffer.replace_at(2, 'z');
        assert_eq!(buffer.as_slice(), &['x', 'y', 'z']);

        // Replace at out of bounds
        buffer.replace_at(3, 'w');
        assert_eq!(buffer.as_slice(), &['x', 'y', 'z']);

        buffer.replace_at(10, 'w');
        assert_eq!(buffer.as_slice(), &['x', 'y', 'z']);
    }
}
