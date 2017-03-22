extern crate alloc;

use std::rc::Rc;
use std::cell::RefCell;
use ::script::error::OpcodeHandlerErr;
use self::alloc::heap;
use ::std::mem::transmute;
use ::script::parser::ArgType;
use ::script::LogicalOpcode;
use ::script::variable::{Variable, VariableKind, VariableType, VarInfo};
use ::script::parser::Parser;

macro_rules! math_op {
	( $name:ident, $opcode:expr, $ops:tt ) => (
		$name.set_handler($opcode, |thread| {
			match thread.parse_var() {
				Some(rc_var) => {
					let mut var = rc_var.borrow_mut();
					thread.parse_any_arg()
						.ok_or(OpcodeHandlerErr::CannotParseArg)
						.map(|arg| {
							match arg {
								ArgType::Integer(value) => {
									let last = var.get::<u32>();
									var.set(last $ops value);
								},
								ArgType::Float(value) => {
									let last = var.get::<f32>();
									var.set(last $ops value);
								},
								ArgType::Var(rc_var_r) => {
									let var_r = rc_var_r.borrow();
									match var_r.kind {
										VariableKind::Float => var.do_stuff(&var_r, |a: f32, b: f32| a $ops b),
										VariableKind::Integer => var.do_stuff(&var_r, |a: u32, b: u32| a $ops b),
										_ => unimplemented!(),
									};
								},
								_ => unimplemented!(),
							};
							true
						})
				}
				None => Err(OpcodeHandlerErr::NotCorrectType("var".to_string())),
			}
		});
	)
}

macro_rules! cond_op {
	( $name:ident, $opcode:expr, $ops:tt ) => (
		$name.set_handler($opcode, |thread| {
			match thread.parse_var() {
				Some(rc_var) => {
					if let Some(arg) = thread.parse_any_arg() {
						let var = rc_var.borrow();
						match arg {
							ArgType::Integer(value) => Ok(var.get::<u32>() $ops value),
							ArgType::Float(value) => Ok(var.get::<f32>() $ops value),
							ArgType::Var(ref rc_var_r) => {
								let var_r = rc_var_r.borrow();
								if var_r.eq_types(&var) {
									match var.kind.clone() {
										VariableKind::Float => Ok(var.get::<f32>() $ops var_r.get::<f32>()),
										VariableKind::Integer => Ok(var.get::<i32>() $ops var_r.get::<i32>()),
										VariableKind::String => Ok(false),
									}
								}
								else {
									Ok(false)
								}
							}
							_ => Ok(false),
						}
					} else {
						Err(OpcodeHandlerErr::CannotParseArg)
					}
				},
				None => Err(OpcodeHandlerErr::NotCorrectType("var".to_string())),
			}
		});
	)
}

#[derive(Debug)]
struct StackInfo {
	args_count: usize,
	return_ptr: usize,
}

pub trait DefaultOpcodes {
	fn set_default_opcodes(&mut self);
}

impl DefaultOpcodes for ::vm::VirtualMachine {
	fn set_default_opcodes(&mut self) {
		// 0000: NOP
		self.set_handler(0x0000, |thread| {
			Ok(true)
		});

		// 0001: wait %int%
		self.set_handler(0x0001, |thread| {
			thread.parse_int().map(|time| thread.set_wake_up(time as u32)).ok_or(OpcodeHandlerErr::CannotParseArg)
		});

		// 0002: jump %int%
		self.set_handler(0x0002, |thread| {
			thread.parse_int().map(|address| thread.jump_to(address as usize)).ok_or(OpcodeHandlerErr::CannotParseArg)
		});

		// 0003: %var% = %any%
		self.set_handler(0x0003, |thread| {
			match thread.parse_var() {
				Some(rc_var) => {
					let mut var = rc_var.borrow_mut();
					thread.parse_any_arg().ok_or(OpcodeHandlerErr::CannotParseArg).map(|arg| {
						match arg {
							ArgType::Integer(value) => {
								var.kind = VariableKind::Integer;
								var.set(value); 
								true
							}
							ArgType::Float(value) => {
								var.kind = VariableKind::Float;
								var.set(value); 
								true
							}
							ArgType::Var(var_r) => {
								var.from(&var_r.borrow()); 
								true
							}
							ArgType::String(value) => {
								var.kind = VariableKind::String;
								var.set_str(value);
								true
							}
							_ => false,
						}
					})
				}
				None => Err(OpcodeHandlerErr::CannotParseArg),
			}
		});

		// 0004-0007: %var% `op`= %any%
		math_op!(self, 0x0004, +);
		math_op!(self, 0x0005, -);
		math_op!(self, 0x0006, *);
		math_op!(self, 0x0007, /);

		// 0008: if %int%
		self.set_handler(0x0008, |thread| {
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
		});

		// 0009: jump_if_false %int%
		self.set_handler(0x0009, |thread| {
			match thread.parse_int() {
				Some(offset) => {					
					if !thread.cond_result {
						thread.jump_to(offset as usize);
					}
					
					Ok(thread.cond_result)
				},
				None => Err(OpcodeHandlerErr::CannotParseArg),
			}
		});

		// 000A: print %any%
		self.set_handler(0x000A, |thread| {
			if let Some(arg) = thread.parse_any_arg() {
				match arg {
					ArgType::Var(rc_var) => println!("{}", *rc_var.borrow()),
					_ => println!("{}", arg),
				}
				Ok(true)
			} else {
				Err(OpcodeHandlerErr::CannotParseArg)
			}
		});

		// 000B: get_label_address %int% to %var%
		self.set_handler(0x000B, |thread| {
			match thread.parse_int() {
				Some(label) => {
					match thread.parse_var() {
						Some(rc_var) => {
							let mut var = rc_var.borrow_mut();

							let address: u32 = unsafe {
								let temp = &thread.bytes[0xFFFFFFFF - (label as usize) + 1];
								transmute(temp)
							};
							var.set(address);
							Ok(true)
						},
						None => Err(OpcodeHandlerErr::CannotParseArg),
					}
				},
				None => Err(OpcodeHandlerErr::CannotParseArg),
			}
		});
		
		// 000C: allocate %int% to %var%
		self.set_handler(0x000C, |thread| {
			match thread.parse_int() {
				Some(size) => {
					match thread.parse_var() {
						Some(rc_var) => {
							let mut var = rc_var.borrow_mut();

							let address: *mut u8 = unsafe {
								heap::allocate(size as usize, 1)
							};

							var.set(address as u32);
							Ok(true)
						},
						None => Err(OpcodeHandlerErr::CannotParseArg),
					}
				},
				None => Err(OpcodeHandlerErr::CannotParseArg),
			}	
		});

		// 000D: deallocate %var% size %int%
		self.set_handler(0x000D, |thread| {
			match thread.parse_var() {
				Some(rc_var) => {
					match thread.parse_int() {
						Some(size) => {
							let var = rc_var.borrow();

							let ptr: *mut u8 = unsafe {
								transmute(var.get::<u32>())
							};
							
							if ptr as usize != 0 {
								unsafe {
									heap::deallocate(ptr, size as usize, 1);
								}
								Ok(true)
							} else {
								Ok(false)
							}
						},
						None => Err(OpcodeHandlerErr::CannotParseArg),
					}
				},
				None => Err(OpcodeHandlerErr::CannotParseArg),
			}	
		});

		// 000E: call %int% args %int% %args...% %ret...%
		self.set_handler(0x000E, |thread| {
			let label = thread.parse_int().unwrap() as usize;
			let count = thread.parse_int().unwrap() as usize;

			for i in 0 .. count {
				let info = thread.get_variable(VariableType::Local(i as usize));
				let mut var = info.borrow_mut();

				thread.stack.push(Box::into_raw(Box::new(var.into_raw())) as usize);
				match thread.parse_any_arg() {
					Some(ArgType::Var(rc_var)) => {
						let var_r = rc_var.borrow();
						var.from(&var_r);
					}
					Some(ArgType::Integer(value)) => var.set(value),
					Some(ArgType::Float(value)) => var.set(value),
					_ => unimplemented!()
				}
			}

			let info = Box::new(StackInfo {
				args_count: count,
				return_ptr: thread.offset,
			});

			thread.stack.push(Box::into_raw(info) as usize);
			thread.jump_to(label);
			
			Ok(true)
		});

		// 000F: ret %int% args %int% %args...%
		self.set_handler(0x000F, |thread| {
			let count = thread.parse_int().unwrap();
			let mut ret_args: Vec<ArgType> = Vec::new();
			
			for _ in 0 .. count {
				ret_args.push(thread.parse_any_arg().unwrap());
			}

			let info: Box<StackInfo> = {
				let raw_info = thread.stack.pop().unwrap();
				unsafe {
					Box::from_raw(raw_info as *mut StackInfo)
				}
			};

			thread.offset = info.return_ptr;

			for i in 0 .. info.args_count {
				let raw = {
					let t = thread.stack.pop().unwrap();
					unsafe {
						Box::from_raw(t as *mut VarInfo)
					}
				};
				thread.local_vars[i] = Rc::new(RefCell::new(Variable::from_raw(i, &raw)));
			}

			for i in 0 .. count {
				let rc_var = thread.parse_var().unwrap();
				let mut var = rc_var.borrow_mut();
				
				match ret_args[i as usize] {
					ArgType::Integer(value) => var.set(value),
					ArgType::Float(value) => var.set(value),
					ArgType::Var(ref rc_var_r) => {
						let var_r = rc_var_r.borrow();
						var.from(&var_r);
					}
					_ => unimplemented!(),
				}
			}

			thread.offset += 1;
			Ok(true)
		});

		// 0010: %var% == %any%
		cond_op!(self, 0x0010, ==);
		cond_op!(self, 0x0011, !=);
		cond_op!(self, 0x0012, >);
		cond_op!(self, 0x0013, <);
		cond_op!(self, 0x0014, >=);
		cond_op!(self, 0x0015, <=);

		/*
		TODO: 
			logical opcodes for usize
		
		math_op!(self, 0x0016, &);
		math_op!(self, 0x0017, |);
		math_op!(self, 0x0018, ^);
		math_op!(self, 0x0019, %);
		*/
	}
}