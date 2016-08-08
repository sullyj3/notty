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
use std::sync::Arc;

use mime::Mime;

use datatypes::{Coords, MediaPosition};
use terminal::{UseStyles, Styles, DEFAULT_STYLES};

use self::CellData::*;

pub const EMPTY_CELL: CharCell = CharCell {
    styles: DEFAULT_STYLES,
    content: CellData::Empty,
};

#[derive(Clone, PartialEq, Debug)]
pub struct CharCell {
    pub styles: UseStyles,
    pub content: CellData,
}

impl CharCell {
    pub fn is_extension(&self) -> bool {
        if let CellData::Extension(_) = self.content { true } else { false }
    }

    pub fn repr(&self) -> String {
        match self.content {
            Char(c)         => c.to_string(),
            Grapheme(ref s) => s.clone(),
            Image { .. }    => String::from("IMG"),
            Empty           => String::new(),
            Extension(_)    => String::from("EXT"),
        }
    }

}

impl Default for CharCell {
    fn default() -> CharCell {
        CharCell {
            content: Empty,
            styles: UseStyles::Custom(Styles::new()),
        }
    }
}

impl ToString for CharCell {
    fn to_string(&self) -> String {
        match self.content {
            Char(c)         => c.to_string(),
            Grapheme(ref s) => s.clone(),
            _               => String::new()
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum CellData {
    Empty,
    Char(char),
    Grapheme(String),
    Extension(Coords),
    Image { 
        data: Arc<ImageData>,
        mime: Mime,
        pos: MediaPosition,
        width: u32,
        height: u32,
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ImageData {
    pub data: Vec<u8>,
    pub coords: Coords,
}
