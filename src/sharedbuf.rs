use std::ops::Index;

/// A buffer of items, where each item is a slice of the underlying data.
///
/// Useful for reducing allocations for many small items which do not need to be modified or individually owned.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SharedBuf<T> {
    /// Shared buffer of all item slices, concatenated.
    data: Vec<T>,
    /// End indices of each item slice in `data`.
    ends: Vec<usize>,
}

impl<T> Default for SharedBuf<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            ends: Vec::new(),
        }
    }
}

impl<T> SharedBuf<T> {
    pub fn push(&mut self, item: impl IntoIterator<Item = T>) {
        self.data.extend(item);
        self.ends.push(self.data.len());
    }

    #[allow(unused)]
    pub fn extend(&mut self, items: impl IntoIterator<Item = impl IntoIterator<Item = T>>) {
        for item in items {
            self.push(item);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ends.len()
    }

    pub fn iter(&self) -> SharedBufIter<'_, T> {
        SharedBufIter::from(self)
    }

    #[allow(unused)]
    pub fn first(&self) -> Option<&[T]> {
        if self.is_empty() {
            None
        } else {
            Some(&self[0])
        }
    }

    #[allow(unused)]
    pub fn last(&self) -> Option<&[T]> {
        if self.is_empty() {
            None
        } else {
            Some(&self[self.len() - 1])
        }
    }
}

impl<T> Index<usize> for SharedBuf<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        let start = if index == 0 { 0 } else { self.ends[index - 1] };
        let end = self.ends[index];
        &self.data[start..end]
    }
}

pub struct SharedBufIter<'a, T> {
    buf: &'a SharedBuf<T>,
    front_idx: usize,
    back_idx: usize,
}

impl<'a, T> From<&'a SharedBuf<T>> for SharedBufIter<'a, T> {
    fn from(buf: &'a SharedBuf<T>) -> Self {
        Self {
            buf,
            front_idx: 0,
            back_idx: buf.len(),
        }
    }
}

impl<'a, T> Iterator for SharedBufIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.front_idx < self.back_idx {
            let item = &self.buf[self.front_idx];
            self.front_idx += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back_idx - self.front_idx;
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for SharedBufIter<'_, T> {}

impl<T> DoubleEndedIterator for SharedBufIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back_idx <= self.front_idx {
            return None;
        }
        self.back_idx -= 1;
        Some(&self.buf[self.back_idx])
    }
}

/// A [SharedBuf] of UTF-8 bytes, where each item is a `str`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SharedStringBuf {
    buf: SharedBuf<u8>,
}

impl SharedStringBuf {
    pub fn push(&mut self, item: impl AsRef<str>) {
        self.buf.push(item.as_ref().bytes());
    }

    #[allow(unused)]
    pub fn extend(&mut self, items: impl IntoIterator<Item = impl AsRef<str>>) {
        for item in items {
            self.push(item);
        }
    }

    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn iter(&self) -> SharedStringBufIter<'_> {
        self.into()
    }

    #[allow(unused)]
    pub fn first(&self) -> Option<&str> {
        self.buf.first().map(bytes_to_str)
    }

    #[allow(unused)]
    pub fn last(&self) -> Option<&str> {
        self.buf.last().map(bytes_to_str)
    }
}

impl Index<usize> for SharedStringBuf {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        bytes_to_str(&self.buf[index])
    }
}

/// Iterator over the `str` items of a [SharedStringBuf].
pub struct SharedStringBufIter<'a> {
    inner: SharedBufIter<'a, u8>,
}

impl<'a> From<&'a SharedStringBuf> for SharedStringBufIter<'a> {
    fn from(buf: &'a SharedStringBuf) -> Self {
        Self {
            inner: (&buf.buf).into(),
        }
    }
}

impl<'a> Iterator for SharedStringBufIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(bytes_to_str)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for SharedStringBufIter<'_> {}

impl DoubleEndedIterator for SharedStringBufIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(bytes_to_str)
    }
}

/// Every item is pushed from a whole `&str`, so each slice is valid UTF-8.
fn bytes_to_str(bytes: &[u8]) -> &str {
    // SAFETY: SharedStringBuf only ever pushes complete `str`s,
    // and items are sliced on the boundaries recorded at push time.
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> SharedStringBuf {
        let mut b = SharedStringBuf::default();
        b.extend(["a", "bb", "", "ccc"]);
        b
    }

    #[test]
    fn forward() {
        let b = make();
        assert_eq!(b.iter().collect::<Vec<_>>(), ["a", "bb", "", "ccc"]);
        assert_eq!(b.iter().len(), 4);
    }

    #[test]
    fn backward() {
        let b = make();
        assert_eq!(b.iter().rev().collect::<Vec<_>>(), ["ccc", "", "bb", "a"]);
    }

    #[test]
    fn mixed() {
        let b = make();
        let mut it = b.iter();
        assert_eq!(it.next(), Some("a"));
        assert_eq!(it.next_back(), Some("ccc"));
        assert_eq!(it.len(), 2);
        assert_eq!(it.next(), Some("bb"));
        assert_eq!(it.next_back(), Some(""));
        assert_eq!(it.len(), 0);
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn single_and_empty() {
        let mut b = SharedStringBuf::default();
        assert_eq!(b.iter().next(), None);
        b.push("x");
        assert_eq!(b.iter().collect::<Vec<_>>(), ["x"]);
        assert_eq!(b.iter().rev().collect::<Vec<_>>(), ["x"]);
    }
}
