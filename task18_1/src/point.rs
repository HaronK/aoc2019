use std::fmt;

pub type PointI = Point<isize>;
pub type PointU = Point<usize>;

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
