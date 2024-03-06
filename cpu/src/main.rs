use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::process;


static OP_MASK:i64 = 0b11111111;
static OP_NOOP:i8 = 0b00000000;
static OP_HALT:i8 = 0b00000001;
// static OP_MOV:i8 = 0b00000010;
// static OP_ADD:i8 = 0b00000011;


fn load_code(file_name: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let fh = File::open(file_name)?;
    Ok(io::BufReader::new(fh).lines())
}

// 4 bits: opcode 
// |0000| 
fn decode(line: &str, op_list: &HashMap<&str, i8>) -> Result<i64, String> {
    let mut inst:i64 = 0;
    for token in line.split_whitespace() {
        match op_list.get(token) {
            Some(op) => inst = inst | i64::from(op.clone()),
            _ => return Err(format!("Unsupported OP: {}", token)),
        }
    }
    Ok(inst)
}

fn main() {
    let _op_list = HashMap::from([
        ("NOOP", 0b0),
        ("HALT", 0b1),
    ]);

    let mut _reg_rax: i64 = 0;
    let mut _reg_rbx: i64 = 0;
    let mut _reg_rcx: i64 = 0;
    let mut _reg_rdx: i64 = 0;

    let code_reader = match load_code("code.rsm") {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Problem parsing code: {err}");
            process::exit(2);
        }
    };

    let mut reg_inst: i64;
    for line in code_reader.flatten() {
        match decode(&line, &_op_list) {
            Err(err) => {
                eprintln!("decoding error: {}", err);
                process::exit(1);
            },
            Ok(inst) => reg_inst = inst,
        }
        println!("{}", reg_inst);

        let op_code = (reg_inst & OP_MASK) as i8;
        println!("op: {}", reg_inst);

        if op_code == OP_NOOP {
            println!("no op");
        } else if op_code == OP_HALT {
            println!("halt!!");
            process::exit(0);
        }
    }

    println!("Hello, world!");
}
