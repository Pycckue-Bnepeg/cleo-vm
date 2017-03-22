use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use script::Script;
use script::error::OpcodeHandlerErr;

pub struct VirtualMachine {
    vars: Rc<RefCell<Vec<u32>>>,
    scripts: HashMap<String, Script>, 
    handlers: HashMap<u16, Box<Fn(&mut Script) -> Result<bool, OpcodeHandlerErr>>>,
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        VirtualMachine {
            vars: Rc::new(RefCell::new(Vec::new())),
            scripts: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    pub fn tick(&mut self) {
        for (name, mut script) in self.scripts.iter_mut() {
            if !script.is_active() {
                continue;
            }

            if let Some(opcode) = script.get_opcode() {
                if let Some(handler) = self.handlers.get(&opcode) {
                    match handler(&mut script) {
                        Ok(result) => script.set_cond_result(result),
                        Err(e) => {
                            let (offset, bytes) = script.get_error();
                            println!("Error: Script \"{}\" at [{:04X}]{} ({}), desc: {}", name, opcode, offset, pretty_bytes(bytes), e);
                        }
                    }
                } else {
                    let (offset, bytes) = script.get_error();
                    println!("Error: Script \"{}\" called undefined opcode {:X} at position {}, code: {:?}", name, opcode, offset, pretty_bytes(bytes));
                }
            } else {
                script.done = true;
            }
        }
    }

    pub fn append_script(&mut self, name: String, bytes: Vec<u8>) {
        let script = Script::new(&name, bytes, self.vars.clone());
        self.scripts.insert(name, script);
    }

    pub fn set_handler<F>(&mut self, opcode: u16, handler: F) where F: Fn(&mut Script) -> Result<bool, OpcodeHandlerErr> + 'static {
        self.handlers.insert(opcode, Box::new(handler));
    }

    pub fn is_done(&self, name: String) -> bool {
        match self.scripts.get(&name) {
            Some(ref script) => script.done,
            None => false,
        }
    }
}

pub fn pretty_bytes(bytes: &[u8]) -> String {
    let mut string = String::from("[ ");

    for byte in bytes.iter() {
        string.push_str(format!("{:02X}, ", byte).as_str());
    }

    string.pop();
    string.pop();
    string.push_str(" ]");

    string
}