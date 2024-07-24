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
use optimizer::optimize;
use parser::generate_ast;

pub fn compile(input: String) -> Result<(), String> {
    let mut opcodes = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .filter_map(parse)
        .collect::<VecDeque<_>>();

    let ast = generate_ast(&mut opcodes)?;

    let ir = optimize(ast);

    let file = File::create("code.s").unwrap();
    let mut f = BufWriter::new(file);
    generate_code(ir, &mut f).unwrap();
    f.flush().unwrap();

    Ok(())
}
