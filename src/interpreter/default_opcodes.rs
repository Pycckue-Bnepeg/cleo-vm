use super::{Variable, LogicalOpcode, ArgType};
use super::error::OpcodeHandlerErr;

pub trait DefaultOpcodes {
	fn set_default_opcodes(&mut self);
}

impl DefaultOpcodes for super::VirtualMachine {
	fn set_default_opcodes(&mut self) {
		// 0001: wait %int%
		self.set_opcode_handler(0x0001, Box::new(|thread| {
			match thread.parse_int() {
				Some(time) => {
					thread.set_wake_up(time as u32);
					Ok(true)
				},
				None => Err(OpcodeHandlerErr::CannotParseArg),
			}
		}));

		// 0002: jump %int%
		self.set_opcode_handler(0x0002, Box::new(|thread| {
			match thread.parse_int() {
				Some(address) => {
					thread.jump_to(address as usize);
					Ok(true)
				},
				None => Err(OpcodeHandlerErr::CannotParseArg),
			}
		}));

		// 0003: %var% = %any%
		self.set_opcode_handler(0x0003, Box::new(|thread| {
			if let Some(var) = thread.parse_variable() {
				if let Some(arg) = thread.get_any_arg() {
					match arg {
						ArgType::Integer(value) => thread.set_variable(&var, value),
						ArgType::Float(value) => thread.set_variable_float(&var, value),
						ArgType::String(value) => unimplemented!(),
						ArgType::Variable(ref var_r) => thread.set_variable(&var, thread.get_variable(&var_r)),
						ArgType::None => return Err(OpcodeHandlerErr::NotCorrectType),
					}
					Ok(true)
				} else {
					Err(OpcodeHandlerErr::CannotParseArg)
				}
			} else {
				Err(OpcodeHandlerErr::CannotParseArg)
			}
		}));

		// 0004: %var% += %any%
		self.set_opcode_handler(0x0004, Box::new(|thread| {
			if let Some(var) = thread.parse_variable() {
				if let Some(arg) = thread.get_any_arg() {
					match arg {
						ArgType::Integer(value) => thread.set_variable(&var, thread.get_variable(&var) + value),
						ArgType::Float(value) => thread.set_variable_float(&var, thread.get_variable_float(&var) + value),
						ArgType::String(value) => unimplemented!(),
						ArgType::Variable(ref var_r) => thread.set_variable(&var, thread.get_variable(&var_r) + thread.get_variable(&var)),
						ArgType::None => return Err(OpcodeHandlerErr::NotCorrectType),
					}
					Ok(true)
				} else {
					Err(OpcodeHandlerErr::CannotParseArg)
				}
			} else {
				Err(OpcodeHandlerErr::CannotParseArg)
			}
		}));

		// 0005: %var% -= %any%
		self.set_opcode_handler(0x0005, Box::new(|thread| {
			if let Some(var) = thread.parse_variable() {
				if let Some(arg) = thread.get_any_arg() {
					match arg {
						ArgType::Integer(value) => thread.set_variable(&var, thread.get_variable(&var) - value),
						ArgType::Float(value) => thread.set_variable_float(&var, thread.get_variable_float(&var) - value),
						ArgType::String(value) => unimplemented!(),
						ArgType::Variable(ref var_r) => thread.set_variable(&var, thread.get_variable(&var_r) - thread.get_variable(&var)),
						ArgType::None => return Err(OpcodeHandlerErr::NotCorrectType),
					}
					Ok(true)
				} else {
					Err(OpcodeHandlerErr::CannotParseArg)
				}
			} else {
				Err(OpcodeHandlerErr::CannotParseArg)
			}
		}));

		// 0006: %var% *= %any%
		self.set_opcode_handler(0x0006, Box::new(|thread| {
			if let Some(var) = thread.parse_variable() {
				if let Some(arg) = thread.get_any_arg() {
					match arg {
						ArgType::Integer(value) => thread.set_variable(&var, thread.get_variable(&var) * value),
						ArgType::Float(value) => thread.set_variable_float(&var, thread.get_variable_float(&var) * value),
						ArgType::String(value) => unimplemented!(),
						ArgType::Variable(ref var_r) => thread.set_variable(&var, thread.get_variable(&var_r) * thread.get_variable(&var)),
						ArgType::None => return Err(OpcodeHandlerErr::NotCorrectType),
					}
					Ok(true)
				} else {
					Err(OpcodeHandlerErr::CannotParseArg)
				}
			} else {
				Err(OpcodeHandlerErr::CannotParseArg)
			}
		}));

		// 0007: %var% /= %any%
		self.set_opcode_handler(0x0007, Box::new(|thread| {
			if let Some(var) = thread.parse_variable() {
				if let Some(arg) = thread.get_any_arg() {
					match arg {
						ArgType::Integer(value) => thread.set_variable(&var, thread.get_variable(&var) / value),
						ArgType::Float(value) => thread.set_variable_float(&var, thread.get_variable_float(&var) / value),
						ArgType::String(value) => unimplemented!(),
						ArgType::Variable(ref var_r) => thread.set_variable(&var, thread.get_variable(&var_r) / thread.get_variable(&var)),
						ArgType::None => return Err(OpcodeHandlerErr::NotCorrectType),
					}
					Ok(true)
				} else {
					Err(OpcodeHandlerErr::CannotParseArg)
				}
			} else {
				Err(OpcodeHandlerErr::CannotParseArg)
			}
		}));

		// 0008: if %int%
		self.set_opcode_handler(0x0008, Box::new(|thread| {
			if let Some(arg) = thread.parse_int() {
				match arg {
					0 => Ok(thread.set_logical_opcode(LogicalOpcode::One)),
					1 ... 7 => Ok(thread.set_logical_opcode(LogicalOpcode::And)),
					21 ... 27 => Ok(thread.set_logical_opcode(LogicalOpcode::Or)),
					_ => Err(OpcodeHandlerErr::UndefinedCondArg),
				}
			} else {
				Err(OpcodeHandlerErr::CannotParseArg)
			}
		}));

		// 0009: jump_if_false %int%
		self.set_opcode_handler(0x0009, Box::new(|thread| {
			match thread.parse_int() {
				Some(offset) => {
					let cond_result = thread.condition_result.get();
					
					if !cond_result {
						thread.jump_to(offset as usize);
					}
					
					Ok(cond_result)
				},
				None => Err(OpcodeHandlerErr::CannotParseArg),
			}
		}));

		// 000A: print %any%
		self.set_opcode_handler(0x000A, Box::new(|thread| {
			if let Some(arg) = thread.get_any_arg() {
				match arg {
					ArgType::Variable(ref var) => println!("{} = {}", arg, thread.get_variable(&var)),
					_ => println!("{}", arg),
				}
				Ok(true)
			} else {
				Err(OpcodeHandlerErr::CannotParseArg)
			}
		}));
	}
}