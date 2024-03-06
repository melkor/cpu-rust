use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::process;


static OP_SIZE: usize = 8;
static OP_MASK: i64 = 0b11111111;
static OP_NOOP: i8 = 0b00000001;
static OP_HALT: i8 = 0b00000010;
static OP_MOV:  i8 = 0b00000011;
static OP_ADD:  i8 = 0b00000100;
static OP_POP:  i8 = 0b00000101;
static OP_PUSH: i8 = 0b00000110;
static OP_INT: i8 = 0b00000111;

static TYPE_SIZE: usize = 4;
static TYPE_MASK: i8 = 0b0011;
static TYPE_VAL: i8 = 0b0001;
static TYPE_REG: i8 = 0b0010;
//static TYPE_ADDR: i8 = 0b11;

static REG_SIZE: usize = 8;
static REG_MASK: i64 = 0b11111111;
static REG_EAX_ADDR: i64 = 0b0001;
static REG_ECX_ADDR: i64 = 0b0010;
static REG_EDX_ADDR: i64 = 0b0011;
static REG_EBX_ADDR: i64 = 0b0100;

fn load_code(file_name: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let fh = File::open(file_name)?;
    Ok(io::BufReader::new(fh).lines())
}

// 4 bits: opcode
// |0000|
fn decode(line: &str, op_list: &HashMap<&str, i8>) -> Result<i64, String> {
    let mut inst: i64 = 0;
    for token in line.split_whitespace() {
        if inst & OP_MASK == 0 {
            match op_list.get(token) {
                Some(op) => inst = inst | i64::from(op.clone()),
                _ => return Err(format!("Unsupported OP: {}", token)),
            }
        } else if inst & (REG_MASK << OP_SIZE) == 0 {
            let type_bitewise = OP_SIZE;
            let val_bitewise = TYPE_SIZE + OP_SIZE;
            if token == "eax" {
                inst = inst | i64::from(TYPE_REG) << type_bitewise;
                inst = inst | i64::from(REG_EAX_ADDR) << val_bitewise;
            } else if token == "ecx" {
                inst = inst | i64::from(TYPE_REG) << type_bitewise;
                inst = inst | i64::from(REG_ECX_ADDR) << val_bitewise;
            } else if token == "edx" {
                inst = inst | i64::from(TYPE_REG) << type_bitewise;
                inst = inst | i64::from(REG_EDX_ADDR) << val_bitewise;
            } else if token == "ebx" {
                inst = inst | i64::from(TYPE_REG) << type_bitewise;
                inst = inst | i64::from(REG_EBX_ADDR) << val_bitewise;
            } else {
                match token.parse::<i32>() {
                    Ok(val) => { 
                        inst = inst | i64::from(TYPE_VAL) << type_bitewise;
                        inst = inst | i64::from(val) << val_bitewise;
                    },
                    Err(_) => println!("TODO todo"),
                }
            }
        } else {
            let type_bitewise = REG_SIZE + TYPE_SIZE + OP_SIZE;
            let val_bitewise = TYPE_SIZE + REG_SIZE + TYPE_SIZE + OP_SIZE;
            if token == "eax" {
                inst = inst | i64::from(TYPE_REG) << type_bitewise;
                inst = inst | i64::from(REG_EAX_ADDR) << val_bitewise;
            } else if token == "ecx" {
                inst = inst | i64::from(TYPE_REG) << type_bitewise;
                inst = inst | i64::from(REG_ECX_ADDR) << val_bitewise;
            } else if token == "edx" {
                inst = inst | i64::from(TYPE_REG) << type_bitewise;
                inst = inst | i64::from(REG_EDX_ADDR) << val_bitewise;
            } else if token == "ebx" {
                inst = inst | i64::from(TYPE_REG) << type_bitewise;
                inst = inst | i64::from(REG_EBX_ADDR) << val_bitewise;
            } else {
                match token.parse::<i32>() {
                    Ok(val) => { 
                        inst = inst | i64::from(TYPE_VAL) << type_bitewise;
                        inst = inst | i64::from(val) << val_bitewise;
                    },
                    Err(_) => println!("TODO todo"),
                }
            }
        }
    }
    Ok(inst)
}

fn main() {
    let op_list = HashMap::from([
        ("NOOP", OP_NOOP), 
        ("HALT", OP_HALT),
        ("MOV", OP_MOV),
        ("ADD", OP_ADD),
        ("PUSH", OP_PUSH),
        ("POP", OP_POP),
        ("INT", OP_INT),
    ]);

    let mut registers: [i32; 16] = [0; 16];
    let mut stack: [i32; 128] = [0; 128];
    let mut sp = 0;

    let code_reader = match load_code("code.rsm") {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Problem parsing code: {err}");
            process::exit(2);
        }
    };

    let mut reg_inst: i64;
    for line in code_reader.flatten() {
        if line.starts_with(";") {
            continue;
        }
        match decode(&line, &op_list) {
            Err(err) => {
                eprintln!("decoding error: {}", err);
                process::exit(1);
            }
            Ok(inst) => reg_inst = inst,
        }
        println!("inst: {:#064b}", reg_inst);

        let op_code = (reg_inst & OP_MASK) as i8;
        let reg_type = (reg_inst >> OP_SIZE) as i8 & TYPE_MASK;
        let reg_val = reg_inst >> (TYPE_SIZE + OP_SIZE) & i64::from(REG_MASK);
        let val_type = (reg_inst >> (REG_SIZE + TYPE_SIZE + OP_SIZE)) as i8 & TYPE_MASK;
        let val = (reg_inst >> (TYPE_SIZE + REG_SIZE + TYPE_SIZE + OP_SIZE)) as i32;

        if op_code == OP_NOOP {
            println!("no op");
        } else if op_code == OP_HALT {
            println!("halt!!");
            break;
        } else if op_code == OP_MOV {
            let reg_addr = reg_val as usize;
            if val_type == TYPE_VAL {
                println!("mov value '{:#b}' into reg at addr '{:#b}'", val, reg_addr);
                registers[reg_addr] = val;
            } else if val_type == TYPE_REG {
                println!("mov from reg at addr '{:#b}' into reg at addr '{:#b}'", val, reg_addr);
                registers[reg_addr] = registers[val as usize];
            }
        } else if op_code == OP_ADD {
            let reg_addr = reg_val as usize;
            if val_type == TYPE_VAL {
                println!("add value '{:#b}' into reg at addr '{:#b}'", val, reg_addr);
                registers[reg_addr] += val;
            } else if val_type == TYPE_REG {
                println!("add from reg at addr '{:#b}' into reg at addr '{:#b}'", val, reg_addr);
                registers[reg_addr] += registers[val as usize];
            }
        } else if op_code == OP_PUSH {
            if reg_type == TYPE_VAL {
                println!("push value '{:#b}' into stack at add'{:#b}'", reg_val, sp);
                stack[sp] = reg_val as i32; 
            } else if reg_type == TYPE_REG {
                println!("push register '{:#b}' into stack at add'{:#b}'", reg_val, sp);
                stack[sp] = registers[reg_val as usize]; 
            }
            sp += 1;
        } else if op_code == OP_POP {
            sp -= 1;
            println!("pop from stack '{:#b}' into register '{:#b}'", sp, reg_val);
            registers[reg_val as usize] = stack[sp];
        } else if op_code == OP_INT {
            if registers[REG_EAX_ADDR as usize] == 4 {
                println!("interupt display");
                println!("{}", stack[registers[REG_ECX_ADDR as usize] as usize]);
            }
        }
        println!("next ......");
    }

    println!("\n\nDump registers:");
    for (index, item) in registers.iter().enumerate() {
        println!("{:#06b} => {:#034b}", index, item);
    }

    println!("\n\nDump stack:");
    println!("Stack pointer (SP): {:#010b}", sp);
    for (index, item) in stack.iter().enumerate() {
        println!("{:#010b} => {:#034b}", index, item);
    }
}
