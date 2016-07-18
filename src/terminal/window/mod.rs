use std::ops::{Deref, DerefMut, Index};

use datatypes::{Coords, Region};
use terminal::CharGrid;

use self::View::*;

pub trait Grid: Index<Coords> {
    fn new(u32, u32, bool) -> Self;
}

impl Grid for CharGrid {
    fn new(width: u32, height: u32, expand: bool) -> CharGrid {
        CharGrid::new(width, height, expand)
    }
}

pub struct Window<T: Grid=CharGrid> {
    grid: T,
    view: View,
}

impl<T: Grid> Window<T> {
    pub fn reflowable(width: u32, height: u32, expand: bool) -> Window<T> {
        Window {
            grid: T::new(width, height, expand),
            view: Reflowable,
        }
    }

}

impl<T: Grid> Index<Coords> for Window<T> {
    type Output = T::Output;

    fn index(&self, idx: Coords) -> &T::Output {
        &self.grid[idx]
    }
}

impl<T: Grid> Deref for Window<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.grid
    }
}

impl<T: Grid> DerefMut for Window<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.grid
    }
}

enum View {
    Moveable(Region),
    Reflowable,
}
