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
use std::collections::HashMap;
use std::ops::Index;
use std::sync::atomic::Ordering::Relaxed;

use unicode_width::*;

use cfg::SCROLLBACK;
use datatypes::{Area, CellData, Coords, CoordsIter, Direction, Movement, Region, Style, move_within};
use datatypes::Area::*;
use datatypes::Movement::*;
use datatypes::Direction::*;

mod cell;
mod cursor;
mod grid;
mod styles;
mod tooltip;
mod window;

pub use self::cell::{CharCell, CharData, ImageData, EMPTY_CELL};
pub use self::cursor::Cursor;
pub use self::grid::Grid;
pub use self::styles::{Styles, UseStyles, DEFAULT_STYLES};
pub use self::tooltip::Tooltip;
pub use self::window::{Window, View};

pub struct GridSettings {
    pub view: View,
    pub retain_offscreen_state: bool,
}

pub struct CharGrid {
    grid: Grid<CharCell>,
    cursor: Cursor,
    window: Window,
    tooltips: HashMap<Coords, Tooltip>,
}

impl CharGrid {
    pub fn new(width: u32, height: u32, settings: GridSettings) -> CharGrid {
        let grid = match (settings.retain_offscreen_state, SCROLLBACK.load(Relaxed)) {
            (false, _)          => Grid::with_x_y_caps(width as usize, height as usize),
            (_, n) if n > 0     => Grid::with_y_cap(cmp::min(n as usize, height as usize)),
            _                   => Grid::with_infinite_scroll(),
        };
        CharGrid {
            grid: grid,
            cursor: Cursor::new(),
            tooltips: HashMap::new(),
            window: Window::new(Coords { x: 0, y: 0 }, width, height, settings.view),
        }
    }

    pub fn resize_width(&mut self, width: u32) {
        self.grid.guarantee_width(width as usize);
        self.window.resize_width(width);
    }

    pub fn resize_height(&mut self, height: u32) {
        self.grid.guarantee_height(height as usize);
        self.window.resize_height(height);
    }

    pub fn write(&mut self, data: CellData) {
        match data {
            CellData::Char(c)       => {
                let width = c.width().unwrap() as u32;
                self.grid.write_at(self.cursor.coords, CharCell::character(c, self.cursor.text_style));
                let bounds = self.window.bounds();
                let mut coords = self.cursor.coords;
                for _ in 1..width {
                    let next_coords = move_within(coords, To(Right, 1, false), bounds);
                    if next_coords == coords { break; } else { coords = next_coords; }
                    self.grid.write_at(coords, CharCell::extension(self.cursor.coords,
                                                                   self.cursor.text_style));
                }
                self.move_cursor(To(Right, 1, true));
            }
            CellData::ExtensionChar(c)  => {
                self.move_cursor(To(Left, 1, true));
                if !self.grid.get_mut(self.cursor.coords).map_or(false, |cell| cell.extend_by(c)) {
                    self.move_cursor(To(Right, 1, true));
                    self.grid.write_at(self.cursor.coords, CharCell::character(c, self.cursor.text_style));
                    self.move_cursor(To(Right, 1, true));
                }
            }
            CellData::Image { pos, width, height, data, mime }   => {
                let mut end = self.cursor.coords;
                end = move_within(end, To(Right, width, false), self.window.bounds());
                end = move_within(end, To(Down, height, false), self.window.bounds());
                let mut iter = CoordsIter::from_area(CursorBound(end),
                                                     self.cursor.coords, self.window.bounds());
                if let Some(cu_coords) = iter.next() {
                    self.grid.write_at(cu_coords, CharCell::image(data, self.cursor.coords, mime,
                                                                  pos, width, height,
                                                                  self.cursor.text_style));
                    for coords in iter {
                        self.grid.write_at(coords, CharCell::extension(cu_coords, self.cursor.text_style));
                    }
                    self.move_cursor(To(Right, 1, true));
                }
            }
        }
    }

    pub fn move_cursor(&mut self, movement: Movement) {
        self.cursor.navigate(&mut self.grid, self.window.bounds(), movement)
    }

    pub fn add_tooltip(&mut self, coords: Coords, tooltip: String) {
        self.tooltips.insert(coords, Tooltip::Basic(tooltip));
    }

    pub fn remove_tooltip(&mut self, coords: Coords) {
        self.tooltips.remove(&coords);
    }

    pub fn add_drop_down(&mut self, coords: Coords, options: Vec<String>) {
        self.tooltips.insert(coords, Tooltip::Menu { options: options, position: None });
    }

    pub fn scroll(&mut self, dir: Direction, n: u32) {
        self.grid.scroll(n as usize, dir)
    }

    pub fn erase(&mut self, area: Area) {
        self.in_area(area, |grid, coords| grid.write_at(coords, CharCell::default()));
    }

    pub fn insert_blank_at(&mut self, n: u32) {
        let mut iter = CoordsIter::from_area(CursorTo(ToEdge(Right)),
                                             self.cursor.coords,
                                             self.window.bounds());
        iter.next();
        for coords in iter.rev().skip(n as usize) {
            self.grid.moveover(coords, Coords {x: coords.x + n, y: coords.y});
        }
    }

    pub fn remove_at(&mut self, n: u32) {
        self.in_area(CursorTo(ToEdge(Right)), |grid, coords| {
            if coords.x + n < grid.width as u32 {
                grid.moveover(Coords {x: coords.x + n, y: coords.y}, coords);
            }
        })
    }

    pub fn insert_rows_at(&mut self, n: u32, include: bool) {
        let region = if include {
            Region::new(0, self.cursor.coords.y, self.grid.width as u32, self.grid.height as u32)
        } else if self.cursor.coords.y + 1 == self.grid.width as u32 {
            return
        } else {
            Region::new(0, self.cursor.coords.y + 1, self.grid.width as u32, self.grid.height as u32)
        };
        for coords in CoordsIter::from_region(region).rev().skip(n as usize * self.grid.width) {
            self.grid.moveover(coords, Coords {x: coords.x, y: coords.y + n});
        }
    }

    pub fn remove_rows_at(&mut self, n: u32, include: bool) {
        self.in_area(BelowCursor(include), |grid, coords| {
            if coords.y + n < grid.height as u32 {
                grid.moveover(Coords {x: coords.x, y: coords.y + n}, coords);
            }
        })
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

    pub fn set_style_in_area(&mut self, area: Area, style: Style) {
        self.in_area(area, |grid, coords| {
            grid.get_mut(coords).map(|cell| cell.styles.update(style));
        });
    }

    pub fn reset_styles_in_area(&mut self, area: Area) {
        self.in_area(area, |grid, coords| {
            grid.get_mut(coords).map(|cell| cell.styles = UseStyles::default());
        });
    }

    pub fn cursor_position(&self) -> Coords {
        self.cursor.coords
    }

    pub fn cursor_styles(&self) -> Styles {
        self.cursor.style
    }

    pub fn chars_in_range(&self, start: Coords, end: Coords) -> String {
        CoordsIter::from_area(
            Area::CursorTo(Movement::Position(end)),
            start,
            self.window.bounds(),
        ).fold(String::new(), |s, coords| {
            let cell_data = self.grid.get(coords).map(CharCell::to_string).unwrap_or_else(String::new);
            if coords.x == 0 && !s.is_empty() { s + "\n" + &cell_data }
            else { s + &cell_data }
        })
    }

    pub fn grid_width(&self) -> u32 {
        self.grid.width as u32
    }

    pub fn grid_height(&self) -> u32 {
        self.grid.height as u32
    }

    pub fn tooltip_at(&self, coords: Coords) -> Option<&Tooltip> {
        self.tooltips.get(&coords)
    }

    pub fn tooltip_at_mut(&mut self, coords: Coords) -> Option<&mut Tooltip> {
        self.tooltips.get_mut(&coords)
    }

    fn in_area<F>(&mut self, area: Area, f: F) where F: Fn(&mut Grid<CharCell>, Coords) {
        for coords in CoordsIter::from_area(area, self.cursor.coords, self.window.bounds()) {
            f(&mut self.grid, coords);
        }
    }

}

impl<'a> IntoIterator for &'a CharGrid {
    type IntoIter = <&'a Grid<CharCell> as IntoIterator>::IntoIter;
    type Item = &'a CharCell;
    fn into_iter(self) -> Self::IntoIter {
        self.grid.into_iter()
    }
}

impl Index<Coords> for CharGrid {
    type Output = CharCell;
    
    fn index(&self, coords: Coords) -> &CharCell {
        const DEFAULT: &'static CharCell = &EMPTY_CELL;
        self.grid.get(coords).unwrap_or(DEFAULT)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::atomic::Ordering::Relaxed;

    use super::*;
    use datatypes::{CellData, Coords, Direction, Movement, Region};

    fn run_test<F: Fn(CharGrid, u32)>(test: F) {
        ::cfg::TAB_STOP.store(4, Relaxed);
        ::cfg::SCROLLBACK.store(-1, Relaxed);
        test(CharGrid::new(10, 10, false), 10);
        test(CharGrid::new(10, 10, true), 11);
    }

    #[test]
    fn window_scrolls_with_cursor() {
        run_test(|mut grid, h| {
            grid.move_cursor(Movement::NextLine(10));
            assert_eq!(grid.window, Region::new(0, h - 10, 10, h));
        })
    }

    #[test]
    fn write() {
        run_test(|mut grid, _| {
            for c in vec![
                CellData::Char('Q'),
                CellData::Char('E'),
                CellData::ExtensionChar('\u{301}'),
            ].into_iter() { grid.write(c); }
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "Q");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "E\u{301}");
        });
    }

    fn setup(grid: &mut CharGrid) {
        let mut chars = vec![
            CellData::Char('A'),
            CellData::Char('B'),
            CellData::Char('C'),
            CellData::Char('D'),
            CellData::Char('E'),
            CellData::Char('1'),
            CellData::Char('2'),
            CellData::Char('3'),
            CellData::Char('4'),
            CellData::Char('5'),
            CellData::Char('!'),
            CellData::Char('@'),
            CellData::Char('#'),
            CellData::Char('$'),
            CellData::Char('%'),
        ].into_iter();
        for _ in 0..3 {
            for c in chars.by_ref().take(5) { grid.write(c); }
            grid.move_cursor(Movement::NextLine(1));
        }
        grid.move_cursor(Movement::ToBeginning);
    }

    #[test]
    fn move_cursor() {
        run_test(|mut grid, h| {
            let movements = vec![
                (Movement::ToEdge(Direction::Down), Coords {x:0, y:9}),
                (Movement::Tab(Direction::Right, 1, false), Coords{x: 4, y:9}),
                (Movement::NextLine(1), Coords{x:0, y:h-1}),
            ];
            for (mov, coords) in movements {
                grid.move_cursor(mov);
                assert_eq!(grid.cursor_position(), coords);
            }
            assert_eq!(grid.grid.height as u32, h);
        })
    }

    #[test]
    fn insert_blank_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.insert_blank_at(1);
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "A");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:0}].repr(), "B");
            assert_eq!(grid.grid[Coords {x:3, y:0}].repr(), "C");
            assert_eq!(grid.grid[Coords {x:4, y:0}].repr(), "D");
            assert_eq!(grid.grid[Coords {x:5, y:0}].repr(), "E");
            grid.move_cursor(Movement::NextLine(1));
            grid.insert_blank_at(2);
            assert_eq!(grid.grid[Coords {x:0, y:1}].repr(), "1");
            assert_eq!(grid.grid[Coords {x:1, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:1}].repr(), "2");
            assert_eq!(grid.grid[Coords {x:4, y:1}].repr(), "3");
            assert_eq!(grid.grid[Coords {x:5, y:1}].repr(), "4");
            assert_eq!(grid.grid[Coords {x:6, y:1}].repr(), "5");
            grid.move_cursor(Movement::NextLine(1));
            grid.insert_blank_at(3);
            assert_eq!(grid.grid[Coords {x:0, y:2}].repr(), "!");
            assert_eq!(grid.grid[Coords {x:1, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:2}].repr(), "@");
            assert_eq!(grid.grid[Coords {x:5, y:2}].repr(), "#");
            assert_eq!(grid.grid[Coords {x:6, y:2}].repr(), "$");
            assert_eq!(grid.grid[Coords {x:7, y:2}].repr(), "%");
        })
    }

    #[test]
    fn remove_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.remove_at(1);
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "B");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "C");
            assert_eq!(grid.grid[Coords {x:2, y:0}].repr(), "D");
            assert_eq!(grid.grid[Coords {x:3, y:0}].repr(), "E");
            assert_eq!(grid.grid[Coords {x:4, y:0}].repr(), "");
            grid.move_cursor(Movement::NextLine(1));
            grid.remove_at(2);
            assert_eq!(grid.grid[Coords {x:0, y:1}].repr(), "3");
            assert_eq!(grid.grid[Coords {x:1, y:1}].repr(), "4");
            assert_eq!(grid.grid[Coords {x:2, y:1}].repr(), "5");
            assert_eq!(grid.grid[Coords {x:3, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:1}].repr(), "");
            grid.move_cursor(Movement::NextLine(1));
            grid.remove_at(3);
            assert_eq!(grid.grid[Coords {x:0, y:2}].repr(), "$");
            assert_eq!(grid.grid[Coords {x:1, y:2}].repr(), "%");
            assert_eq!(grid.grid[Coords {x:2, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:2}].repr(), "");
        })
    }

    #[test]
    fn insert_rows_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.insert_rows_at(2, false);
            assert_eq!(grid.grid[Coords {x:0, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:3}].repr(), "1");
            assert_eq!(grid.grid[Coords {x:1, y:3}].repr(), "2");
            assert_eq!(grid.grid[Coords {x:2, y:3}].repr(), "3");
            assert_eq!(grid.grid[Coords {x:3, y:3}].repr(), "4");
            assert_eq!(grid.grid[Coords {x:4, y:3}].repr(), "5");
            grid.insert_rows_at(3, true);
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:0}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:1}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:1, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:2, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:3, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:4, y:2}].repr(), "");
            assert_eq!(grid.grid[Coords {x:0, y:3}].repr(), "A");
            assert_eq!(grid.grid[Coords {x:1, y:3}].repr(), "B");
            assert_eq!(grid.grid[Coords {x:2, y:3}].repr(), "C");
            assert_eq!(grid.grid[Coords {x:3, y:3}].repr(), "D");
            assert_eq!(grid.grid[Coords {x:4, y:3}].repr(), "E");
        })
    }

    #[test]
    fn remove_rows_at() {
        run_test(|mut grid, _| {
            setup(&mut grid);
            grid.remove_rows_at(2, true);
            assert_eq!(grid.grid[Coords {x:0, y:0}].repr(), "!");
            assert_eq!(grid.grid[Coords {x:1, y:0}].repr(), "@");
            assert_eq!(grid.grid[Coords {x:2, y:0}].repr(), "#");
            assert_eq!(grid.grid[Coords {x:3, y:0}].repr(), "$");
            assert_eq!(grid.grid[Coords {x:4, y:0}].repr(), "%");
        })
    }

}
