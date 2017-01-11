use super::{Variable, LogicalOpcode, ArgType};

pub trait DefaultOpcodes {
	fn set_default_opcodes(&mut self);
}

impl DefaultOpcodes for super::VirtualMachine {
	fn set_default_opcodes(&mut self) {
		// 0001: wait int%time%
		self.set_opcode_handler(0x0001, Box::new(|thread| {
			match thread.parse_int() {
				Some(time) => {
					println!("sleep {}", time as u32);
					thread.set_wake_up(time as u32);
					true
				},
				None => false,
			}
		}));

		// 0002: jump int%address%
		self.set_opcode_handler(0x0002, Box::new(|thread| {
			match thread.parse_int() {
				Some(address) => {
					thread.jump_to(address as usize);
					true
				},
				None => false,
			}
		}));

		// 0005: %g_var% = %int%
		// 0006: %l_var% = %int%
		self.set_opcode_handler(0x0006, Box::new(|thread| {
			if let Some(var) = thread.parse_variable() {
				match thread.parse_int() {
					Some(value) => {
						thread.set_variable(&var, value);
						true
					},
					None => false,
				}
			} else {
				false
			}
		}));

		// 000A: %l_var% += %int%
		self.set_opcode_handler(0x000A, Box::new(|thread| {
			if let Some(var) = thread.parse_variable() {
				match thread.parse_int() {
					Some(value) => {
						let old_value = thread.get_variable(&var);
						thread.set_variable(&var, old_value + value);
						true
					},
					None => false,
				}
			} else {
				false
			}
		}));

		// 00D6: if %int%
		self.set_opcode_handler(0x00D6, Box::new(|thread| {
			if let Some(arg) = thread.parse_int() {
				match arg {
					0 => thread.set_logical_opcode(LogicalOpcode::One),
					1 ... 7 => thread.set_logical_opcode(LogicalOpcode::And),
					21 ... 27 => thread.set_logical_opcode(LogicalOpcode::Or),
					_ => false,
				}
			} else {
				false
			}
		}));

		// 004D: jump_if_false %int%
		self.set_opcode_handler(0x004D, Box::new(|thread| {
			match thread.parse_int() {
				Some(offset) => {
					let cond_result = thread.condition_result.get();
					
					if !cond_result {
						thread.jump_to(offset as usize);
					}
					
					cond_result
				},
				None => false,
			}
		}));
	}
}