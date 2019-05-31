//! For converting Chip8 machine code into a assembly language.

pub fn disassemble(opcode: u16) -> String
{
    //! Disassembles the provided opcode.

    //Split the 16-byte opcode into four 4-bit nibbles. This will allow us to use pattern matching to detect the opcode.
    let nibble3: u8 = ((opcode & 0xF000) >> 12) as u8;
    let nibble2: u8 = ((opcode & 0x0F00) >> 8)  as u8;
    let nibble1: u8 = ((opcode & 0xF0F0) >> 4)  as u8;
    let nibble0: u8 = ((opcode & 0xF00F) >> 0)  as u8;

    //Decode the current instruction then execute the instruction.
    let instruction_string: String = match (nibble3, nibble2, nibble1, nibble0)
    {
        (0x0, 0x0, 0xE, 0x0) => String::from("CLS"), //CLS
        (0x0, 0x0, 0xE, 0xE) => String::from("RET"), //RET
        (0x0,   _,   _,   _) => format!("SYS {address:X}", address=(opcode & 0x0FFF)),  //SYS nnn
        (0x1,   _,   _,   _) => format!("JP {address:X}",  address=(opcode & 0x0FFF)),  //JP nnn
        (0x2,   _,   _,   _) => format!("CALL {address:X}", address=(opcode & 0x0FFF)), //CALL nnn
        (0x3,   _,   _,   _) => format!("SE {register:X} {value:X}",  register=nibble2, value=(opcode & 0x00FF)),  //SE x nn
        (0x4,   _,   _,   _) => format!("SNE {register:X} {value:X}",  register=nibble2, value=(opcode & 0x00FF)), //SNE x nn
        (0x5,   _,   _, 0x0) => format!("SE {register1:X} {register2:X}",  register1=nibble2, register2=nibble1),  //SE x y
        (0x6,   _,   _,   _) => format!("LD {register:X} {value:X}",  register=nibble2, value=(opcode & 0x00FF)),   //LD x nn
        (0x7,   _,   _,   _) => format!("ADD {register:X} {value:X}",  register=nibble2, value=(opcode & 0x00FF)),  //ADD v nn
        (0x8,   _,   _, 0x0) => format!("LD {register1:X} {register2:X}",  register1=nibble2, register2=nibble1),  //LD x y
        (0x8,   _,   _, 0x1) => format!("OR {register1:X} {register2:X}",  register1=nibble2, register2=nibble1),  //OR x y
        (0x8,   _,   _, 0x2) => format!("AND {register1:X} {register2:X}",  register1=nibble2, register2=nibble1), //AND x y
        (0x8,   _,   _, 0x3) => format!("XOR {register1:X} {register2:X}",  register1=nibble2, register2=nibble1), //XOR x y
        (0x8,   _,   _, 0x4) => format!("ADD {register1:X} {register2:X}",  register1=nibble2, register2=nibble1), //ADD x y
        (0x8,   _,   _, 0x5) => format!("SUB {register1:X} {register2:X}",  register1=nibble2, register2=nibble1), //SUB x y
        (0x8,   _,   _, 0x6) => format!("SHR {register1:X} {register2:X}",  register1=nibble2, register2=nibble1), //SHR x y
        (0x8,   _,   _, 0x7) => format!("SUBN {register1:X} {register2:X}",  register1=nibble2, register2=nibble1),//SUBN x y
        (0x8,   _,   _, 0xE) => format!("SHL {register1:X} {register2:X}",  register1=nibble2, register2=nibble1), //SHL x y
        (0x9,   _,   _, 0x0) => format!("SNE {register1:X} {register2:X}",  register1=nibble2, register2=nibble1), //SNE x y
        (0xA,   _,   _,   _) => format!("LD I {value:X}", value=(opcode & 0x0FFF)),                                 //LD x nn
        (0xB,   _,   _,   _) => format!("JP V0 {value:X}", value=(opcode & 0x0FFF)),                                //JP vx nn
        (0xC,   _,   _,   _) => format!("RND {register1:X}", register1=nibble2),                                   //RND x
        (0xD,   _,   _,   _) => format!("SNE {register1:X} {register2:X} {value:X}",  register1=nibble2, register2=nibble1, value=nibble0), //DRW x y n
        (0xE,   _, 0x9, 0xE) => format!("SKP {register1:X}", register1=nibble2),
        (0xE,   _, 0xA, 0x1) => format!("SKNP {register1:X}", register1=nibble2),
        (0xF,   _, 0x0, 0x7) => format!("LD {register1:X} DT", register1=nibble2),
        (0xF,   _, 0x0, 0xA) => format!("LD {register1:X} K", register1=nibble2),
        (0xF,   _, 0x1, 0x5) => format!("LD DT {register1:X}", register1=nibble2),
        (0xF,   _, 0x1, 0x8) => format!("LD ST {register1:X}", register1=nibble2),
        (0xF,   _, 0x1, 0xE) => format!("ADD I {register1:X}", register1=nibble2),
        (0xF,   _, 0x2, 0x9) => format!("LD F {register1:X}", register1=nibble2),
        (0xF,   _, 0x3, 0x3) => format!("LD B {register1:X}", register1=nibble2),
        (0xF,   _, 0x5, 0x5) => format!("LD [I] {register1:X}", register1=nibble2),
        (0xF,   _, 0x6, 0x5) => format!("LD {register1:X} [I]", register1=nibble2),
        (  _,   _,   _,   _) => (String::from("?")),
    };

    instruction_string
}