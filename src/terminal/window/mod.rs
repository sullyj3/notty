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
use std::ops::{Index, Deref, DerefMut};

use datatypes::{Area, Coords, CoordsIter, Direction, Movement, Style, move_within};

use terminal::{CharGrid, Grid, Styles, UseStyles, Tooltip};

mod cursor;
mod view;

pub use self::cursor::Cursor;
pub use self::view::Flow;

use self::view::View;

const TO_RIGHT_EDGE: Area = Area::CursorTo(Movement::ToEdge(Direction::Right));

pub struct GridSettings {
    pub flow: Flow,
    pub retain_offscreen_state: bool,
}

pub struct Window<T: Grid = CharGrid> {
    grid: T,
    cursor: Cursor,
    view: View,
}

impl<T: Grid> Window<T> {
    pub fn new(width: u32, height: u32, settings: GridSettings) -> Window<T> {
        Window {
            grid: T::new(width, height, settings.retain_offscreen_state),
            cursor: Cursor::new(),
            view: View::new(Coords { x: 0, y: 0 }, width, height, settings.flow),
        }
    }

    pub fn resize_width(&mut self, width: u32) {
        self.grid.resize_width(width);
        self.view.resize_width(width);
    }

    pub fn resize_height(&mut self, height: u32) {
        self.grid.resize_height(height);
        self.view.resize_height(height);
    }

    pub fn write(&mut self, data: T::Data) {
        self.cursor.coords = self.grid.write(self.cursor.coords, data, self.cursor.text_style);
        self.move_cursor(Movement::To(Direction::Right, 1, true));
    }

    pub fn move_cursor(&mut self, movement: Movement) {
        self.cursor.coords = self.calculate_movement(self.cursor.coords, movement);
        self.view.keep_cursor_within(self.cursor.coords);
    }

    pub fn set_style(&mut self, style: Style) {
        self.cursor.text_style.update(style);
    }

    pub fn reset_styles(&mut self) {
        self.cursor.text_style = UseStyles::default();
    }

    pub fn set_cursor_style(&mut self, style: Style) {
        self.cursor.style.update(style);
    }

    pub fn reset_cursor_styles(&mut self) {
        self.cursor.style = Styles::new();
    }

    pub fn cursor_position(&self) -> Coords {
        self.cursor.coords
    }

    pub fn cursor_styles(&self) -> Styles {
        self.cursor.style
    }

    fn iterate_over_area(&self, area: Area) -> CoordsIter {
        CoordsIter::from_area(area, self.cursor.coords, self.view.bounds())
    }

    fn calculate_movement(&self, coords: Coords, movement: Movement) -> Coords {
        // First, calculate the movement within the bounds of the view
        let new_coords = move_within(self.cursor.coords, movement, self.view.bounds());
        // Then, avoid a movement which would land in an extension cell
        self.grid.move_out_of_extension(new_coords, movement.direction(coords))
    }

    pub fn set_style_in_area(&mut self, area: Area, style: Style) {
        for coords in self.iterate_over_area(area) {
            self.grid.set_style(coords, style);
        }
    }

    pub fn reset_styles_in_area(&mut self, area: Area) {
        for coords in self.iterate_over_area(area) {
            self.grid.reset_style(coords);
        }
    }

    pub fn erase(&mut self, area: Area) {
        for coords in self.iterate_over_area(area) {
            self.grid.erase(coords);
        }
    }

    pub fn insert_blank_at(&mut self, n: u32) {
        let iter = self.iterate_over_area(TO_RIGHT_EDGE);
        let Window { ref mut grid, ref view, .. } = * self;
        let iter = iter.rev().skip(n as usize)
                       .map(|coords| view.translate(coords));
        for coords in iter {
            grid.moveover(coords, Coords { x: coords.x + n, y: coords.y });
        }
    }

    pub fn remove_at(&mut self, n: u32) {
        let iter = self.iterate_over_area(TO_RIGHT_EDGE);
        let Window { ref mut grid, ref view, .. } = *self;
        let iter = iter.take_while(|&Coords { x, .. }| x + n < view.width())
                       .map(|coords| view.translate(coords));
        for coords in iter {
            grid.moveover(Coords { x: coords.x + n, y: coords.y }, coords);
        }
    }

    pub fn insert_rows_at(&mut self, n: u32, include: bool) {
        let iter = self.iterate_over_area(Area::BelowCursor(include));
        let Window { ref mut grid, ref view, .. } = *self;
        let iter = iter.rev().skip((n * view.width()) as usize)
                       .map(|coords| view.translate(coords));
        for coords in iter {
            grid.moveover(coords, Coords { x: coords.x, y: coords.y + n });
        }
    }

    pub fn remove_rows_at(&mut self, n: u32, include: bool) {
        let iter = self.iterate_over_area(Area::BelowCursor(include));
        let Window { ref mut grid, ref view, .. } = *self;
        let iter = iter.take_while(|&Coords { y, .. }| y + n < view.height())
                       .map(|coords| view.translate(coords));
        for coords in iter {
            grid.moveover(Coords { x: coords.x, y: coords.y + n }, coords);
        }
    }

    pub fn tooltip_at(&self, coords: Coords) -> Option<&Tooltip> {
        self.grid.tooltip_at(self.view.translate(coords))
    }

    pub fn tooltip_at_mut(&mut self, coords: Coords) -> Option<&mut Tooltip> {
        self.grid.tooltip_at_mut(self.view.translate(coords))
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

impl<T: Grid> Index<Coords> for Window<T> {
    type Output = T::Output;

    fn index(&self, coords: Coords) -> &T::Output {
        &self.grid[self.view.translate(coords)]
    }
}

//#[cfg(test)]
//mod tests {
//
//    use std::sync::atomic::Ordering::Relaxed;
//
//    use super::*;
//    use datatypes::{CharData, Coords, Direction, Movement, Region};
//
//    fn settings(expand: bool) -> GridSettings {
//        GridSettings {
//            retain_offscreen_state: expand,
//            view: View::Moveable,
//        }
//    }
//
//    fn run_test<F: Fn(Window, u32)>(test: F) {
//        ::cfg::TAB_STOP.store(4, Relaxed);
//        ::cfg::SCROLLBACK.store(-1, Relaxed);
//        test(Window::new(10, 10, settings(false)), 10);
//        test(Window::new(10, 10, settings(true)), 11);
//    }
//
//    #[test]
//    fn view_scrolls_with_cursor() {
//        run_test(|mut grid, h| {
//            grid.move_cursor(Movement::NextLine(10));
//            assert_eq!(grid.view, Region::new(0, h - 10, 10, h));
//        })
//    }
//
//    #[test]
//    fn write() {
//        run_test(|mut grid, _| {
//            for c in vec![
//                CharData::Char('Q'),
//                CharData::Char('E'),
//                CharData::ExtensionChar('\u{301}'),
//            ].into_iter() { grid.write(c); }
//            assert_eq!(grid.grid.get(Coords {x:0, y:0}).unwrap().repr(), "Q");
//            assert_eq!(grid.grid.get(Coords {x:1, y:0}).unwrap().repr(), "E\u{301}");
//        });
//    }
//
//    fn setup(grid: &mut Window) {
//        let mut chars = vec![
//            CharData::Char('A'),
//            CharData::Char('B'),
//            CharData::Char('C'),
//            CharData::Char('D'),
//            CharData::Char('E'),
//            CharData::Char('1'),
//            CharData::Char('2'),
//            CharData::Char('3'),
//            CharData::Char('4'),
//            CharData::Char('5'),
//            CharData::Char('!'),
//            CharData::Char('@'),
//            CharData::Char('#'),
//            CharData::Char('$'),
//            CharData::Char('%'),
//        ].into_iter();
//        for _ in 0..3 {
//            for c in chars.by_ref().take(5) { grid.write(c); }
//            grid.move_cursor(Movement::NextLine(1));
//        }
//        grid.move_cursor(Movement::ToBeginning);
//    }
//
//    #[test]
//    fn move_cursor() {
//        run_test(|mut grid, h| {
//            let movements = vec![
//                (Movement::ToEdge(Direction::Down), Coords {x:0, y:9}),
//                (Movement::Tab(Direction::Right, 1, false), Coords{x: 4, y:9}),
//                (Movement::NextLine(1), Coords{x:0, y:h-1}),
//            ];
//            for (mov, coords) in movements {
//                grid.move_cursor(mov);
//                assert_eq!(grid.cursor_position(), coords);
//            }
//            assert_eq!(grid.grid.height as u32, h);
//        })
//    }
//
//    #[test]
//    fn insert_blank_at() {
//        run_test(|mut grid, _| {
//            setup(&mut grid);
//            grid.insert_blank_at(1);
//            assert_eq!(grid.grid.get(Coords {x:0, y:0}).unwrap().repr(), "A");
//            assert_eq!(grid.grid.get(Coords {x:1, y:0}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:2, y:0}).unwrap().repr(), "B");
//            assert_eq!(grid.grid.get(Coords {x:3, y:0}).unwrap().repr(), "C");
//            assert_eq!(grid.grid.get(Coords {x:4, y:0}).unwrap().repr(), "D");
//            assert_eq!(grid.grid.get(Coords {x:5, y:0}).unwrap().repr(), "E");
//            grid.move_cursor(Movement::NextLine(1));
//            grid.insert_blank_at(2);
//            assert_eq!(grid.grid.get(Coords {x:0, y:1}).unwrap().repr(), "1");
//            assert_eq!(grid.grid.get(Coords {x:1, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:2, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:3, y:1}).unwrap().repr(), "2");
//            assert_eq!(grid.grid.get(Coords {x:4, y:1}).unwrap().repr(), "3");
//            assert_eq!(grid.grid.get(Coords {x:5, y:1}).unwrap().repr(), "4");
//            assert_eq!(grid.grid.get(Coords {x:6, y:1}).unwrap().repr(), "5");
//            grid.move_cursor(Movement::NextLine(1));
//            grid.insert_blank_at(3);
//            assert_eq!(grid.grid.get(Coords {x:0, y:2}).unwrap().repr(), "!");
//            assert_eq!(grid.grid.get(Coords {x:1, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:2, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:3, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:4, y:2}).unwrap().repr(), "@");
//            assert_eq!(grid.grid.get(Coords {x:5, y:2}).unwrap().repr(), "#");
//            assert_eq!(grid.grid.get(Coords {x:6, y:2}).unwrap().repr(), "$");
//            assert_eq!(grid.grid.get(Coords {x:7, y:2}).unwrap().repr(), "%");
//        })
//    }
//
//    #[test]
//    fn remove_at() {
//        run_test(|mut grid, _| {
//            setup(&mut grid);
//            grid.remove_at(1);
//            assert_eq!(grid.grid.get(Coords {x:0, y:0}).unwrap().repr(), "B");
//            assert_eq!(grid.grid.get(Coords {x:1, y:0}).unwrap().repr(), "C");
//            assert_eq!(grid.grid.get(Coords {x:2, y:0}).unwrap().repr(), "D");
//            assert_eq!(grid.grid.get(Coords {x:3, y:0}).unwrap().repr(), "E");
//            assert_eq!(grid.grid.get(Coords {x:4, y:0}).unwrap().repr(), "");
//            grid.move_cursor(Movement::NextLine(1));
//            grid.remove_at(2);
//            assert_eq!(grid.grid.get(Coords {x:0, y:1}).unwrap().repr(), "3");
//            assert_eq!(grid.grid.get(Coords {x:1, y:1}).unwrap().repr(), "4");
//            assert_eq!(grid.grid.get(Coords {x:2, y:1}).unwrap().repr(), "5");
//            assert_eq!(grid.grid.get(Coords {x:3, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:4, y:1}).unwrap().repr(), "");
//            grid.move_cursor(Movement::NextLine(1));
//            grid.remove_at(3);
//            assert_eq!(grid.grid.get(Coords {x:0, y:2}).unwrap().repr(), "$");
//            assert_eq!(grid.grid.get(Coords {x:1, y:2}).unwrap().repr(), "%");
//            assert_eq!(grid.grid.get(Coords {x:2, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:3, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:4, y:2}).unwrap().repr(), "");
//        })
//    }
//
//    #[test]
//    fn insert_rows_at() {
//        run_test(|mut grid, _| {
//            setup(&mut grid);
//            grid.insert_rows_at(2, false);
//            assert_eq!(grid.grid.get(Coords {x:0, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:1, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:2, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:3, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:4, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:0, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:1, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:2, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:3, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:4, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:0, y:3}).unwrap().repr(), "1");
//            assert_eq!(grid.grid.get(Coords {x:1, y:3}).unwrap().repr(), "2");
//            assert_eq!(grid.grid.get(Coords {x:2, y:3}).unwrap().repr(), "3");
//            assert_eq!(grid.grid.get(Coords {x:3, y:3}).unwrap().repr(), "4");
//            assert_eq!(grid.grid.get(Coords {x:4, y:3}).unwrap().repr(), "5");
//            grid.insert_rows_at(3, true);
//            assert_eq!(grid.grid.get(Coords {x:0, y:0}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:1, y:0}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:2, y:0}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:3, y:0}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:4, y:0}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:0, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:1, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:2, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:3, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:4, y:1}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:0, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:1, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:2, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:3, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:4, y:2}).unwrap().repr(), "");
//            assert_eq!(grid.grid.get(Coords {x:0, y:3}).unwrap().repr(), "A");
//            assert_eq!(grid.grid.get(Coords {x:1, y:3}).unwrap().repr(), "B");
//            assert_eq!(grid.grid.get(Coords {x:2, y:3}).unwrap().repr(), "C");
//            assert_eq!(grid.grid.get(Coords {x:3, y:3}).unwrap().repr(), "D");
//            assert_eq!(grid.grid.get(Coords {x:4, y:3}).unwrap().repr(), "E");
//        })
//    }
//
//    #[test]
//    fn remove_rows_at() {
//        run_test(|mut grid, _| {
//            setup(&mut grid);
//            grid.remove_rows_at(2, true);
//            assert_eq!(grid.grid.get(Coords {x:0, y:0}).unwrap().repr(), "!");
//            assert_eq!(grid.grid.get(Coords {x:1, y:0}).unwrap().repr(), "@");
//            assert_eq!(grid.grid.get(Coords {x:2, y:0}).unwrap().repr(), "#");
//            assert_eq!(grid.grid.get(Coords {x:3, y:0}).unwrap().repr(), "$");
//            assert_eq!(grid.grid.get(Coords {x:4, y:0}).unwrap().repr(), "%");
//        })
//    }
//
//}
