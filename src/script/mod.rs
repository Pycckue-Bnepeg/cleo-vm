extern crate time;

use std::cell::RefCell;
use std::rc::Rc;
use self::time::Duration;
use std::time::Instant;

pub mod parser;
pub mod error;
pub mod variable;

use self::parser::Parser;
use self::variable::*;

pub enum LogicalOpcode {
	One,
	And,
	Or,
}

pub struct Script {
    pub bytes: Vec<u8>,
	name: String,
	pub offset: usize,
	pub local_vars: Vec<Rc<RefCell<Variable>>>,
	global_vars: Rc<RefCell<Vec<u32>>>,
	active: bool,
    pub done: bool,
	pub cond_result: bool,
	logical_opcode: LogicalOpcode,
	pub stack: Vec<usize>,
	wake_up: u32,
    instant: Instant,
    not_flag: bool,
}

impl Script {
    pub fn new(name: &String, bytes: Vec<u8>, vars: Rc<RefCell<Vec<u32>>>) -> Script {
        let mut local_vars: Vec<Rc<RefCell<Variable>>> = Vec::new();

        for i in 0..32 {
            local_vars.push(Rc::new(RefCell::new(Variable::new(VariableKind::Integer, 0, i))));
        }

        Script {
            bytes: bytes,
            name: name.clone(),
            offset: 0,
            local_vars: local_vars,
            global_vars: vars,
            active: true,
            cond_result: false,
            logical_opcode: LogicalOpcode::One,
            stack: Vec::new(),
            wake_up: 0,
            instant: Instant::now(),
            not_flag: false,
            done: false,
        }
    }

    pub fn get_error(&self) -> (usize, &[u8]) {
        let length = if self.offset + 3 >= self.bytes.len() {
            self.bytes.len()
        } else {
            self.offset + 3
        };
        (self.offset, &self.bytes[self.offset - 2..length])
    }

    pub fn get_opcode(&mut self) -> Option<u16> {
        if self.offset + 1 >= self.bytes.len() {
            None
        } else {
            let opcode: u16 = ((self.bytes[self.offset + 1] as u16) << 8) + (self.bytes[self.offset] as u16);
            self.offset += 2;
            
            if opcode & 0x8000 != 0 {
                self.not_flag = true;
                Some(opcode ^ 0x8000)
            } else {
                self.not_flag = false;
                Some(opcode)
            }
        }
    }

    pub fn set_cond_result(&mut self, result: bool) {
        let new = result ^ self.not_flag;

        match self.logical_opcode {
			LogicalOpcode::One => self.cond_result = new,
			LogicalOpcode::And => self.cond_result &= new,
			LogicalOpcode::Or => self.cond_result |= new,
		}
    }

    pub fn set_logical_opcode(&mut self, result: LogicalOpcode) -> bool {
        self.logical_opcode = result;
        true
    }

    pub fn get_variable(&mut self, var: VariableType) -> Rc<RefCell<Variable>> {
        match var {
            VariableType::Local(id) => self.local_vars[id].clone(),
            VariableType::Global(id) => unimplemented!(),
        }
    }

    pub fn skip_args(&mut self, count: usize) -> bool {
        for _ in 0..count {
            match self.parse_any_arg() {
                Some(_) => continue,
                None => return false,
            }
        }

        true
    }

    pub fn jump_to(&mut self, address: usize) -> bool {
        self.offset = 0xFFFFFFFF - address + 1;
        true
    }

    pub fn set_wake_up(&mut self, time: u32) -> bool {
        self.wake_up = time;
        self.instant = Instant::now();
        true
    }

    pub fn is_active(&mut self) -> bool {
        if self.active {
			match Duration::from_std(self.instant.elapsed()) {
				Ok(duration) => duration.num_milliseconds() >= self.wake_up as i64,
				Err(_) => false,
			}
		} else {
			false
		}
    }
}