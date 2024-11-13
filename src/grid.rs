use std::ops::{Index, IndexMut};

use crate::util::iter_2d;

#[derive(Debug, Clone)]
pub struct Grid<T> {
    buf: Vec<T>,
    width: usize,
}

impl<T> Grid<T> {
    pub fn from_fn<F: FnMut(usize, usize) -> T>(width: usize, height: usize, mut f: F) -> Self {
        let mut buf = Vec::with_capacity(width * height);

        for (x, y) in iter_2d(0..width, 0..height) {
            buf.push(f(x, y));
        }

        Self { buf, width }
    }

    pub fn new(width: usize, height: usize) -> Self
    where
        T: Default,
    {
        Self::from_fn(width, height, |_, _| T::default())
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x > self.width {
            return None;
        }
        self.buf.get(y * self.width + x)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x > self.width {
            return None;
        }
        self.buf.get_mut(y * self.width + x)
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.buf.len() / self.width)
    }
}

impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;
    fn index(&self, (x, y): (usize, usize)) -> &T {
        self.get(x, y).unwrap()
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut T {
        self.get_mut(x, y).unwrap()
    }
}
