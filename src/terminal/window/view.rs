use datatypes::{Coords, Region};

use self::Flow::*;

#[derive(Eq, PartialEq, Debug)]
pub struct View {
    point: Coords,
    width: u32,
    height: u32,
    flow: Flow,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Flow {
    Moveable,
    Reflowable,
}

impl View {
    pub fn new(point: Coords, width: u32, height: u32, flow: Flow) -> View {
        View {
            point: point,
            width: width,
            height: height,
            flow: flow,
        }
    }

    pub fn translate(&self, Coords { x, y }: Coords) -> Coords {
        match self.flow {
            Moveable    => {
                let coords = Coords { x: x + self.point.x, y: y + self.point.y };
                assert!(self.bounds().contains(coords));
                coords
            }
            Reflowable  => unimplemented!()
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bounds(&self) -> Region {
        match self.flow {
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

    pub fn keep_cursor_within(&mut self, coords: Coords) {
        match self.flow {
            Moveable    => {
                let new_bounds = self.bounds().move_to_contain(coords);
                if new_bounds != self.bounds() {
                    self.point = Coords { x: new_bounds.left, y: new_bounds.top };
                }
            }
            Reflowable  => unimplemented!()
        }
    }
}
