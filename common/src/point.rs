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
    pub z: T,
}

impl<T> Point<T> {
    pub fn new3(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn set3(&mut self, x: T, y: T, z: T) {
        self.x = x;
        self.y = y;
        self.z = z;
    }
}

impl<T: Default> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
            z: T::default(),
        }
    }

    pub fn set(&mut self, x: T, y: T) {
        self.x = x;
        self.y = y;
        // self.z = T::default();
    }
}

impl<T: Max> Point<T> {
    pub fn max() -> Self {
        Self {
            x: T::max(),
            y: T::max(),
            z: T::max(),
        }
    }
}

impl<T: Default> Default for Point<T> {
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
            z: T::default(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Point<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}, {:?}, {:?}]", self.x, self.y, self.z)
    }
}
