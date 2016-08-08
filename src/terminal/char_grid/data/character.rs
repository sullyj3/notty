use datatypes::{Coords};
use terminal::{CharData, CellData, UseStyles};
use super::{WritableGrid, WritableCell};

impl CharData for char {
    fn write<T: WritableGrid>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords {
        if let Some(cell) = grid.writable(coords) {
            cell.write(CellData::Char(*self), styles);
        }
        coords
    }

    fn repr(&self) -> String {
        self.to_string()
    }
}

pub struct WideChar(pub char, pub u32);

impl WideChar {
    pub fn new(ch: char, width: u32) -> WideChar {
        WideChar(ch, width)
    }
}

impl CharData for WideChar {
    fn write<T: WritableGrid>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords {
        let coords = grid.best_fit_for_region(super::region_at(coords, self.1, 1));
        if let Some(cell) = grid.writable(coords) {
            cell.write(CellData::Char(self.0), styles);
        }
        for extension_coords in (1..self.1).map(|i| Coords { x: coords.x + i, ..coords }) {
            if let Some(cell) = grid.writable(extension_coords) {
                cell.write(CellData::Extension(coords), styles)
            }
        }
        Coords { x: coords.x + self.1 - 1, y: coords.y }
    }

    fn repr(&self) -> String {
        self.0.to_string()
    }
}

pub struct CharExtender(pub char);

impl CharExtender {
    pub fn new(ch: char) -> CharExtender {
        CharExtender(ch)
    }
}

impl CharData for CharExtender {
    fn write<T: WritableGrid>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords {
        match grid.find_cell_to_extend(coords) {
            Some(coords)    => {
                if let Some(cell) = grid.writable(coords) {
                    cell.extend(self.0, styles);
                }
                coords
            }
            None            => {
                if let Some(cell) = grid.writable(coords) {
                    cell.write(CellData::Char(self.0), styles);
                }
                coords
            }
        }
    }

    fn repr(&self) -> String {
        self.0.to_string()
    }
}
