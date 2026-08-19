use std::fmt::{self, Write};

#[derive(Debug, Clone, Copy)]
pub struct CharBuffer {
    data: [char; Self::MAX_CAPACITY],
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
        // data beyond [0..len] is never accessed; no need to zero the slot.
        Some(self.data[self.len])
    }

    pub fn clear(&mut self) {
        // data beyond [0..len] is never accessed; no need to zero slots.
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

    pub fn remove(&mut self, index: usize) {
        if index < self.len {
            if index < self.len - 1 {
                self.data.copy_within(index + 1..self.len, index);
            }
            // Decrement length directly; no need to zero the now-unreachable slot.
            self.len -= 1;
        }
    }

    /// Returns a snapshot of the buffer data and its length.
    /// Used by engine to save/restore buffer state without sentinel values.
    pub fn snapshot(&self) -> ([char; Self::MAX_CAPACITY], usize) {
        (self.data, self.len)
    }

    /// Restores buffer from a snapshot produced by [`snapshot`].
    pub fn restore(&mut self, data: &[char], len: usize) {
        self.data[..len].copy_from_slice(&data[..len]);
        self.len = len;
    }

    /// Returns an iterator over the chars in the buffer.
    pub fn iter(&self) -> std::slice::Iter<'_, char> {
        self.as_slice().iter()
    }
}

impl fmt::Display for CharBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &c in self.as_slice() {
            f.write_char(c)?;
        }
        Ok(())
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
        for i in 0..16u8 {
            assert!(buffer.push(char::from(b'a' + i)));
        }
        assert_eq!(buffer.len(), 16);
        assert!(buffer.is_full());
        assert!(!buffer.push('z'));
        assert_eq!(buffer.len(), 16);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_remove() {
        let mut buffer = CharBuffer::new();
        buffer.push('a');
        buffer.push('b');
        buffer.push('c');
        buffer.push('d');

        buffer.remove(1);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.as_slice(), &['a', 'c', 'd']);

        buffer.remove(2);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.as_slice(), &['a', 'c']);

        buffer.remove(0);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.as_slice(), &['c']);

        buffer.remove(0);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_pop_does_not_corrupt() {
        let mut buffer = CharBuffer::new();
        buffer.push('x');
        buffer.push('y');
        let popped = buffer.pop();
        assert_eq!(popped, Some('y'));
        assert_eq!(buffer.len(), 1);
        // Subsequent push should overwrite correctly
        buffer.push('z');
        assert_eq!(buffer.as_slice(), &['x', 'z']);
    }
}
