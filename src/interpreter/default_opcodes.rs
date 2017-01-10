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

    	/// 0002: print string%text%
    	self.set_opcode_handler(0x0002, Box::new(|thread| {
        	match thread.parse_string() {
            	Some(text) => {
                	println!("{}", text);
                	true
            	},
            	None => false,
        	}
		}));
	}
}