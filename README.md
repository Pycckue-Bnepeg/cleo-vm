# CLEO Virtual Machine
## About Opcodes
Every opcode has the syntax: `0001: wait %d%` where `0001` is an opcode id, `%d` any param.

For example `0002: jump 0xABCD` as bytecode `02 00 05 CD AB`.

CLEO VM has the following types of arguments:

| Type       | ID | Example              | Result     |
|------------|----|----------------------|------------|
| Int8       | 04 | 04 EA                | 0xEA       |
| Int16      | 05 | 05 AB CD             | 0xCDAB     |
| Int32      | 01 | 01 AB CD EF 00       | 0x00EFCDAB |
| Float      | 06 | 06 06 5C 8F C2 3F    | 1.52       |
| Local var  | 03 | 03 0A 00             | 10@        |
| Global var | 02 | 02 0A 0F             | $3850      |
| String     | 0E | 0E 05 68 65 6C 6C 6F | "hello"    |

## Current Default Opcodes
### 0001: wait @int
Set a wake up timer for a script.
Example: `0001: wait 10`

### 0002: jump @label
Jump to address.
Example: `0002: jump @some_label`

### 0006: @var = @int
Binding variable (global or local). **Only integer**.
Example: `0006: 0@ = 10`

### 000A: @var += @int
Add some value to the variable. **Only integer**.
Example: `000A: $g_var = 5156`

### 00D6: if @int
Set flags of VM.  
`@int = 0` - only one opcode must be true (not or). `LogicalOpcode::One`.  
`@int = 1 .. 7` - same as AND. `LogicalOpcode::And`.  
`@int = 21 .. 27` - same as OR. `LogicalOpcode::Or`.

### 004D: jump_if_false @int
Jump to label if a condition is false. Example  
```
006D: if and
00AB: some_opcode 10 50
0AF0: some_opcode 2@
004D: jump_if_false @condition_false
// here is true
<some code>

:condition_false
// here is false
<some code>
```