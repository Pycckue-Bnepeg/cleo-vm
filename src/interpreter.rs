#![allow(dead_code)]

extern crate byteorder;

use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::io::Cursor;
use self::byteorder::{BigEndian, ReadBytesExt};

pub enum Variable {
	Global(usize),
	Local(usize),
}

pub enum ArgType {
	None,
	Integer,
	Float,
	Variable,
	String,
}

pub enum LogicalOpcode {
	One,
	And,
	Or,
}

pub struct VirtualMachine {
	global_vars: Rc<RefCell<Vec<i32>>>,
	scripts: Vec<ScriptThread>,
	handlers: HashMap<u16, Box<Fn(&ScriptThread) -> bool>>,
}

impl VirtualMachine {
	pub fn new() -> VirtualMachine {
		VirtualMachine {
			global_vars: Rc::new(RefCell::new(vec![0; 0x10000])),
			scripts: Vec::new(),
			handlers: HashMap::new(),
		}
	}

	pub fn set_opcode_handler(&mut self, opcode: u16, handler: Box<Fn(&ScriptThread) -> bool>) {
		self.handlers.insert(opcode, handler);
	}

	pub fn append_script(&mut self, bytecode: Vec<u8>) {
		self.scripts.push(ScriptThread::new(bytecode, self.global_vars.clone()));
	}

	pub fn run(&self) {
		'test: loop {
			for script in &self.scripts {
				if !script.is_active() {
					continue;
				}

				match script.get_opcode() {
					Some(opcode) => {
						let handler = self.handlers.get(&opcode).expect("Opcode doesn't exist!");
						let result = handler(&script);

						script.set_conditional_result(result);
					},
					None => break 'test,
				}
			}
		}
	}
}

pub struct ScriptThread {
	bytes: Vec<u8>,
	name: String,
	offset: Cell<usize>,
	local_vars: RefCell<Vec<i32>>,
	global_vars: RefCell<Rc<RefCell<Vec<i32>>>>,
	active: Cell<bool>,
	condition_result: Cell<bool>,
	logical_opcode: RefCell<LogicalOpcode>,
	stack: RefCell<Vec<u32>>,
	wake_up: Cell<u32>,
}

impl ScriptThread {
	pub fn new(bytes: Vec<u8>, vars: Rc<RefCell<Vec<i32>>>) -> ScriptThread {
		ScriptThread {
			bytes: bytes,
			name: String::new(),
			offset: Cell::new(0),
			local_vars: RefCell::new(vec![0; 34]),
			global_vars: RefCell::new(vars),
			active: Cell::new(true),
			condition_result: Cell::new(false),
			logical_opcode: RefCell::new(LogicalOpcode::One),
			stack: RefCell::new(Vec::new()),
			wake_up: Cell::new(0),
		}
	}

	pub fn get_opcode(&self) -> Option<u16> {
		let offset = self.offset.get();

		if offset + 1 >= self.bytes.len() {
			return None;
		}
		
		let opcode: u16 = ((self.bytes[offset + 1] as u16) << 8) + (self.bytes[offset] as u16);
		self.offset.set(offset + 2);

		Some(opcode)		
	}

	pub fn set_conditional_result(&self, result: bool) {
		let conditional_result = self.condition_result.get();

		match *self.logical_opcode.borrow() {
			LogicalOpcode::One => self.condition_result.set(result),
			LogicalOpcode::And => self.condition_result.set(conditional_result & result),
			LogicalOpcode::Or => self.condition_result.set(conditional_result | result),
		}
	}

	pub fn set_logical_opcode(&self, opcode: LogicalOpcode) {
		self.condition_result.set(true);
		*self.logical_opcode.borrow_mut() = opcode;
	}

	pub fn parse_int(&self) -> Option<i32> {
		let offset = self.offset.get();
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
				self.offset.set(offset + 5);
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 5]);
				buffer.read_i32::<BigEndian>().ok()
			},
			0x04 => {
				self.offset.set(offset + 2);
				Some(self.bytes[offset + 1] as i32)
			}, 
			0x05 => {
				self.offset.set(offset + 3);
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 3]);
				Some(buffer.read_i16::<BigEndian>().unwrap() as i32)
			},
			_ => None,
		}
	}

	pub fn parse_string(&self) -> Option<String> {
		let offset = self.offset.get();
		
		if self.bytes[offset] == 0x0E && offset + 1 < self.bytes.len() {
			let length = self.bytes[offset + 1] as usize;

			if length + offset <= self.bytes.len() {
				self.offset.set(offset + 2 + length);
				String::from_utf8(Vec::from(&self.bytes[offset + 2 .. offset + 2 + length])).ok()
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn parse_float(&self) -> Option<f32> {
		let offset = self.offset.get();

		if offset + 4 < self.bytes.len() && self.bytes[offset] == 0x06 {
			self.offset.set(offset + 5);
			let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 5]);
			buffer.read_f32::<BigEndian>().ok()
		} else {
			None
		}
	}

	pub fn parse_variable(&self) -> Option<Variable> {
		/*
			0x02 - Variable::Global
			0x03 - Variable::Local
		*/

		let offset = self.offset.get();

		match self.bytes[offset] {
			0x02 => {
				self.offset.set(offset + 3);
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 3]);
				Some(Variable::Global(buffer.read_u16::<BigEndian>().unwrap() as usize))
			},
			0x03 => {
				self.offset.set(offset + 3);
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 3]);
				Some(Variable::Local(buffer.read_u16::<BigEndian>().unwrap() as usize))
			},
			_ => None,
		}
	}

	pub fn get_arg_type(&self) -> ArgType {
		let offset = self.offset.get();

		match self.bytes[offset] {
			0x01 | 0x04 | 0x05 => ArgType::Integer,
			0x02 | 0x03 => ArgType::Variable,
			0x06 => ArgType::Float,
			0x0E => ArgType::String,
			_ => ArgType::None,
		}
	}

	pub fn get_variable(&self, var: Variable) -> i32 {
		match var {
			Variable::Global(id) => {
				let global_vars_ptr = self.global_vars.borrow();
				let global_vars = global_vars_ptr.borrow();
				global_vars[id]
			},
			Variable::Local(id) => self.local_vars.borrow()[id],
		}
	}

	pub fn set_variable(&self, var: Variable, value: i32) {
		match var {
			Variable::Global(id) => {
				let global_vars_ptr = self.global_vars.borrow();
				let mut global_vars = (*global_vars_ptr).borrow_mut();
				global_vars[id] = value;
			},
			Variable::Local(id) => self.local_vars.borrow_mut()[id] = value,
		}
	}

	pub fn is_active(&self) -> bool {
		// TODO: checking wake up timer
		self.active.get()
	}
}