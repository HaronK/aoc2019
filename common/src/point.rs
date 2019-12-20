use std::fmt;

pub type PointI = Point<isize>;
pub type PointU = Point<usize>;

pub trait Max {
    fn max() -> Self;
}

impl Max for isize {
    fn max() -> Self {
        std::isize::MAX
    }
}

impl Max for usize {
    fn max() -> Self {
        std::usize::MAX
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn set(&mut self, x: T, y: T) {
        self.x = x;
        self.y = y;
    }
}

impl<T: Max> Point<T> {
    pub fn max() -> Self {
        Self {
            x: T::max(),
            y: T::max(),
        }
    }
}

impl<T: Default> Default for Point<T> {
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Point<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}, {:?}]", self.x, self.y)
    }
}
