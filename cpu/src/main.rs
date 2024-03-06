use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::process;

static OP_MASK: i64 = 0b11111111;
static OP_NOOP: i8 = 0b00000001;
static OP_HALT: i8 = 0b00000010;
static OP_MOV:i8 = 0b00000011;
static OP_ADD:i8 = 0b00000100;
//

static REG_MASK: i64 = 0b11111111 << 8;
static REG_EAX_ADDR: i64 = 0b1;
static REG_ECX_ADDR: i64 = 0b10;
static REG_EDX_ADDR: i64 = 0b11;
static REG_EBX_ADDR: i64 = 0b100;

//static VAL_MASK: i64 = 0b11111111 << 16;

fn load_code(file_name: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let fh = File::open(file_name)?;
    Ok(io::BufReader::new(fh).lines())
}

// 4 bits: opcode
// |0000|
fn decode(line: &str, _registers: &mut [i32; 16], op_list: &HashMap<&str, i8>) -> Result<i64, String> {
    let mut inst: i64 = 0;
    for token in line.split_whitespace() {
        if inst & OP_MASK == 0 {
            match op_list.get(token) {
                Some(op) => inst = inst | i64::from(op.clone()),
                _ => return Err(format!("Unsupported OP: {}", token)),
            }
        } else if inst & REG_MASK == 0 {
            if token == "eax" {
                inst = inst | i64::from(REG_EAX_ADDR) << 8;
            } else if token == "ecx" {
                inst = inst | i64::from(REG_ECX_ADDR) << 8;
            } else if token == "edx" {
                inst = inst | i64::from(REG_EDX_ADDR) << 8;
            } else if token == "ebx" {
                inst = inst | i64::from(REG_EBX_ADDR) << 8;
            }
        } else {
            let val = token.parse::<i32>().unwrap();
            inst = inst | i64::from(val) << 16;
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
    ]);

    let mut registers: [i32; 16] = [0; 16];

    let code_reader = match load_code("code.rsm") {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Problem parsing code: {err}");
            process::exit(2);
        }
    };

    let mut reg_inst: i64;
    for line in code_reader.flatten() {
        match decode(&line, &mut registers, &op_list) {
            Err(err) => {
                eprintln!("decoding error: {}", err);
                process::exit(1);
            }
            Ok(inst) => reg_inst = inst,
        }
        println!("{:#064b}", reg_inst);

        let op_code = (reg_inst & OP_MASK) as i8;
        println!("op: {:#b}", op_code);

        if op_code == OP_NOOP {
            println!("no op");
        } else if op_code == OP_HALT {
            println!("halt!!");
            process::exit(0);
        } else if op_code == OP_MOV {
            let reg_addr = ((reg_inst & REG_MASK) >> 8) as usize;
            let val = (reg_inst >> 16) as i32;
            println!("mov {:#b} into reg at addr {:#b}", val, reg_addr);
            registers[reg_addr] = val;
        } else if op_code == OP_ADD {
            let reg_addr = ((reg_inst & REG_MASK) >> 8) as usize;
            let val = (reg_inst >> 16) as i32;
            println!("add {:#b} into reg at addr {:#b}", val, reg_addr);
            registers[reg_addr] = registers[reg_addr] + val;
        }
        println!("next ......");
    }

    println!("Hello, world!");
}
