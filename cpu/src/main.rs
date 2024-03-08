use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Seek, BufRead};
use std::process;

static OP_SIZE: usize = 5;
static OP_MASK: u64 = 0b11111;
static OP_NOOP: u8 = 0b00001;
static OP_HALT: u8 = 0b00010;
static OP_MOV:  u8 = 0b00011;
static OP_ADD:  u8 = 0b00100;
static OP_POP:  u8 = 0b00101;
static OP_PUSH: u8 = 0b00110;
static OP_INT:  u8 = 0b00111;
static OP_CMP:  u8 = 0b01000;
static OP_JMP:  u8 = 0b01001;
static OP_JE:   u8 = 0b01010;
static OP_JNE:  u8 = 0b01011;
static OP_JL:   u8 = 0b01100;
static OP_JLE:  u8 = 0b01101;
static OP_JG:   u8 = 0b01110;
static OP_JGE:  u8 = 0b01111;

static TYPE_SIZE: usize = 2;
static TYPE_MASK: u8 = 0b11;
static TYPE_VAL:  u8 = 0b01;
static TYPE_REG:  u8 = 0b10;
static TYPE_ADDR: u8 = 0b11;

static REG_SIZE: usize = 32;
static REG_MASK: u64 = 0b11111111111111111111111111111111;
static REG_EAX_ADDR: u64 = 0b0001;
static REG_ECX_ADDR: u64 = 0b0010;
static REG_EDX_ADDR: u64 = 0b0011;
static REG_EBX_ADDR: u64 = 0b0100;

static REG_FLAG_ADDR: usize = 0b1000;
static CF_FLAG: u64 = 1;
static ZF_FLAG: u64 = 6;

static REG_TYPE_BITE_OFFSET: usize = OP_SIZE;
static REG_VALUE_BITE_OFFSET: usize = TYPE_SIZE + OP_SIZE;
static VAL_TYPE_BITE_OFFSET: usize = REG_SIZE + TYPE_SIZE + OP_SIZE;
static VAL_VALUE_BITE_OFFSET: usize = TYPE_SIZE + REG_SIZE + TYPE_SIZE + OP_SIZE;

fn load_inst(token: &str, type_bite_offset: usize, val_bite_offset: usize, inst: &mut u64, tag_addr: &HashMap<String, u64>) {
    if token == "eax" {
        *inst = *inst | u64::from(TYPE_REG) << type_bite_offset;
        *inst = *inst | REG_EAX_ADDR << val_bite_offset;
    } else if token == "ecx" {
        *inst = *inst | u64::from(TYPE_REG) << type_bite_offset;
        *inst = *inst | REG_ECX_ADDR << val_bite_offset;
    } else if token == "edx" {
        *inst = *inst | u64::from(TYPE_REG) << type_bite_offset;
        *inst = *inst | REG_EDX_ADDR << val_bite_offset;
    } else if token == "ebx" {
        *inst = *inst | u64::from(TYPE_REG) << type_bite_offset;
        *inst = *inst | REG_EBX_ADDR << val_bite_offset;
    } else {
        match token.parse::<u32>() {
            Ok(val) => { 
                *inst = *inst | u64::from(TYPE_VAL) << type_bite_offset;
                *inst = *inst | u64::from(val) << val_bite_offset;
            },
            Err(_) => {
                if tag_addr.contains_key(token) {
                    *inst = *inst | u64::from(TYPE_ADDR) << type_bite_offset;
                    let addr = tag_addr.get(token).unwrap(); 
                    *inst = *inst | u64::from(*addr) << val_bite_offset;
                } 
            },
        }
    }
}

fn decode(line: &str, op_list: &HashMap<&str, u8>, tag_addr: &HashMap<String, u64>) -> Result<u64, String> {
    let mut inst: u64 = 0;
    for token in line.split_whitespace() {
        if token.starts_with(";") {
            break;
        }
        if token.contains(":") {
            continue;
        }
        if inst & OP_MASK == 0 {
            match op_list.get(token) {
                Some(op) => inst = inst | u64::from(op.clone()),
                _ => return Err(format!("Unsupported OP: {}", token)),
            }
        } else if inst & (REG_MASK << OP_SIZE) == 0 {
            load_inst(token, REG_TYPE_BITE_OFFSET, REG_VALUE_BITE_OFFSET, &mut inst, tag_addr) 
        } else {
            load_inst(token, VAL_TYPE_BITE_OFFSET, VAL_VALUE_BITE_OFFSET, &mut inst, tag_addr) 
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
        ("CMP", OP_CMP),
        ("JMP", OP_JMP),
        ("JE", OP_JE),
        ("JNE", OP_JNE),
        ("JL", OP_JL),
        ("JLE", OP_JLE),
        ("JG", OP_JG),
        ("JGE", OP_JGE),
    ]);

    let mut registers: [u64; 16] = [0; 16];

    //let mut memories: [i64; 1024] = [0; 1024];
    let mut memories: [u64; 128] = [0; 128];
    let mut ip = 0; // instruction pointer

    let mut fh = match File::open("code.rsm") {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Cannot parse code: {err}");
            process::exit(2);
        }
    };
    let mut code_reader = io::BufReader::new(&fh);

    let mut tag_addr = HashMap::new();
    for l in code_reader.lines() {
        let line = l.unwrap();
        if line.is_empty() {
            continue;
        }
        if line.starts_with(";") {
            continue;
        }
        for token in line.split_whitespace() {
            if token.trim().is_empty() {
                continue;
            }
            if token.contains(":") {
                let mut words = token.split(":");
                let tag = words.next().unwrap();
                tag_addr.insert(String::from(tag), ip as u64);
            }
        }
        ip = ip + 1;
    }

    if let Err(err) = fh.rewind() {
        eprintln!("Problem rewind file: {err}");
        process::exit(2);
    }

    ip = 0;
    code_reader = io::BufReader::new(&fh);
    for line in code_reader.lines() {
        let line = line.unwrap();
        match decode(&line, &op_list, &tag_addr) {
            Ok(inst) => {
                if inst & 0xFFFFFFFFFFFFFFFF != 0 {
                    memories[ip] = inst;
                    ip += 1;
                }
            }
            Err(err) => {
                eprintln!("decoding error: {}", err);
                process::exit(1);
            }
        }
    }
    drop(fh); 

    let low_sp = ip; // the lowest stack address possible
    let mut sp = low_sp; // stack pointer
    ip = 0;

    println!("\n\nDump tag:");
    for (tag, addr) in tag_addr.clone() {
        println!("{} => {:#010b}", tag, addr);
    }

    println!("\n\nDump inst in memories:");
    for i in 0..low_sp {
        println!("{:#011b} => {:#066b}", i, memories[i]);
    }

    println!("\n\nStart execution:");
    while ip < low_sp { 
        let reg_inst = memories[ip];
        println!("inst: {:#064b}", reg_inst);

        let op_code = (reg_inst & OP_MASK) as u8;
        let reg_type = (reg_inst >> OP_SIZE) as u8 & TYPE_MASK;
        let reg_val = reg_inst >> (TYPE_SIZE + OP_SIZE) & REG_MASK;
        let val_type = (reg_inst >> (REG_SIZE + TYPE_SIZE + OP_SIZE)) as u8 & TYPE_MASK;
        let val = (reg_inst >> (TYPE_SIZE + REG_SIZE + TYPE_SIZE + OP_SIZE)) as u64;

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
                memories[sp] = reg_val; 
            } else if reg_type == TYPE_REG {
                println!("push register '{:#b}' into stack at add'{:#b}'", reg_val, sp);
                memories[sp] = registers[reg_val as usize]; 
            }
            sp += 1;
        } else if op_code == OP_POP {
            sp -= 1;
            println!("pop from stack '{:#b}' into register '{:#b}'", sp, reg_val);
            registers[reg_val as usize] = memories[sp];
        } else if op_code == OP_INT {
            if registers[REG_EAX_ADDR as usize] == 4 {
                println!("interupt display");
                let addr = registers[REG_ECX_ADDR as usize] as usize + low_sp;
                println!("{}", memories[addr]);
            }
        } else if op_code == OP_CMP {
            let mut val1: i64 = 0;
            if reg_type == TYPE_REG {
                val1 = registers[reg_val as usize] as i64;
            } else if reg_type == TYPE_VAL {
                val1 = reg_val as i64;
            }

            let mut val2: i64 = 0;
            if val_type == TYPE_REG {
                val2 = registers[val as usize] as i64;
            } else if val_type == TYPE_VAL {
                val2 = val as i64;
            }

            if val1 - val2 == 0 {
                println!("CMP val1 == val2");
                registers[REG_FLAG_ADDR] = registers[REG_FLAG_ADDR] | 1 << ZF_FLAG; 
            } else if val1 > val2 {
                println!("CMP val1 > val2");
                registers[REG_FLAG_ADDR] = registers[REG_FLAG_ADDR] | 0 << ZF_FLAG; 
                registers[REG_FLAG_ADDR] = registers[REG_FLAG_ADDR] | 0 << CF_FLAG; 
            } else {
                println!("CMP val1 < val2");
                registers[REG_FLAG_ADDR] = registers[REG_FLAG_ADDR] | 0 << ZF_FLAG; 
                registers[REG_FLAG_ADDR] = registers[REG_FLAG_ADDR] | 1 << CF_FLAG; 
            }
        } else if op_code == OP_JMP {
            ip = reg_val as usize;
            continue;
        } else if op_code == OP_JE {
            if registers[REG_FLAG_ADDR] & (1 << ZF_FLAG) != 0 { 
                ip = reg_val as usize;
                println!("jump equal to {:#b}", ip);
                continue;
            }
        } else if op_code == OP_JNE {
            if registers[REG_FLAG_ADDR] & (1 << ZF_FLAG) == 0 { 
                ip = reg_val as usize;
                println!("jump not equal to {:#b}", ip);
                continue;
            }
        } else if op_code == OP_JL {
            if registers[REG_FLAG_ADDR] & (1 << ZF_FLAG) == 0 &&
               registers[REG_FLAG_ADDR] & (1 << CF_FLAG) != 0 
            { 
                ip = reg_val as usize;
                println!("jump lower to {:#b}", ip);
                continue;
            }
        } else if op_code == OP_JLE {
            if registers[REG_FLAG_ADDR] & (1 << ZF_FLAG) != 0 ||
               ( registers[REG_FLAG_ADDR] & (1 << ZF_FLAG) == 0 &&
                 registers[REG_FLAG_ADDR] & (1 << CF_FLAG) != 0 )
            { 
                ip = reg_val as usize;
                println!("jump lower or equal to {:#b}", ip);
                continue;
            }
        } else if op_code == OP_JG {
            if registers[REG_FLAG_ADDR] & (1 << ZF_FLAG) == 0 &&
               registers[REG_FLAG_ADDR] & (1 << CF_FLAG) == 0 
            { 
                ip = reg_val as usize;
                println!("jump greater to {:#b}", ip);
                continue;
            }
        } else if op_code == OP_JGE {
            if registers[REG_FLAG_ADDR] & (1 << ZF_FLAG) != 0 ||
               ( registers[REG_FLAG_ADDR] & (1 << ZF_FLAG) == 0 &&
                 registers[REG_FLAG_ADDR] & (1 << CF_FLAG) == 0 )
            { 
                ip = reg_val as usize;
                println!("jump greater or equal to {:#b}", ip);
                continue;
            }
        }
        println!("next ......");
        ip += 1;
    }

    println!("\n\nDump registers:");
    for (index, item) in registers.iter().enumerate() {
        println!("{:#06b} => {:#066b}", index, item);
    }

    println!("\n\nDump stack:");
    println!("Stack pointer (SP): {:#010b}", sp);
    for i in low_sp..sp {
        println!("{:#011b} => {:#066b}", i, memories[i]);
    }
}
