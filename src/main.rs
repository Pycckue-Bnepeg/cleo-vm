mod interpreter;

fn main() {
    let bytecode: Vec<u8> = vec![0x00, 0x00, 0x05, 0xFF, 0xF0, 0x01, 0x00, 0x0E, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F];

    let mut cleo_vm = interpreter::VirtualMachine::new();
    
    cleo_vm.append_script(bytecode);
    cleo_vm.run();

    //cleo_vm.init_default_opcodes();
    //cleo_vm.append_script(script);
    //cleo_vm.run();
}
