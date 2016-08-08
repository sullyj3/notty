pub use datatypes::{Coords, Region};
pub use terminal::{CellData, CharCell, UseStyles};

pub use super::*;

pub const COORDS: Coords = Coords { x: 7, y: 19 };
pub const STYLES: UseStyles = ::terminal::styles::DEFAULT_STYLES;

mod test_char {
    use super::*;
    const CHAR: char = 'Q';
    const DATA: CellData = CellData::Char(CHAR);

    struct Grid(Cell);
    struct Cell;

    impl WritableGrid for Grid {
        type Cell = Cell;
        fn writable(&mut self, coords: Coords) -> Option<&mut Cell> {
            assert_eq!(coords, COORDS);
            Some(&mut self.0)
        }
        fn best_fit_for_region(&self, _: Region) -> Coords { unreachable!() }
        fn find_cell_to_extend(&self, _: Coords) -> Option<Coords> { unreachable!() }
    }

    impl WritableCell for Cell {
        fn write(&mut self, data: CellData, styles: UseStyles) {
            assert_eq!(data, DATA);
            assert_eq!(styles, STYLES);
        }
        fn extend(&mut self, _: char, _: UseStyles) { unreachable!() }
        fn is_extendable(&self) -> bool { unreachable!() }
        fn is_extension_of(&self) -> Option<Coords> { unreachable!() }
    }

    #[test]
    fn char_write() {
        assert_eq!(CHAR.write(COORDS, STYLES, &mut Grid(Cell)), COORDS);
    }
}

mod test_wide_char {
    use super::*;
    const CHAR: char = 'R';
    const WIDTH: u32 = 2;
    const WIDE_CHAR: WideChar = WideChar(CHAR, WIDTH);
    const REGION: Region = Region {
        left: COORDS.x,
        top: COORDS.y,
        right: COORDS.x + WIDTH,
        bottom: COORDS.y + 1
    };
    const BEST_FIT_COORDS: Coords = Coords { x: 1, y: 1 };
    const FINAL_COORDS: Coords = Coords { x: BEST_FIT_COORDS.x + 1, ..BEST_FIT_COORDS };

    struct Grid(Cell, Cell);
    enum Cell { Char, Extension }

    impl WritableGrid for Grid {
        type Cell = Cell;
        fn writable(&mut self, coords: Coords) -> Option<&mut Cell> {
            match (coords.x, coords.y) {
                (1, 1)  => Some(&mut self.0),
                (2, 1)  => Some(&mut self.1),
                _       => panic!("Passed incorrect coords to write_to: {:?}", coords),
            }
        }
        fn best_fit_for_region(&self, region: Region) -> Coords {
            assert_eq!(region, REGION);
            BEST_FIT_COORDS
        }
        fn find_cell_to_extend(&self, _: Coords) -> Option<Coords> { unreachable!() }
    }

    impl WritableCell for Cell {
        fn write(&mut self, data: CellData, styles: UseStyles) {
            match *self {
                Cell::Char      => assert_eq!(data, CellData::Char(CHAR)),
                Cell::Extension => assert_eq!(data, CellData::Extension(BEST_FIT_COORDS)),
            }
            assert_eq!(styles, STYLES);
        }
        fn extend(&mut self, _: char, _: UseStyles) { unreachable!() }
        fn is_extendable(&self) -> bool { unreachable!() }
        fn is_extension_of(&self) -> Option<Coords> { unreachable!() }
    }

    #[test]
    fn wide_char_write() {
        assert_eq!(WIDE_CHAR.write(COORDS, STYLES, &mut Grid(Cell::Char, Cell::Extension)), FINAL_COORDS);
    }
}

mod test_char_extender {
    pub use super::*;

    pub const CHAR: char = '$';
    pub const DATA: CellData = CellData::Char(CHAR);
    pub const CHAR_EXTENDER: CharExtender = CharExtender(CHAR);
    pub const COORDS_BEFORE: Coords = Coords { x: 0, y: 0 };

    mod extendable_cell {
        use super::*;

        const FINAL_COORDS: Coords = COORDS_BEFORE;

        struct Grid(Cell);
        struct Cell;

        impl WritableGrid for Grid {
            type Cell = Cell;
            fn writable(&mut self, coords: Coords) -> Option<&mut Cell> {
                assert_eq!(coords, COORDS_BEFORE);
                Some(&mut self.0)
            }
            fn best_fit_for_region(&self, _: Region) -> Coords { unreachable!() }
            fn find_cell_to_extend(&self, coords: Coords) -> Option<Coords> {
                assert_eq!(coords, COORDS);
                Some(COORDS_BEFORE)
            }
        }

        impl WritableCell for Cell {
            fn write(&mut self, _: CellData, _: UseStyles) { unreachable!() }
            fn extend(&mut self, c: char, styles: UseStyles) {
                assert_eq!(c, CHAR);
                assert_eq!(styles, STYLES);
            }
            fn is_extendable(&self) -> bool { unreachable!() }
            fn is_extension_of(&self) -> Option<Coords> { unreachable!() }
            
        }

        #[test]
        fn char_extender_write() {
            assert_eq!(CHAR_EXTENDER.write(COORDS, STYLES, &mut Grid(Cell)), FINAL_COORDS);
        }
    }

    mod non_extendable_cell {
        use super::*;

        const FINAL_COORDS: Coords = COORDS;

        struct Grid(Cell);
        struct Cell;

        impl WritableGrid for Grid {
            type Cell = Cell;
            fn writable(&mut self, coords: Coords) -> Option<&mut Cell> {
                assert_eq!(coords, COORDS);
                Some(&mut self.0)
            }
            fn best_fit_for_region(&self, _: Region) -> Coords { unreachable!() }
            fn find_cell_to_extend(&self, coords: Coords) -> Option<Coords> {
                assert_eq!(coords, COORDS);
                None
            }
        }

        impl WritableCell for Cell {
            fn write(&mut self, data: CellData, styles: UseStyles) {
                assert_eq!(data, DATA);
                assert_eq!(styles, STYLES);
            }
            fn extend(&mut self, _: char, _: UseStyles) { unreachable!() }
            fn is_extendable(&self) -> bool { unreachable!() }
            fn is_extension_of(&self) -> Option<Coords> { unreachable!() }
        }

        #[test]
        fn char_extender_write() {
            assert_eq!(CHAR_EXTENDER.write(COORDS, STYLES, &mut Grid(Cell)), FINAL_COORDS);
        }
    }
}

mod test_image {
    use super::*;

    use std::str::FromStr;
    use mime::Mime;
    use datatypes::MediaPosition;

    // TODO what about when image is wider than grid is allowed to be?
    const FINAL_COORDS: Coords = Coords { x: COORDS.x + WIDTH, ..COORDS };
    const DATA: &'static [u8] = &[0x0B, 0xEE, 0xFD, 0xAD];
    const MIME: &'static str = "image/jpeg";
    const MEDIA_POSITION: MediaPosition = MediaPosition::Fill;
    const WIDTH: u32 = 5;
    const HEIGHT: u32 = 6;
    const REGION: Region = Region {
        left: COORDS.x,
        top: COORDS.y,
        right: COORDS.x + WIDTH,
        bottom: COORDS.y + HEIGHT,
    };
    const BEST_FIT_COORDS: Coords = Coords { x: 0, y: 0 };

    struct Grid(Cell, Cell);
    enum Cell { Image, Extension }

    impl WritableGrid for Grid {
        type Cell = Cell;
        fn writable(&mut self, coords: Coords) -> Option<&mut Cell> {
            match (coords.x, coords.y) {
                (0, 0)                              => Some(&mut self.0),
                (x, y) if x < WIDTH && y < HEIGHT   => Some(&mut self.1),
                _ => panic!("Passed incorrect coords to write_to: {:?}", coords),
            }
        }
        fn best_fit_for_region(&self, region: Region) -> Coords {
            assert_eq!(region, REGION);
            BEST_FIT_COORDS
        }
        fn find_cell_to_extend(&self, coords: Coords) -> Option<Coords> { unreachable!() }
    }

    impl WritableCell for Cell {
        fn write(&mut self, data: CellData, styles: UseStyles) {
            match *self {
                Cell::Image     => {
                    if let CellData::Image { data, mime, pos, width, height } = data {
                        assert_eq!(DATA, &*data.data);
                        assert_eq!(MIME, &mime.to_string());
                        assert_eq!(pos, MEDIA_POSITION);
                        assert_eq!(width, WIDTH);
                        assert_eq!(height, HEIGHT);
                    } else { panic!("instead of image, recieved: {:?}", data) }
                }
                Cell::Extension => assert_eq!(data, CellData::Extension(BEST_FIT_COORDS)),
            }
            assert_eq!(styles, STYLES);
        }
        fn extend(&mut self, _: char, _: UseStyles) { unreachable!() }
        fn is_extendable(&self) -> bool { unreachable!() }
        fn is_extension_of(&self) -> Option<Coords> { unreachable!() }
    }

    #[test]
    fn image_write() {
        let data = Vec::from(DATA);
        let mime = Mime::from_str(MIME).unwrap();
        let image = Image::new(data, mime, MEDIA_POSITION, WIDTH, HEIGHT);
        assert_eq!(image.write(COORDS, STYLES, &mut Grid(Cell::Image, Cell::Extension)), FINAL_COORDS);
    }
}

mod test_writable_grid { 
    use super::*;
    use terminal::char_grid::grid::Grid;

    #[derive(PartialEq, Eq)]
    enum Cell { Extensible, Extension, Empty }
    impl Default for Cell { fn default() -> Cell { Cell::Empty } }

    impl WritableCell for Cell {
        fn write(&mut self, _: CellData, _: UseStyles) { unimplemented!() }
        fn extend(&mut self, _: char, _: UseStyles) { unimplemented!() }
        fn is_extendable(&self) -> bool {
            *self == Cell::Extensible
        }
        fn is_extension_of(&self) -> Option<Coords> {
            if let Cell::Extension = *self {
                Some(Coords { x: 0, y: 0 })
            } else { None }
        }
    }

    fn grid() -> Grid<Cell> {
        Grid::with_x_y_caps(4, 4)
    }

    #[test]
    fn writable() {
        let mut grid = Grid::<Cell>::with_x_y_caps(4, 4);
        assert!(grid.writable(Coords { x: 1, y: 1 }).is_some());
        assert!(grid.width >= 2);
        assert!(grid.height >= 2);
    }

    #[test]
    fn best_fit_for_region_with_caps() {
        let grid = Grid::<Cell>::with_x_y_caps(4, 4);
        let within_bounds = Region::new(1, 1, 3, 3);
        let outside_bounds = Region::new(1, 1, 5, 5);
        assert_eq!(grid.best_fit_for_region(within_bounds),
                   Coords { x: within_bounds.left, y: within_bounds.top });
        assert_eq!(grid.best_fit_for_region(outside_bounds),
                   Coords { x: 0, y: 0 });
    }

    #[test]
    fn best_fit_for_region_without_caps() {
        let grid = Grid::<Cell>::with_infinite_scroll();
        let region = Region::new(1, 1, 3, 3);
        assert_eq!(grid.best_fit_for_region(region), Coords { x: region.left, y: region.top });
    }

    #[test]
    fn test_coords_before() {
        assert_eq!(coords_before(Coords { x: 0, y: 0 }, 4), Coords { x: 0, y: 0 });
        assert_eq!(coords_before(Coords { x: 0, y: 1 }, 4), Coords { x: 3, y: 0 });
        assert_eq!(coords_before(Coords { x: 2, y: 2 }, 4), Coords { x: 1, y: 2 });
    }

    #[test]
    fn test_cell_to_extend() {
        let mut grid = Grid::with_infinite_scroll();
        *grid.writable(Coords { x: 0, y: 0 }).unwrap() = Cell::Extensible;
        *grid.writable(Coords { x: 1, y: 0 }).unwrap() = Cell::Extension;
        *grid.writable(Coords { x: 2, y: 0 }).unwrap() = Cell::Empty;
        assert_eq!(cell_to_extend(&grid, Coords { x: 0, y: 0 }), Some(Coords { x: 0, y: 0 }));
        assert_eq!(cell_to_extend(&grid, Coords { x: 1, y: 0 }), Some(Coords { x: 0, y: 0 }));
        assert_eq!(cell_to_extend(&grid, Coords { x: 2, y: 0 }), None);
        assert_eq!(cell_to_extend(&grid, Coords { x: 3, y: 0 }), None);
    }
}

mod test_writable_cell {
    use super::*;
    use terminal::{Styles, DEFAULT_STYLES};

    fn cell(data: CellData) -> CharCell {
        CharCell {
            styles: DEFAULT_STYLES,
            content: data,
        }
    }

    #[test]
    fn write() {
        let data = CellData::Char('6');
        let styles = UseStyles::Custom(Styles::default());
        let mut cell = cell(CellData::Empty);
        cell.write(data.clone(), styles);
        assert_eq!(cell.content, data);
        assert_eq!(cell.styles, styles);
    }

    #[test]
    fn extend_char() {
        let styles = UseStyles::Custom(Styles::default());
        let mut cell = cell(CellData::Char('E'));
        cell.extend('!', styles);
        assert_eq!(cell.content, CellData::Grapheme(String::from("E!")));
        assert_eq!(cell.styles, styles);
    }

    #[test]
    fn extend_grapheme() {
        let styles = UseStyles::Custom(Styles::default());
        let mut cell = cell(CellData::Grapheme(String::new()));
        cell.extend('!', styles);
        assert_eq!(cell.content, CellData::Grapheme(String::from("!")));
        assert_eq!(cell.styles, styles);
    }

    #[test]
    fn is_extendable() {
        assert!(cell(CellData::Char('\0')).is_extendable());
        assert!(cell(CellData::Grapheme(String::new())).is_extendable());
        assert!(!cell(CellData::Empty).is_extendable());
        assert!(!cell(CellData::Extension(Coords { x: 0, y: 0 })).is_extendable());
    }

    #[test]
    fn is_extension_of() {
        let coords = Coords { x: 7, y: 5 };
        assert_eq!(cell(CellData::Extension(coords)).is_extension_of(), Some(coords));
        assert_eq!(cell(CellData::Empty).is_extension_of(), None);
        assert_eq!(cell(CellData::Char('z')).is_extension_of(), None);
    }
}
