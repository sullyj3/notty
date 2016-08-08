use datatypes::{Coords, Region};
use terminal::{CharCell, CellData, UseStyles};
use terminal::char_grid::grid::Grid;

mod character;
mod image;
#[cfg(test)]
mod tests;

pub use self::character::*;
pub use self::image::*;

pub trait CharData: Send + 'static {
    fn write<T: WritableGrid>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords;
    fn repr(&self) -> String {
        String::from("DATA")
    }
}

pub trait WritableGrid {
    type Cell: WritableCell;
    fn writable(&mut self, coords: Coords) -> Option<&mut Self::Cell>;
    fn best_fit_for_region(&self, region: Region) -> Coords;
    fn find_cell_to_extend(&self, coords: Coords) -> Option<Coords>;
}

pub trait WritableCell { 
    fn write(&mut self, data: CellData, styles: UseStyles);
    fn extend(&mut self, c: char, styles: UseStyles);
    fn is_extendable(&self) -> bool;
    fn is_extension_of(&self) -> Option<Coords>;
}

impl<T: WritableCell + Default> WritableGrid for Grid<T> {
    type Cell = T;

    fn writable(&mut self, coords: Coords) -> Option<&mut T> {
        self.fill_to(coords);
        self.get_mut(coords)
    }

    fn best_fit_for_region(&self, region: Region) -> Coords {
        let x_offset = self.max_width().map_or(0, |width| {
            region.right.saturating_sub(width as u32)
        });
        let y_offset = self.max_width().map_or(0, |height| {
            region.bottom.saturating_sub(height as u32)
        });
        Coords { x: region.left - x_offset, y: region.top - y_offset }
    }

    fn find_cell_to_extend(&self, coords: Coords) -> Option<Coords> {
        cell_to_extend(self, coords_before(coords, self.width as u32))
    }
}

impl WritableCell for CharCell {
    fn write(&mut self, data: CellData, styles: UseStyles) {
        self.content = data;
        self.styles = styles;
    }

    fn extend(&mut self, extension: char, styles: UseStyles) {
        if let CellData::Char(c) = self.content {
            self.content = CellData::Grapheme(format!("{}{}", c, extension));
            self.styles = styles;
        } else if let CellData::Grapheme(ref mut s) = self.content {
            s.push(extension);
            self.styles = styles;
        }
    }

    fn is_extendable(&self) -> bool {
        match self.content {
            CellData::Char(_) | CellData::Grapheme(_)   => true,
            _                                           => false,
        }
    }

    fn is_extension_of(&self) -> Option<Coords> {
        match self.content {
            CellData::Extension(coords) => Some(coords),
            _                           => None,
        }
    }
}

fn region_at(Coords { x, y }: Coords, width: u32, height: u32) -> Region {
    Region::new(x, y, x + width, y + height)
}

pub fn coords_before(Coords { x, y }: Coords, width: u32) -> Coords {
    match (x == 0, y == 0) {
        (true, true)    => Coords { x: x, y: y },
        (true, _)       => Coords { x: width - 1, y: y - 1},
        (_, _)          => Coords { x: x - 1, y: y },
    }
}

pub fn cell_to_extend<T: WritableCell>(grid: &Grid<T>, coords: Coords) -> Option<Coords> {
    if let Some(cell) = grid.get(coords) {
        if cell.is_extendable() {
            Some(coords)
        } else if let Some(coords) = cell.is_extension_of() {
            cell_to_extend(grid, coords)
        } else { None }
    } else { None }
}
