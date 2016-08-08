use std::cmp;
use std::collections::HashMap;
use std::mem;
use std::ops::Index;
use std::sync::atomic::Ordering::Relaxed;

use cfg::SCROLLBACK;
use datatypes::{Coords, Direction, Style};
use terminal::{UseStyles, DEFAULT_STYLES};

mod cell;
mod data;
mod grid;
mod tooltip;

use self::cell::EMPTY_CELL;
use self::grid::Grid as DataGrid;

pub use self::cell::{CharCell, CellData, ImageData};
pub use self::data::*;
pub use self::tooltip::Tooltip;

const DEFAULT_CELL: &'static CharCell = &EMPTY_CELL;

pub trait Grid: Index<Coords> {
    fn new(width: u32, height: u32, retain_offscreen_state: bool) -> Self;

    fn resize_width(&mut self, width: u32);
    fn resize_height(&mut self, height: u32);

    fn write<T: CharData>(&mut self, coords: Coords, data: &T, styles: UseStyles) -> Coords;
    fn set_style(&mut self, coords: Coords, style: Style);
    fn reset_style(&mut self, coords: Coords);
    fn erase(&mut self, coords: Coords);
    fn moveover(&mut self, from: Coords, to: Coords);

    fn tooltip_at(&self, coords: Coords) -> Option<&Tooltip>;
    fn tooltip_at_mut(&mut self, coords: Coords) -> Option<&mut Tooltip>;

    fn move_out_of_extension(&self, coords: Coords, direction: Direction) -> Coords;
}

impl Grid for CharGrid {
    fn new(width: u32, height: u32, retain_offscreen_state: bool) -> CharGrid {
        let grid = match (retain_offscreen_state, SCROLLBACK.load(Relaxed)) {
            (false, _)          => DataGrid::with_x_y_caps(width as usize, height as usize),
            (_, n) if n > 0     => DataGrid::with_y_cap(cmp::min(n as usize, height as usize)),
            _                   => DataGrid::with_infinite_scroll(),
        };
        CharGrid {
            grid: grid,
            tooltips: HashMap::new(),
        }
    }

    fn resize_width(&mut self, width: u32) {
        self.grid.guarantee_width(width as usize);
    }

    fn resize_height(&mut self, height: u32) {
        self.grid.guarantee_height(height as usize);
    }

    fn write<T: CharData>(&mut self, coords: Coords, data: &T, styles: UseStyles) -> Coords {
        data.write(coords, styles, &mut self.grid)
    }

    fn set_style(&mut self, coords: Coords, style: Style) {
        self.grid.get_mut(coords).map(|cell| cell.styles.update(style));
    }
    
    fn reset_style(&mut self, coords: Coords) {
        self.grid.get_mut(coords).map(|cell| cell.styles = DEFAULT_STYLES);
    }

    fn moveover(&mut self, from: Coords, to: Coords) {
        if let Some(from) = self.grid.get_mut(from).map(|cell| mem::replace(cell, EMPTY_CELL)) {
            self.grid.fill_to(to);
            *self.grid.get_mut(to).unwrap() = from;
        }
    }

    fn erase(&mut self, coords: Coords) {
        self.grid.get_mut(coords).map(|cell| *cell = EMPTY_CELL);
    }

    fn tooltip_at(&self, coords: Coords) -> Option<&Tooltip> {
        self.tooltips.get(&coords)
    }

    fn tooltip_at_mut(&mut self, coords: Coords) -> Option<&mut Tooltip> {
        self.tooltips.get_mut(&coords)
    }

    fn move_out_of_extension(&self, mut coords: Coords, direction: Direction) -> Coords {
        fn up(Coords { x, y }: Coords) -> Coords    { Coords { x: x, y: y - 1 } }
        fn down(Coords { x, y }: Coords) -> Coords  { Coords { x: x, y: y + 1 } }
        fn left(Coords { x, y }: Coords) -> Coords  { Coords { x: x - 1, y: y } }
        fn right(Coords { x, y }: Coords) -> Coords { Coords { x: x + 1, y: y } }

        loop {
            match self.grid.get(coords).map(CharCell::is_extension) {
                Some(true)  => coords = match direction {
                    Direction::Up       => up(coords),
                    Direction::Down     => down(coords),
                    Direction::Left     => left(coords),
                    Direction::Right    => right(coords),
                },
                Some(false) => return coords,
                None        => {
                    return coords
                }
            }
        }
    }
}

pub struct CharGrid {
    grid: DataGrid<CharCell>,
    tooltips: HashMap<Coords, Tooltip>,
}

impl CharGrid {
    pub fn add_tooltip(&mut self, coords: Coords, tooltip: String) {
        self.tooltips.insert(coords, Tooltip::Basic(tooltip));
    }

    pub fn add_drop_down(&mut self, coords: Coords, options: Vec<String>) {
        self.tooltips.insert(coords, Tooltip::Menu { options: options, position: None });
    }

    pub fn remove_tooltip(&mut self, coords: Coords) {
        self.tooltips.remove(&coords);
    }

    pub fn scroll(&mut self, dir: Direction, n: u32) {
        self.grid.scroll(n as usize, dir)
    }
}

impl Index<Coords> for CharGrid {
    type Output = CharCell;
    
    fn index(&self, coords: Coords) -> &CharCell {
        self.grid.get(coords).unwrap_or(DEFAULT_CELL)
    }
}

