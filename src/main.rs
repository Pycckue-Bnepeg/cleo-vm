mod interpreter;
use interpreter::default_opcodes::DefaultOpcodes;

fn main() {
	let bytecode: Vec<u8> = vec![0x01, 0x00, 0x05, 0xD0, 0x07, 0x02, 0x00, 0x0E, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F];

	let mut cleo_vm = interpreter::VirtualMachine::new();

	cleo_vm.set_default_opcodes();
	cleo_vm.append_script(bytecode);
	cleo_vm.run();
}
