extern crate byteorder;

use std::rc::Rc;
use std::cell::RefCell;
use std::io::Cursor;
use self::byteorder::{LittleEndian, ReadBytesExt};
use ::script::variable::{Variable, VariableType};

pub enum ArgType {
	None,
	Integer(u32),
	Float(f32),
	String(String),
	Var(Rc<RefCell<Variable>>),
}

pub trait Parser {
    fn parse_int(&mut self) -> Option<u32>;
    fn parse_float(&mut self) -> Option<f32>;
    fn parse_string(&mut self) -> Option<String>;
    fn parse_var(&mut self) -> Option<Rc<RefCell<Variable>>>;
	fn parse_any_arg(&mut self) -> Option<ArgType>;
}

impl Parser for super::Script {
    fn parse_int(&mut self) -> Option<u32> {
        let offset = self.offset;
		/*
			0x01 - 32 bits
			0x04 - 8 bits
			0x05 - 16 bits
		*/

		if offset >= self.bytes.len() {
			return None;
		}

		match self.bytes[offset] {
			0x01 => {
				self.offset += 5;
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 5]);
				buffer.read_u32::<LittleEndian>().ok()
			},
			0x04 => {
				self.offset += 2;
				Some(self.bytes[offset + 1] as u32)
			}, 
			0x05 => {
				self.offset += 3;
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 3]);
				Some(buffer.read_u16::<LittleEndian>().unwrap() as u32)
			},
			_ => None,
		}
    }

    fn parse_float(&mut self) -> Option<f32> {
        let offset = self.offset;

		if offset + 4 < self.bytes.len() && self.bytes[offset] == 0x06 {
			self.offset += 5;
			let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 5]);
			buffer.read_f32::<LittleEndian>().ok()
		} else {
			None
		}
    }

    fn parse_string(&mut self) -> Option<String> {
        let offset = self.offset;
		
		if self.bytes[offset] == 0x0E && offset + 1 < self.bytes.len() {
			let length = self.bytes[offset + 1] as usize;

			if length + offset <= self.bytes.len() {
				self.offset += 2 + length;
				String::from_utf8(Vec::from(&self.bytes[offset + 2 .. offset + 2 + length])).ok()
			} else {
				None
			}
		} else {
			None
		}
    }

    fn parse_var(&mut self) -> Option<Rc<RefCell<Variable>>> {
		/*
			0x02 - Variable::Global
			0x03 - Variable::Local
		*/

		let offset = self.offset;

		let var = match self.bytes[offset] {
			0x02 => {
				self.offset += 3;
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 3]);
				Some(VariableType::Global(buffer.read_u16::<LittleEndian>().unwrap() as usize))
			},
			0x03 => {
				self.offset += 3;
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 3]);
				Some(VariableType::Local(buffer.read_u16::<LittleEndian>().unwrap() as usize))
			},
			_ => None,
		};

		if let Some(variable) = var {
			Some(self.get_variable(variable))
		} else {
			None
		}
    }

	fn parse_any_arg(&mut self) -> Option<ArgType> {
		match self.bytes[self.offset] {
			0x01 | 0x04 | 0x05 => self.parse_int().and_then(|val| Some(ArgType::Integer(val))),
			0x02 | 0x03 => self.parse_var().and_then(|val| Some(ArgType::Var(val))),
			0x06 => self.parse_float().and_then(|val| Some(ArgType::Float(val))),
			0x0E => self.parse_string().and_then(|val| Some(ArgType::String(val))),
			_ => Some(ArgType::None),
		}
	}
}

impl ::std::fmt::Display for ArgType {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
			ArgType::Integer(val) => write!(f, "{}", val),
			ArgType::Float(val) => write!(f, "{}", val),
			ArgType::String(ref val) => write!(f, "{}", val),
			ArgType::Var(ref val) => write!(f, "{}", *val.borrow()),
			ArgType::None => write!(f, "None value"),
		}
	}
}