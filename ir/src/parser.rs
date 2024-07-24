use std::collections::VecDeque;

use crate::lexer::Opcode;

#[derive(Debug)]
pub enum Expression {
    Plus,
    Minus,
    Rigth,
    Left,
    Comma,
    Dot,

    Conditional(Ast),
}

#[derive(Debug)]
pub struct Ast(pub Vec<Expression>);

fn generate_ast_vec(tokens: &mut VecDeque<Opcode>) -> Result<Ast, String> {
    let mut ast = vec![];

    while let Some(opcode) = tokens.pop_front() {
        let expr = match opcode {
            Opcode::Plus => Expression::Plus,
            Opcode::Minus => Expression::Minus,
            Opcode::Rigth => Expression::Rigth,
            Opcode::Left => Expression::Left,
            Opcode::Comma => Expression::Comma,
            Opcode::Dot => Expression::Dot,
            Opcode::OpenBracket => {
                let inner = generate_ast_vec(tokens)?;
                match tokens.pop_front() {
                    Some(Opcode::CloseBracket) => Expression::Conditional(inner),
                    _ => return Err("missing closing bracket".into()),
                }
            }
            Opcode::CloseBracket => {
                tokens.push_front(Opcode::CloseBracket);
                return Ok(Ast(ast));
            }
        };

        ast.push(expr);
    }

    Ok(Ast(ast))
}

pub fn generate_ast(tokens: &mut VecDeque<Opcode>) -> Result<Ast, String> {
    let block = generate_ast_vec(tokens)?;

    if tokens.is_empty() {
        Ok(block)
    } else {
        Err("Extra character in input".into())
    }
}
