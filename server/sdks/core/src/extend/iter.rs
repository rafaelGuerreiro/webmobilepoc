pub trait IterExt: Iterator {
    fn chunked(self, size: usize) -> ChunkedIter<Self>
    where
        Self: Sized;

    fn chunked_exact(self, size: usize) -> ChunkedIter<Self>
    where
        Self: Sized;
}

impl<T, I> IterExt for T
where
    T: Iterator<Item = I>,
{
    fn chunked(self, size: usize) -> ChunkedIter<Self> {
        ChunkedIter::new(self, size, false)
    }

    fn chunked_exact(self, size: usize) -> ChunkedIter<Self> {
        ChunkedIter::new(self, size, true)
    }
}

pub struct ChunkedIter<I: Iterator> {
    inner: I,
    size: usize,
    exact: bool,
}

impl<I> ChunkedIter<I>
where
    I: Iterator,
{
    fn new(inner: I, size: usize, exact: bool) -> Self {
        let size = size.max(1);
        Self { inner, size, exact }
    }
}

impl<I> Iterator for ChunkedIter<I>
where
    I: Iterator,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = Vec::with_capacity(self.size);
        for _ in 0..self.size {
            match self.inner.next() {
                Some(item) => result.push(item),
                None => break,
            }
        }

        if self.exact && result.len() < self.size {
            return None;
        }

        if result.is_empty() {
            return None;
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunked_collects_in_chunks() {
        let values: Vec<_> = (1..=7).collect();
        let chunks: Vec<Vec<_>> = values.into_iter().chunked(3).collect();
        assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]]);
    }

    #[test]
    fn chunked_exact_drops_remainder() {
        let values: Vec<_> = (1..=7).collect();
        let chunks: Vec<Vec<_>> = values.into_iter().chunked_exact(3).collect();
        assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6]]);
    }
}
