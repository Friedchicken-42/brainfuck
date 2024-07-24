use std::io::Write;

use crate::optimizer::{IRExpr, IR};

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

fn generate_code_ast(ir: IR, f: &mut impl Write) -> std::io::Result<()> {
    for statement in ir {
        match statement {
            IRExpr::Set(v) => writeln!(f, "  mov r15b, {v}"),
            IRExpr::Update(v) => {
                if v >= 0 {
                    writeln!(f, "  add r15b, {}", v)
                } else {
                    writeln!(f, "  sub r15b, {}", -v)
                }
            }
            IRExpr::Step(v) => {
                writeln!(f, "  mov byte [memory + r14], r15b")?;

                if v >= 0 {
                    writeln!(f, "  add r14, {}", v)?;
                } else {
                    writeln!(f, "  sub r14, {}", -v)?;
                }

                writeln!(f, "  mov byte r15b, [memory + r14]")?;
                Ok(())
            }
            IRExpr::Input => {
                writeln!(f, "  xor rax, rax")?;
                writeln!(f, "  xor rdi, rdi")?;
                writeln!(f, "  mov rdx, 1")?;
                writeln!(f, "  mov rsi, buffer")?;
                writeln!(f, "  syscall")?;
                writeln!(f, "  mov byte r15b, [buffer]")?;
                Ok(())
            }
            IRExpr::Output => {
                writeln!(f, "  mov byte [buffer], r15b")?;
                writeln!(f, "  mov rax, 1")?;
                writeln!(f, "  mov rdi, 1")?;
                writeln!(f, "  mov rdx, 1")?;
                writeln!(f, "  mov rsi, buffer")?;
                writeln!(f, "  syscall")?;
                Ok(())
            }
            IRExpr::ConditionalStart(id) => {
                writeln!(f, "call_{id}:")?;
                writeln!(f, "  cmp r15b, 0")?;
                writeln!(f, "  jz end_{id}")?;
                Ok(())
            }
            IRExpr::ConditionalEnd(id) => {
                writeln!(f, "  jmp call_{id}")?;
                writeln!(f, "end_{id}:")?;
                Ok(())
            }
        }?;
    }

    Ok(())
}

pub fn generate_code(ir: IR, f: &mut impl Write) -> std::io::Result<()> {
    generate_header(f)?;
    generate_code_ast(ir, f)?;
    generate_end(f)?;

    Ok(())
}
