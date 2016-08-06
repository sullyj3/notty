use unicode_width::UnicodeWidthChar;

use datatypes::{CharData, Coords, CoordsIter, Region};
use terminal::UseStyles;
use super::{CellData, CharCell, CharGrid, CellModifier};
use super::grid::Grid;

pub struct Writer {
    iter: Iter,
}

impl Writer {
    pub fn new(data: CharData, coords: Coords, grid: &CharGrid) -> Writer {
        match data {
            CharData::Char(c)           => {
                let modifier = CellModifier::Char(c);
                let width = c.width().unwrap() as u32;
                let coords = grid.best_fit_for_region(region_at(coords, width, 1));
                let iter = CoordsIter::from(region_at(coords, width, 1));
                Writer {
                    iter: Iter {
                        main: Some(modifier),
                        main_coords: coords,
                        iter: iter,
                    },
                }
            }
            CharData::ExtensionChar(c)  => {
                match find_coords_to_modify_char(coords, grid) {
                    Some(coords)                                => {
                        let modifier = CellModifier::ChExtend(c);
                        let iter = CoordsIter::from(region_at(coords, 1, 1));
                        Writer {
                            iter: Iter {
                                main: Some(modifier),
                                main_coords: coords,
                                iter: iter,
                            },
                        }
                    }
                    None                                        => {
                        let modifier = CellModifier::Char(c);
                        let iter = CoordsIter::from(region_at(coords, 1, 1));
                        Writer {
                            iter: Iter {
                                main: Some(modifier),
                                main_coords: coords,
                                iter: iter,
                            },
                        }
                    }
                }
            }
            CharData::Image { data, mime, pos, width, height }  => {
                let modifier = CellModifier::Image(data, mime, pos, width, height);
                let coords = grid.best_fit_for_region(region_at(coords, width, height));
                let iter = CoordsIter::from(region_at(coords, width, height));
                Writer {
                    iter: Iter {
                        main: Some(modifier),
                        main_coords: coords,
                        iter: iter,
                    }
                }
            }
        }
    }

    pub fn write(self, grid: &mut Grid<CharCell>, styles: UseStyles) -> Coords {
        let y = self.iter.main_coords.y;
        let x = self.iter.flat_map(|(modifier, coords)| {
            grid.fill_to(coords);
            grid.get_mut(coords).map(|cell| {
                cell.mod_content(modifier, coords);
                cell.styles = styles;
                coords.x
            })
        }).last().unwrap();
        Coords { x: x, y: y }
    }
}

struct Iter {
    main: Option<CellModifier>,
    main_coords: Coords,
    iter: CoordsIter,
}

impl Iterator for Iter {
    type Item = (CellModifier, Coords);
    fn next(&mut self) -> Option<(CellModifier, Coords)> {
        self.iter.next().map(|coords| {
            let modifier = self.main.take().unwrap_or(CellModifier::Extension(self.main_coords));
            (modifier, coords)
        })
    }
}

fn find_coords_to_modify_char(coords: Coords, grid: &CharGrid) -> Option<Coords> {
    let previous_coords = grid.coords_before(coords);
    match grid[previous_coords].content {
        CellData::Char(_) | CellData::Grapheme(_)   => Some(previous_coords),
        CellData::Extension(previous_coords)        => find_coords_to_modify_char(previous_coords, grid),
        _                                           => None,
    }
}

fn region_at(Coords { x, y }: Coords, width: u32, height: u32) -> Region {
    Region::new(x, y, x + width, y + height)
}

#[cfg(test)]
mod tests {

    use super::super::CellModifier;
    use datatypes::*;

    #[test]
    fn iterator() {
        const TEST: &'static [(CellModifier, Coords)] = &[
            (CellModifier::Char('\0'), Coords { x: 3, y: 4 }),
            (CellModifier::Extension(Coords { x: 3, y: 4 }), Coords { x: 4, y: 4 }),
            (CellModifier::Extension(Coords { x: 3, y: 4 }), Coords { x: 3, y: 5 }),
            (CellModifier::Extension(Coords { x: 3, y: 4 }), Coords { x: 4, y: 5 }),
        ];
        let iter = super::Iter {
            main: Some(CellModifier::Char('\0')),
            main_coords: Coords { x: 3, y: 4 },
            iter: CoordsIter::from(Region::new(3, 4, 5, 6)),
        };
        for (left, right) in iter.zip(TEST) {
            assert_eq!(left, *right)
        }
    }

    // TODO test that writer works around edges of the grid
}
