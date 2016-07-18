use datatypes::{Coords, Region};

use self::View::*;

pub struct Window {
    point: Coords,
    width: u32,
    height: u32,
    view: View,
}

pub enum View {
    Moveable,
    Reflowable,
}

impl Window {
    pub fn new(point: Coords, width: u32, height: u32, view: View) -> Window {
        Window {
            point: point,
            width: width,
            height: height,
            view: view,
        }
    }

    pub fn translate(&self, Coords { x, y }: Coords) -> Coords {
        match self.view {
            Moveable    => {
                let coords = Coords { x: x + self.point.x, y: y + self.point.y };
                assert!(self.bounds().contains(coords));
                coords
            }
            Reflowable  => unimplemented!()
        }
    }

    pub fn bounds(&self) -> Region {
        match self.view {
            Moveable    => Region {
                left: self.point.x,
                top: self.point.y,
                right: self.point.x + self.width,
                bottom: self.point.y + self.height,
            },
            Reflowable  => unimplemented!()
        }
    }

    pub fn resize_width(&mut self, width: u32) {
        self.width = width;
    }

    pub fn resize_height(&mut self, height: u32) {
        self.height = height;
    }
}
