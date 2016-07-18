//  notty is a new kind of terminal emulator.
//  Copyright (C) 2015 without boats
//  
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//  
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Affero General Public License for more details.
//  
//  You should have received a copy of the GNU Affero General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
use std::cmp;
use std::collections::VecDeque;
use std::iter;
use std::mem;

use datatypes::{Coords, Direction, Region};

pub struct Grid<T> {
    pub width: usize,
    pub height: usize,
    data: VecDeque<T>,
    rem_x: Option<usize>,
    rem_y: Option<usize>,
}

impl<T: Clone + Default> Grid<T> {

    pub fn with_x_cap(max_x: usize) -> Grid<T> {
        Grid::constructor(Some(max_x), None)
    }

    pub fn with_y_cap(max_y: usize) -> Grid<T> {
        Grid::constructor(None, Some(max_y))
    }

    pub fn with_x_y_caps(max_x: usize, max_y: usize) -> Grid<T> {
        Grid::constructor(Some(max_x), Some(max_y))
    }

    pub fn with_infinite_scroll() -> Grid<T> {
        Grid::constructor(None, None)
    }

    fn constructor(max_x: Option<usize>, max_y: Option<usize>)
            -> Grid<T> {
        Grid {
            width: 0,
            height: 0,
            data: VecDeque::new(),
            rem_x: max_x,
            rem_y: max_y,
        }
    }

    pub fn bounds(&self) -> Option<Region> {
        if self.width > 0 && self.height > 0 {
            Some(Region::new(0, 0, self.width as u32, self.height as u32))
        } else { None }
    }

    pub fn range_inclusive(&self, start: Coords, end: Coords)
            -> iter::Take<iter::Skip<<&VecDeque<T> as IntoIterator>::IntoIter>> {
        assert!(self.width > start.x as usize, "{} outside of x bounds", start.x);
        assert!(self.height > start.y as usize, "{} outside of y bounds", start.y);
        assert!(self.width > end.x as usize, "{} outside of x bounds", end.x);
        assert!(self.height > end.y as usize, "{} outside of y bounds", end.y);
        let start = start.x as usize + start.y as usize * self.width;
        let end = end.x as usize + end.y as usize * self.width;
        assert!(end >= start, "range must be ascending");
        self.into_iter().skip(start).take(end - start + 1)
    }

    // Guarantee that this grid can grow to the given width.
    pub fn guarantee_width(&mut self, width: usize) {
        let new_rem = self.width.saturating_sub(width);
        self.rem_x.as_mut().map(|rem| *rem = cmp::max(*rem, new_rem));
    }

    // Guarantee that this grid can grow to the given height.
    pub fn guarantee_height(&mut self, height: usize) {
        let new_rem = self.height.saturating_sub(height);
        self.rem_y.as_mut().map(|rem| *rem = cmp::max(*rem, new_rem));
    }

    pub fn add_to_top(&mut self, data: Vec<T>) {
        assert!(data.len() % self.width == 0);
        self.height += data.len() / self.width;
        for item in data {
            self.data.push_front(item);
        }
    }

    pub fn add_to_bottom(&mut self, data: Vec<T>) {
        assert!(data.len() % self.width == 0);
        self.height += data.len() / self.width;
        for item in data {
            self.data.push_back(item);
        }
    }

    pub fn remove_from_top(&mut self, n: usize) -> Vec<T> {
        assert!(n < self.height);
        self.height -= n;
        let n = n * self.width;
        self.data.drain(..n).collect()
    }

    pub fn remove_from_bottom(&mut self, n: usize) -> Vec<T> {
        assert!(n < self.height);
        self.height -= n;
        let n = self.data.len() - (n * self.width);
        self.data.drain(n..).collect()
    }

    pub fn add_to_left(&mut self, data: Vec<T>) {
        assert!(data.len() % self.height == 0);
        let extra_width = data.len() / self.height;
        let width = self.width;
        self.width += extra_width;
        let iter = data.into_iter().enumerate().map(|(idx, item)| {
            ((idx / extra_width) * width, item)
        }).rev();
        for (idx, item) in iter {
            self.data.insert(idx, item);
        }
    }

    pub fn remove_from_left(&mut self, n: usize) -> Vec<T> {
        assert!(n < self.width);
        let width = self.width;
        let len = self.data.len();
        self.width -= n;
        (0..len).filter(|&x| (x % width) < n)
                .rev().map(|idx| self.data.remove(idx).unwrap())
                .collect()
    }

    pub fn add_to_right(&mut self, data: Vec<T>) {
        assert!(data.len() % self.height == 0);
        let extra_width = data.len() / self.height;
        let width = self.width;
        self.width += extra_width;
        let iter = data.into_iter().enumerate().map(|(idx, item)| {
            ((idx / extra_width) * width + width, item)
        }).rev();
        for (idx, item) in iter {
            self.data.insert(idx, item);
        }
    }

    pub fn remove_from_right(&mut self, n: usize) -> Vec<T> {
        assert!(n < self.width);
        let width = self.width;
        let len = self.data.len();
        self.width -= n;
        (0..len).filter(|&x| (x % width) >= width - n)
                .rev().map(|idx| self.data.remove(idx).unwrap())
                .collect()
    }

    pub fn scroll(&mut self, n: usize, direction: Direction) {
        use datatypes::Direction::*;
        match direction {
            Up if self.rem_y != Some(0)     => self.extend_up(n),
            Up if n >= self.height          => self.data.clear(),
            Up                              => self.shift_up(n),
            Down if self.rem_y != Some(0)   => self.extend_down(n),
            Down if n >= self.height        => self.data.clear(),
            Down                            => self.shift_down(n),
            Left if self.rem_x != Some(0)   => self.extend_left(n),
            Left if n >= self.width         => self.data.clear(),
            Left                            => self.shift_left(n),
            Right if self.rem_x != Some(0)  => self.extend_right(n),
            Right if n >= self.width        => self.data.clear(),
            Right                           => self.shift_right(n),
        }
    }

    pub fn moveover(&mut self, from: Coords, to: Coords) {
        if let Some(from) = self.get_mut(from).map(|cell| mem::replace(cell, T::default())) {
            self.get_mut(to).map(|to| *to = from);
        }
    }

    fn extend_up(&mut self, n: usize) {
        let rem_or_n = self.rem_y.map_or(n, |y| cmp::min(y, n));
        for _ in 0..(rem_or_n * self.width) {
            self.data.push_front(T::default());
        }
        self.height += rem_or_n;
        if self.rem_y.map_or(false, |y| n > y) {
            let rem = n - self.rem_y.unwrap();
            self.shift_up(rem);
        }
        self.rem_y = self.rem_y.map(|y| y.saturating_sub(n));
    }

    fn extend_down(&mut self, n: usize) {
        let rem_or_n = self.rem_y.map_or(n, |y| cmp::min(y, n));
        for _ in 0..(rem_or_n * self.width) {
            self.data.push_back(T::default());
        }
        self.height += rem_or_n;
        if self.rem_y.map_or(false, |y| n > y) {
            let rem = n - self.rem_y.unwrap();
            self.shift_down(rem);
        }
        self.rem_y = self.rem_y.map(|y| y.saturating_sub(n));
    }

    fn extend_left(&mut self, n: usize) {
        let rem_or_n = self.rem_x.map_or(n, |x| cmp::min(x, n));
        for i in 0..rem_or_n {
            for j in (1..self.height).rev() {
                self.data.insert((self.width + i) * j, T::default());
            }
            self.data.push_front(T::default());
        }
        self.width += rem_or_n;
        if self.rem_x.map_or(false, |x| n > x) {
            let rem = n - self.rem_x.unwrap();
            self.shift_left(rem);
        }
        self.rem_x = self.rem_x.map(|x| x.saturating_sub(n));
    }

    fn extend_right(&mut self, n: usize) {
        let rem_or_n = self.rem_x.map_or(n, |x| cmp::min(x, n));
        for i in 0..rem_or_n {
            for j in (1..self.height).rev() {
                self.data.insert((self.width + i) * j, T::default());
            }
            self.data.push_back(T::default());
        }
        self.width += rem_or_n;
        if self.rem_x.map_or(false, |x| n > x) {
            let rem = n - self.rem_x.unwrap();
            self.shift_right(rem);
        }
        self.rem_x = self.rem_x.map(|x| x.saturating_sub(n));
    }

    fn shift_up(&mut self, n: usize) {
        for _ in 0..(n * self.width) {
            self.data.pop_back();
            self.data.push_front(T::default());
        }
    }

    fn shift_down(&mut self, n: usize) {
        for _ in 0..(n * self.width) {
            self.data.pop_front();
            self.data.push_back(T::default());
        }
    }

    fn shift_left(&mut self, n: usize) {
        for _ in 0..n {
            self.data.pop_back();
            self.data.push_front(T::default());
            for i in 1..self.height {
                self.data[i * self.width] = T::default();
            }
        }
    }

    fn shift_right(&mut self, n: usize) {
        for _ in 0..n {
            self.data.pop_front();
            self.data.push_back(T::default());
            for i in 1..self.height {
                self.data[(i * self.width) - 1] = T::default();
            }
        }
    }

    pub fn fill_to_width(&mut self, width: usize) {
        let extension = width.saturating_sub(self.width);
        self.extend_right(extension);
    }

    pub fn fill_to_height(&mut self, height: usize) {
        let extension = height.saturating_sub(self.height);
        self.extend_down(extension);
    }

    pub fn write_at(&mut self, coords: Coords, data: T) {
        self.fill_to_width(coords.x as usize + 1);
        self.fill_to_height(coords.y as usize + 1);
        self.data[linearize(self.width, coords)] = data;
    }

    pub fn get(&self, coords: Coords) -> Option<&T> {
        self.bounds().and_then(move |bounds| if bounds.contains(coords) { 
            Some(&self.data[linearize(self.width, coords)])
        } else { None })
    }

    pub fn get_mut(&mut self, coords: Coords) -> Option<&mut T> {
        self.bounds().and_then(move |bounds| if bounds.contains(coords) { 
            Some(&mut self.data[linearize(self.width, coords)])
        } else { None })
    }

}

fn linearize(width: usize, Coords { x, y }: Coords) -> usize {
    y as usize * width + x as usize
}

/*
impl<T> Index<Coords> for Grid<T> {
    type Output = T;
    fn index(&self, idx: Coords) -> &T {
        assert!(self.width > idx.x as usize, "{} index outside of x bounds", idx.x);
        assert!(self.height > idx.y as usize, "{} index outside of y bounds", idx.y);
        &self.data[(idx.y as usize * self.width) + idx.x as usize]
    }
}

impl<T> IndexMut<Coords> for Grid<T> {
    fn index_mut(&mut self, idx: Coords) -> &mut T {
        assert!(self.width > idx.x as usize, "{} index outside of x bounds", idx.x);
        assert!(self.height > idx.y as usize, "{} index outside of y bounds", idx.y);
        &mut self.data[(idx.y as usize * self.width) + idx.x as usize]
    }
}
*/

impl<'a, T> IntoIterator for &'a Grid<T> {
    type IntoIter = <&'a VecDeque<T> as IntoIterator>::IntoIter;
    type Item = <&'a VecDeque<T> as IntoIterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        (&self.data).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Grid<T> {
    type IntoIter = <&'a mut VecDeque<T> as IntoIterator>::IntoIter;
    type Item = <&'a mut VecDeque<T> as IntoIterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.data).into_iter()
    }
}

#[cfg(test)]
mod tests {

    use datatypes::Coords;
    use datatypes::Direction::*;

    use super::Grid;

    fn run_test<F: Fn(Grid<i32>, usize, usize)>(test: F, new_w: usize, new_h: usize) {
        let fill = |grid: &mut Grid<i32>| for i in grid { *i = 1; };
        test({ let mut grid = Grid::new(8, 8); fill(&mut grid); grid }, 8, 8);
        test({ let mut grid = Grid::with_x_cap(8, 8, 10); fill(&mut grid); grid }, new_w, 8);
        test({ let mut grid = Grid::with_y_cap(8, 8, 10); fill(&mut grid); grid }, 8, new_h);
        test({ let mut grid = Grid::with_x_y_caps(8, 8, 10, 10); fill(&mut grid); grid },
             new_w, new_h);
    }

    #[test]
    fn add_to_top() {
        run_test(|mut grid, width, _| {
            grid.add_to_top(vec![0; 8]);
            for i in 0..grid.width {
                assert_eq!(grid[Coords {x:i as u32, y:0}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:1}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(9, grid.height);
            assert_eq!(grid.data.len(), width * 9);
        }, 8, 9)
    }

    #[test]
    fn add_to_bottom() {
        run_test(|mut grid, width, _| {
            grid.add_to_bottom(vec![0; 8]);
            for i in 0..grid.width {
                assert_eq!(grid[Coords {x:i as u32, y:8}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:7}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(9, grid.height);
            assert_eq!(grid.data.len(), width * 9);
        }, 8, 9);
    }

    #[test]
    fn add_to_left() {
        run_test(|mut grid, _, height| {
            grid.add_to_left(vec![0; 8]);
            for i in 0..grid.height {
                assert_eq!(grid[Coords {x:0, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:1, y:i as u32}], 1);
            }
            assert_eq!(9, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), height * 9);
        }, 9, 8)
    }

    #[test]
    fn add_to_right() {
        run_test(|mut grid, _, height| {
            grid.add_to_right(vec![0; 8]);
            for i in 0..grid.height {
                assert_eq!(grid[Coords {x:8, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:7, y:i as u32}], 1);
            }
            assert_eq!(9, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), height * 9);
        }, 9, 8);
    }

    #[test]
    fn remove_from_top() {
        run_test(|mut grid, width, _| {
            assert_eq!(grid.remove_from_top(2), vec![1; 16]);
            assert_eq!(width, grid.width);
            assert_eq!(6, grid.height);
            assert_eq!(grid.data.len(), width * 6);
        }, 8, 6);
    }

    #[test]
    fn remove_from_bottom() {
        run_test(|mut grid, width, _| {
            assert_eq!(grid.remove_from_bottom(2), vec![1; 16]);
            assert_eq!(width, grid.width);
            assert_eq!(6, grid.height);
            assert_eq!(grid.data.len(), width * 6);
        }, 8, 6);
    }

    #[test]
    fn remove_from_left() {
        run_test(|mut grid, _, height| {
            assert_eq!(grid.remove_from_left(2), vec![1; 16]);
            assert_eq!(6, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), height * 6);
        }, 6, 8)
    }

    #[test]
    fn remove_from_right() {
        run_test(|mut grid, _, height| {
            assert_eq!(grid.remove_from_right(2), vec![1; 16]);
            assert_eq!(6, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), height * 6);
        }, 6, 8)
    }

    #[test]
    fn scroll_left() {
        run_test(|mut grid, width, height| {
            grid.scroll(3, Left);
            for i in 0..grid.height {
                assert_eq!(grid[Coords {x:0, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:1, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:2, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:3, y:i as u32}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 10, 8);
    }

    #[test]
    fn scroll_right() {
        run_test(|mut grid, width, height| {
            grid.scroll(3, Right);
            for i in 0..grid.height {
                assert_eq!(grid[Coords {x:width as u32-1, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:width as u32-2, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:width as u32-3, y:i as u32}], 0);
                assert_eq!(grid[Coords {x:width as u32-4, y:i as u32}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 10, 8);
    }

    #[test]
    fn scroll_up() {
        run_test(|mut grid, width, height| {
            grid.scroll(3, Up);
            for i in 0..grid.width {
                assert_eq!(grid[Coords {x:i as u32, y:0}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:1}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:2}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:3}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 8, 10);
    }

    #[test]
    fn scroll_down() {
        run_test(|mut grid, width, height| {
            grid.scroll(3, Down);
            for i in 0..grid.width {
                assert_eq!(grid[Coords {x:i as u32, y:height as u32-1}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:height as u32-2}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:height as u32-3}], 0);
                assert_eq!(grid[Coords {x:i as u32, y:height as u32-4}], 1);
            }
            assert_eq!(width, grid.width);
            assert_eq!(height, grid.height);
            assert_eq!(grid.data.len(), width * height);
        }, 8, 10);
    }

    #[test]
    fn range() {
        const RANGE_TESTS: &'static [(Coords, Coords, &'static [u32])] = &[
            (Coords { x: 0, y: 0 }, Coords { x: 7, y: 0 }, &[0, 1, 2, 3, 4, 5, 6, 7]),
            (Coords { x: 6, y: 0 }, Coords { x: 1, y: 1 }, &[6, 7, 8, 9]),
            (Coords { x: 4, y: 7 }, Coords { x: 7, y: 7 }, &[60, 61, 62, 63]),
            (Coords { x: 7, y: 1 }, Coords { x: 0, y: 3 },
                 &[15, 16, 17, 18, 19, 20, 21, 22, 23, 24])
        ];
        let mut grid = Grid::new(8, 8);
        for (idx, cell) in (&mut grid).into_iter().enumerate() { *cell = idx as u32; }
        for &(coords1, coords2, values) in RANGE_TESTS {
            assert_eq!(&*grid.range_inclusive(coords1, coords2).cloned().collect::<Vec<_>>(),
                       values);
        }
    }

}
