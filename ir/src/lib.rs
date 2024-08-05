mod codegen;
mod lexer;
mod optimizer;
mod parser;

use std::{
    collections::VecDeque,
    fs::File,
    io::{BufWriter, Write},
};

use codegen::generate_code;
use lexer::parse;
use optimizer::{convert, optimize};
use parser::generate_ast;

pub fn compile(input: String) -> Result<(), String> {
    let mut opcodes = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .filter_map(parse)
        .collect::<VecDeque<_>>();

    let ast = generate_ast(&mut opcodes)?;

    let mut call_stack = 0;
    let ir = convert(ast, &mut call_stack);

    let ir = optimize(ir);

    let file = File::create("code.s").unwrap();
    let mut f = BufWriter::new(file);
    generate_code(ir, &mut f).unwrap();
    f.flush().unwrap();

    Ok(())
}
