use std::ops::Index;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Minivec<const SIZE: usize, T: Default + Copy> {
    length: usize,
    inner: [T; SIZE],
}

impl<const SIZE: usize, T: Default + Copy> Minivec<SIZE, T> {
    pub fn new() -> Self {
        Self {
            length: 0,
            inner: [T::default(); SIZE],
        }
    }

    pub fn len(&self) -> usize {
        return self.length;
    }

    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.length >= SIZE {
            return Err(item);
        }
        self.inner[self.length] = item;
        self.length += 1;

        Ok(())
    }
}

impl<'a, const SIZE: usize, T: Default + Copy> IntoIterator for &'a Minivec<SIZE, T> {
    type Item = &'a T;

    type IntoIter = MiniVecIterator<'a, SIZE, T>;

    fn into_iter(self) -> Self::IntoIter {
        MiniVecIterator {
            inner: self,
            position: 0,
        }
    }
}

impl<const SIZE: usize, T: Default + Copy> Index<usize> for Minivec<SIZE, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len() {
            panic!("out of bounds")
        }
        &self.inner[index]
    }
}

pub struct MiniVecIterator<'a, const SIZE: usize, T: Default + Copy> {
    inner: &'a Minivec<SIZE, T>,
    position: usize,
}

impl<'a, const SIZE: usize, T: Default + Copy> Iterator for MiniVecIterator<'a, SIZE, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.inner.len() {
            let value = Some(&self.inner.inner[self.position]);
            self.position += 1;
            value
        } else {
            None
        }
    }
}
