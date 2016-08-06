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
use mime::Mime;

use std::cell::RefCell;

use command::prelude::*;
use datatypes::{CharData, Coords, MediaPosition};
use datatypes::Movement::Position;

pub struct Put(RefCell<Option<CharData>>);

impl Put {
    pub fn new_char(ch: char) -> Put {
        Put(RefCell::new(Some(CharData::Char(ch))))
    }
    pub fn new_extension(ch: char) -> Put {
        Put(RefCell::new(Some(CharData::ExtensionChar(ch))))
    }
    pub fn new_image(data: Vec<u8>, mime: Mime, pos: MediaPosition, w: u32, h: u32) -> Put {
        Put(RefCell::new(Some(CharData::Image {
            pos: pos,
            width: w,
            height: h,
            data: data,
            mime: mime,
        })))
    }
}

impl Command for Put {

    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(data) = self.0.borrow_mut().take() {
            terminal.write(data)
        }
        Ok(())
    }

    fn repr(&self) -> String {
        match *self.0.borrow() {
            Some(CharData::Char(c)) | Some(CharData::ExtensionChar(c))
                                            => c.to_string(),
            _                               => String::from("PUT"),
        }
    }

}

pub struct PutAt(RefCell<Option<CharData>>, Coords);

impl PutAt {

    pub fn new_image(data: Vec<u8>, mime: Mime, pos: MediaPosition, w: u32, h: u32, at: Coords)
            -> PutAt {
        PutAt(RefCell::new(Some(CharData::Image {
            pos: pos,
            width: w,
            height: h,
            data: data,
            mime: mime,
        })), at)
    }
}

impl Command for PutAt {

    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        if let Some(data) = self.0.borrow_mut().take() {
            let coords = terminal.cursor_position();
            terminal.move_cursor(Position(self.1));
            terminal.write(data);
            terminal.move_cursor(Position(coords));
        }
        Ok(())
    }

    fn repr(&self) -> String {
        String::from("PUT AT")
    }

}
