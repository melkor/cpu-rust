use std::fs::File;
use std::io::{self, Read};
use std::process;

enum OpCodes {
    OpNop,
    OpHalt,
    OpMov,
    OpAdd,
}

fn load_code(file_name: &str) -> Result<String, io::Error> {
    let mut code = String::new();
    File::open(file_name)?.read_to_string(&mut code)?;
    Ok(code)
}

fn main() {
    let _reg_eax = "";
    let _reg_ebx = "";
    let _reg_ecx = "";
    let _reg_edx = "";

    let op_code = OpCodes::OpNop;

    let code = match load_code("code.rsm") {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Problem parsing code: {err}");
            process::exit(2);
        },
    };

    let mut current_inst = String::new();

    for c in code.chars() {
        if c != '\n' {
            current_inst.push(c);
        }
    }

    println!("{}", current_inst);

    match op_code {
        OpCodes::OpNop => println!("no op"),
        OpCodes::OpHalt => { 
            println!("halt");
            process::exit(0);
        },
        OpCodes::OpMov => println!("move"),
        OpCodes::OpAdd => println!("add"),
    }
    

    println!("Hello, world!");
}
