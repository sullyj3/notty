use std::cell::RefCell;
use std::sync::Arc;

use mime::Mime;

use datatypes::{Coords, CoordsIter, MediaPosition};
use terminal::{CharData, CellData, ImageData, UseStyles};
use super::{WritableGrid, WritableCell, region_at};


pub struct Image {
    data: RefCell<Option<(Vec<u8>, Mime)>>,
    pos: MediaPosition,
    width: u32,
    height: u32,
}

impl Image {
    pub fn new(data: Vec<u8>, mime: Mime, pos: MediaPosition, w: u32, h: u32) -> Image {
        Image {
            data: RefCell::new(Some((data, mime))),
            pos: pos,
            width: w,
            height: h,
        }
    }
}

impl CharData for Image {
    fn write<T: WritableGrid>(&self, coords: Coords, styles: UseStyles, grid: &mut T) -> Coords {
        if let Some((data, mime)) = self.data.borrow_mut().take() {
            let coords = grid.best_fit_for_region(region_at(coords, self.width, self.height));
            if let Some(cell) = grid.writable(coords) {
                let image = CellData::Image {
                    data: Arc::new(ImageData {
                        data: data,
                        coords: coords,
                    }),
                    mime: mime,
                    pos: self.pos,
                    width: self.width,
                    height: self.height,
                };
                cell.write(image, styles);
            }
            let iter = CoordsIter::from(region_at(coords, self.width, self.height));
            for extension_coords in iter.skip(1) {
                if let Some(cell) = grid.writable(extension_coords) {
                    cell.write(CellData::Extension(coords), styles);
                }
            }
            unimplemented!() // return proper coords
        } else { coords }
    }
}
