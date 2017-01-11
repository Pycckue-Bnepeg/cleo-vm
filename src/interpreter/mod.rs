#![allow(dead_code)]

extern crate byteorder;
extern crate time;

use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::io::Cursor;
use self::byteorder::{LittleEndian, ReadBytesExt};
use self::time::Duration;
use std::time::Instant;
use std::mem::transmute;
use std::fmt;

pub mod default_opcodes;
pub mod error;

type OpcodeHandler = Box<Fn(&ScriptThread) -> Result<bool, error::OpcodeHandlerErr>>;

pub enum Variable {
	Global(usize),
	Local(usize),
}

pub enum ArgType {
	None,
	Integer(i32),
	Float(f32),
	Variable(Variable),
	String(String),
}

pub enum LogicalOpcode {
	One,
	And,
	Or,
}

pub struct VirtualMachine {
	global_vars: Rc<RefCell<Vec<i32>>>,
	scripts: Vec<ScriptThread>,
	handlers: HashMap<u16, OpcodeHandler>,
}

impl VirtualMachine {
	/// Create new Virtual Machine
	/// # Examples
	/// ```
	/// use interpreter::VirtualMachine;
	/// 
	/// let cleo_vm = VirtualMachine::new();
	/// ```
	pub fn new() -> VirtualMachine {
		VirtualMachine {
			global_vars: Rc::new(RefCell::new(vec![0; 0x10000])),
			scripts: Vec::new(),
			handlers: HashMap::new(),
		}
	}

	/// Set handler for an opcode.
	/// # Example
	/// ```
	/// vm.set_opcode_handler(0x00FF, Box::new(|thread| {
	///		unimplemented!();
	/// }));
	/// ```
	pub fn set_opcode_handler(&mut self, opcode: u16, handler: OpcodeHandler) {
		self.handlers.insert(opcode, handler);
	}

	pub fn append_script(&mut self, bytecode: Vec<u8>) {
		self.scripts.push(ScriptThread::new(bytecode, self.global_vars.clone()));
	}

	pub fn run(&self) {
		loop {
			for script in &self.scripts {
				if !script.is_active() {
					continue;
				}

				match script.get_opcode() {
					Some(opcode) => {
						let handler = self.handlers.get(&opcode).expect("Opcode doesn't exist!");
						
						match handler(&script) {
							Ok(result) => script.set_conditional_result(result),
							Err(err) => println!("{}", err),
						}
					},
					None => continue,
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
	instant: RefCell<Instant>,
	not_flag: Cell<bool>,
}

impl ScriptThread {
	/// Create new script thread (use in VirtualMachine) passing to opcode handler.
	/// 
	/// `bytes` - CLEO script bytecode
	/// `vars` - Global variables of Virtual Machine
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
			instant: RefCell::new(Instant::now()),
			not_flag: Cell::new(false),
		}
	}

	/// Get current opcode of script
	///
	/// # Example 
	/// ```
	/// match script.get_opcode() {
	///		Some(opcode) => handle_opcode(opcode, script),
	///		None => println!("Current script is done"),
	/// }
	/// ```
	pub fn get_opcode(&self) -> Option<u16> {
		let offset = self.offset.get();

		if offset + 1 >= self.bytes.len() {
			return None;
		}
		
		let opcode: u16 = ((self.bytes[offset + 1] as u16) << 8) + (self.bytes[offset] as u16);
		self.offset.set(offset + 2);

		if opcode & 0x8000 != 0 {
			self.not_flag.set(true);
			Some(opcode ^ 0x8000)
		} else {
			self.not_flag.set(false);
			Some(opcode)
		}	
	}

	/// Set conditional result of current script. Used in `VirtualMachine` and by opcode 00D6. 
	pub fn set_conditional_result(&self, result: bool) {
		let conditional_result = self.condition_result.get();
		let not_flag = self.not_flag.get();

		match *self.logical_opcode.borrow() {
			LogicalOpcode::One => self.condition_result.set(result ^ not_flag),
			LogicalOpcode::And => self.condition_result.set(conditional_result & (result ^ not_flag)),
			LogicalOpcode::Or => self.condition_result.set(conditional_result | (result ^ not_flag)),
		}
	}

	pub fn set_logical_opcode(&self, opcode: LogicalOpcode) -> bool {
		self.condition_result.set(true);
		*self.logical_opcode.borrow_mut() = opcode;
		true
	}

	/// Parse an integer value in arguments of an opcode.
	/// # Example
	/// ```
	/// let vm = VirtualMachine::new();
	/// vm.set_opcode_handler(0x0AB1, Box::new(|thread| {
	/// 	match thread.parse_int() {
	///			Some(value) => do_something(),
	///			None => false,
	/// 	}
	/// }));
	/// ```
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
				buffer.read_i32::<LittleEndian>().ok()
			},
			0x04 => {
				self.offset.set(offset + 2);
				Some(self.bytes[offset + 1] as i32)
			}, 
			0x05 => {
				self.offset.set(offset + 3);
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 3]);
				Some(buffer.read_i16::<LittleEndian>().unwrap() as i32)
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
			buffer.read_f32::<LittleEndian>().ok()
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
				Some(Variable::Global(buffer.read_u16::<LittleEndian>().unwrap() as usize))
			},
			0x03 => {
				self.offset.set(offset + 3);
				let mut buffer = Cursor::new(&self.bytes[offset + 1 .. offset + 3]);
				Some(Variable::Local(buffer.read_u16::<LittleEndian>().unwrap() as usize))
			},
			_ => None,
		}
	}

	pub fn get_any_arg(&self) -> Option<ArgType> {
		let offset = self.offset.get();

		match self.bytes[offset] {
			0x01 | 0x04 | 0x05 => self.parse_int().and_then(|val| Some(ArgType::Integer(val))),
			0x02 | 0x03 => self.parse_variable().and_then(|val| Some(ArgType::Variable(val))),
			0x06 => self.parse_float().and_then(|val| Some(ArgType::Float(val))),
			0x0E => self.parse_string().and_then(|val| Some(ArgType::String(val))),
			_ => Some(ArgType::None),
		}
	}

	pub fn get_variable(&self, var: &Variable) -> i32 {
		match *var {
			Variable::Global(id) => {
				let global_vars_ptr = self.global_vars.borrow();
				let global_vars = global_vars_ptr.borrow();
				global_vars[id]
			},
			Variable::Local(id) => self.local_vars.borrow()[id],
		}
	}

	pub fn set_variable(&self, var: &Variable, value: i32) {
		match *var {
			Variable::Global(id) => {
				let global_vars_ptr = self.global_vars.borrow();
				let mut global_vars = (*global_vars_ptr).borrow_mut();
				global_vars[id] = value;
			},
			Variable::Local(id) => self.local_vars.borrow_mut()[id] = value,
		}
	}

	pub fn set_variable_float(&self, var: &Variable, value: f32) {
		let integer: u32 = unsafe {
			transmute(value)
		};
		
		self.set_variable(&var, integer as i32);
	}

	pub fn get_variable_float(&self, var: &Variable) -> f32 {
		unsafe {
			transmute(self.get_variable(&var))
		}
	}

	pub fn set_wake_up(&self, time: u32) {
		self.wake_up.set(time);
		*self.instant.borrow_mut() = Instant::now();
	}

	pub fn is_active(&self) -> bool {
		let active = self.active.get();

		if active {
			match Duration::from_std(self.instant.borrow().elapsed()) {
				Ok(duration) => duration.num_milliseconds() >= self.wake_up.get() as i64,
				Err(_) => false,
			}
		} else {
			false
		}
	}

	pub fn jump_to(&self, address: usize) {
		self.offset.set(0xFFFFFFFF - address + 1);
	} 
}

impl fmt::Display for ArgType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ArgType::Integer(val) => write!(f, "{}", val),
			ArgType::Float(val) => write!(f, "{}", val),
			ArgType::String(ref val) => write!(f, "{}", val),
			ArgType::Variable(ref val) => {
				match *val {
					Variable::Global(id) => write!(f, "${}", id),
					Variable::Local(id) => write!(f, "{}@", id),
				}
			},
			ArgType::None => write!(f, "None value"),
		}
	}
}