//  notty is a new kind of terminal emulator.
//  Copyright (C) 2016 Wayne Warren
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
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::atomic::Ordering::Relaxed;
use std::{error, fmt, io, mem, result};

use super::Config;

use notty::cfg::{SCROLLBACK, TAB_STOP};
use notty_cairo::{ColorConfig, TrueColor};

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Parse(String), // TODO: once https://github.com/alexcrichton/toml-rs/issue#69
                   // is closed, change this to Parse(toml::ParserError)
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Io(ref err) => write!(f, "IO Error: {}", err),
            ConfigError::Parse(ref string) => write!(f, "{}", string),
        }
    }
}

impl error::Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::Io(ref err) => err.description(),
            ConfigError::Parse(ref string) => &string,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ConfigError::Io(ref err) => err.cause(),
            ConfigError::Parse(_) => None,
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError{
        ConfigError::Io(err)
    }
}

pub type Result<T> = result::Result<T, ConfigError>;

fn update_general(font: &mut String, table: &toml::Table) {
    for (k, v) in table.iter() {
        match &k[..] {
            "font" => *font = v.as_str().
                map(|s| s.to_string()).
                unwrap(),
            "tabstop" => TAB_STOP.store(v.as_integer().unwrap() as usize, Relaxed),
            "scrollback" => SCROLLBACK.store(v.as_integer().unwrap() as isize, Relaxed),
            _ => {},
        };
    }
}

fn update_colors(config: &mut ColorConfig, table: &toml::Table) {
    for (k, v) in table.iter() {
        match &k[..] {
            "fg" => config.bg_color = convert_tomlv_to_color(v),
            "bg" => config.fg_color = convert_tomlv_to_color(v),
            "cursor" => config.cursor_color = convert_tomlv_to_color(v),
            "palette" => config.palette = convert_tomlv_to_palette(v),
            _ => {},
        };
    }
}

/// Update &config from toml file identified by path string.
pub fn update_from_file<P: AsRef<Path>>(config: &mut Config, path: P) -> Result<()> {
    let table = try!(read_toml_file(path));

    for (k, v) in table.iter() {
        match &k[..] {
            "colors" => update_colors(&mut config.color_cfg, v.as_table().unwrap()),
            "general" => update_general(&mut config.font, v.as_table().unwrap()),
            _ => {},
        };
    }
    Ok(())
}

fn convert_tomlv_to_color(value: &toml::Value) -> TrueColor {
    let slice = value.as_slice().unwrap();
    (
        slice[0].as_integer().unwrap() as u8,
        slice[1].as_integer().unwrap() as u8,
        slice[2].as_integer().unwrap() as u8
    )
}

fn convert_tomlv_to_palette(value: &toml::Value) -> [TrueColor; 256] {
    let v = value.as_slice().unwrap()
                 .into_iter().map(convert_tomlv_to_color)
                 .collect::<Vec<_>>();
    assert_eq!(v.len(), 256);
    let palette: &[TrueColor; 256] = unsafe { mem::transmute(v.as_ptr()) };
    *palette
}

fn read_toml_file<P: AsRef<Path>>(path: P) -> Result<toml::Table> {
    let mut file = try!(File::open(&path));
    let mut source = String::new();
    try!(file.read_to_string(&mut source));
    parse_toml(&source.to_string(), path)
}

fn parse_toml<P: AsRef<Path>>(toml_string: &String, toml_path: P)
                              -> Result<toml::Table> {
    let mut parser = toml::Parser::new(toml_string);
    match parser.parse() {
        Some(toml) => {
            Ok(toml)
        }
        None => {
            let mut error_string = String::new();
            for err in &parser.errors {
                let (loline, locol) = parser.to_linecol(err.lo);
                let (hiline, hicol) = parser.to_linecol(err.hi);
                error_string = format!("{}\n{}:{}:{}:{}:{} error: {}",
                        error_string, toml_path.as_ref().display(), loline,
                        locol, hiline, hicol, err.desc);
            }
            Err(ConfigError::Parse(error_string))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate toml;

    use super::*;

    use super::super::Config;

    fn test_default_config(config: &Config) {
        assert_eq!(config.font, "Inconsolata 10");
        assert_eq!(config.color_cfg.fg_color, (0xff,0xff,0xff));
        assert_eq!(config.color_cfg.bg_color, (0x00,0x00,0x00));
        assert_eq!(config.color_cfg.cursor_color, (0xbb,0xbb,0xbb));
        assert_eq!(config.color_cfg.palette[0], (0x00,0x00,0x00));
        assert_eq!(config.color_cfg.palette[5], (0xff,0x55,0xff));
    }

    #[test]
    fn test_default() {
        let config = Config::default();
        test_default_config(&config);
    }

    #[test]
    fn test_update_from_file() {
        let mut config = Config::default();
        test_default_config(&config);

        let update_path = "resources/update-config.toml".to_string();
        update_from_file(&mut config, &update_path).unwrap();
        assert_eq!(config.font, "Liberation Mono 8");
    }
}
