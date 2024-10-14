use std::{
    collections::VecDeque,
    fs::File,
    io::{BufWriter, Write},
};

#[derive(Debug)]
enum Opcode {
    Plus,
    Minus,
    Rigth,
    Left,
    Comma,
    Dot,
    OpenBracket,
    CloseBracket,
}

fn parse(character: char) -> Option<Opcode> {
    match character {
        '+' => Some(Opcode::Plus),
        '-' => Some(Opcode::Minus),
        '>' => Some(Opcode::Rigth),
        '<' => Some(Opcode::Left),
        ',' => Some(Opcode::Comma),
        '.' => Some(Opcode::Dot),
        '[' => Some(Opcode::OpenBracket),
        ']' => Some(Opcode::CloseBracket),
        _ => None,
    }
}

#[derive(Debug)]
enum Expression {
    Plus,
    Minus,
    Rigth,
    Left,
    Comma,
    Dot,

    Conditional(Ast),
}

#[derive(Debug)]
struct Ast(Vec<Expression>);

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

fn generate_ast(tokens: &mut VecDeque<Opcode>) -> Result<Ast, String> {
    let block = generate_ast_vec(tokens)?;

    if tokens.is_empty() {
        Ok(block)
    } else {
        Err("Extra character in input".into())
    }
}

fn generate_header(f: &mut impl Write) -> std::io::Result<()> {
    write!(
        f,
        r#"global _start

section .data
  memory: times 1000 db 0
  buffer: db 0

  hexbuff: times 90 db " "
  idxbuff: db 0xA, " idx: --  "
  hex: db "0123456789abcdef"

section .text
"#
    )?;

    if cfg!(debug_assertions) {
        write!(
            f,
            r#"
dump:
  mov rax, r14             ; print index

  mov rcx, 0xf0
  and rcx, rax
  sar rcx, 4
  mov rcx, [hex + rcx]
  mov [idxbuff + 7], cl 

  mov rcx, 0x0f
  and rcx, rax
  mov rcx, [hex + rcx]
  mov [idxbuff + 8], cl 

  mov rax, 1
  mov rdi, 1
  mov rdx, 11
  mov rsi, idxbuff 
  syscall

  mov r13, 0
dumploop:
  mov rax, [memory + r13]

  mov rcx, 0xf0
  and rcx, rax
  sar rcx, 4
  mov rcx, [hex + rcx]
  mov [hexbuff + r13 * 3 + 0], cl 

  mov rcx, 0x0f
  and rcx, rax
  mov rcx, [hex + rcx]
  mov [hexbuff + r13 * 3 + 1], cl 

  inc r13 
  cmp r13, 30
  jnz dumploop

  mov byte [hexbuff + 89], 0xA

  mov rax, 1
  mov rdi, 1
  mov rdx, 91
  mov rsi, hexbuff 
  syscall

  ret
"#
        )?;
    }

    write!(
        f,
        r#"
_start:
  mov r15b, 0              ; current cell
  mov r14, 0               ; index
"#
    )?;

    Ok(())
}

fn generate_end(f: &mut impl Write) -> std::io::Result<()> {
    writeln!(f, "  mov byte [memory + r14], r15b")?;

    if cfg!(debug_assertions) {
        writeln!(f, "  call dump")?;
    }

    writeln!(
        f,
        r#"
  mov rax, 60
  xor rdi, rdi
  syscall"#
    )?;

    Ok(())
}

fn generate_code_ast(ast: Ast, f: &mut impl Write, call_stack: &mut u32) -> std::io::Result<()> {
    for statement in ast.0 {
        match statement {
            Expression::Plus => writeln!(f, "  inc r15b"),
            Expression::Minus => writeln!(f, "  dec r15b"),
            Expression::Rigth => {
                writeln!(f, "  mov byte [memory + r14], r15b")?;
                writeln!(f, "  inc r14")?;
                writeln!(f, "  mov byte r15b, [memory + r14]")?;
                Ok(())
            }
            Expression::Left => {
                writeln!(f, "  mov byte [memory + r14], r15b")?;
                writeln!(f, "  dec r14")?;
                writeln!(f, "  mov byte r15b, [memory + r14]")?;
                Ok(())
            }
            Expression::Comma => {
                writeln!(f, "  xor rax, rax")?;
                writeln!(f, "  xor rdi, rdi")?;
                writeln!(f, "  mov rdx, 1")?;
                writeln!(f, "  mov rsi, buffer")?;
                writeln!(f, "  syscall")?;
                writeln!(f, "  mov byte r15b, [buffer]")?;
                Ok(())
            }
            Expression::Dot => {
                writeln!(f, "  mov byte [buffer], r15b")?;
                // writeln!(f, "  cmp r15b, 10 ; replace line feed with space")?;
                // writeln!(f, "  jnz continue")?;
                // writeln!(f, "  mov byte [buffer], 32")?;
                // writeln!(f, "continue:")?;
                writeln!(f, "  mov rax, 1")?;
                writeln!(f, "  mov rdi, 1")?;
                writeln!(f, "  mov rdx, 1")?;
                writeln!(f, "  mov rsi, buffer")?;
                writeln!(f, "  syscall")?;
                Ok(())
            }
            Expression::Conditional(inner) => {
                let current = *call_stack;
                *call_stack += 1;
                writeln!(f, "call_{current}:")?;
                writeln!(f, "  cmp r15b, 0")?;
                writeln!(f, "  jz end_{current}")?;
                generate_code_ast(inner, f, call_stack)?;
                writeln!(f, "  jmp call_{current}")?;
                writeln!(f, "end_{current}:")?;

                Ok(())
            }
        }?
    }

    Ok(())
}

fn generate_code(ast: Ast, f: &mut impl Write) -> std::io::Result<()> {
    let mut call_stack = 0;

    generate_header(f)?;
    generate_code_ast(ast, f, &mut call_stack)?;
    generate_end(f)?;

    Ok(())
}

pub fn compile(input: String) -> Result<(), String> {
    let mut opcodes = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .filter_map(parse)
        .collect::<VecDeque<_>>();

    let ast = generate_ast(&mut opcodes)?;

    let file = File::create("code.s").unwrap();
    let mut f = BufWriter::new(file);
    generate_code(ast, &mut f).unwrap();
    f.flush().unwrap();

    Ok(())
}
