use std::{
    collections::VecDeque,
    fs::File,
    io::{BufWriter, Write},
    process::{exit, Command},
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
    Right,
    Left,
    Comma,
    Dot,
    Conditional(Ast),
}

#[derive(Debug)]
struct Ast(pub Vec<Expression>);

fn generate_ast_vec(tokens: &mut VecDeque<Opcode>) -> Result<Ast, String> {
    let mut ast = vec![];

    while let Some(opcode) = tokens.pop_front() {
        let expr = match opcode {
            Opcode::Plus => Expression::Plus,
            Opcode::Minus => Expression::Minus,
            Opcode::Rigth => Expression::Right,
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

fn codegen_header(counter: &mut usize, f: &mut impl Write) -> std::io::Result<()> {
    write!(
        f,
        r#"@stdin = external global ptr, align 8
@stdout = external global ptr, align 8

define i8 @main() {{
  %arr = alloca ptr, align 8
  %1 = call noalias ptr @calloc(i64 noundef 0, i64 noundef 1000) #3
  store ptr %1, ptr %arr, align 8
"#
    )?;

    *counter += 2;

    Ok(())
}

fn codegen_end(counter: &mut usize, f: &mut impl Write) -> std::io::Result<()> {
    write!(
        f,
        r#"  
  %{0} = load ptr, ptr %arr, align 8
  %{1} = load i8, ptr %{0}, align 1

  ret i8 0
}}


declare noalias ptr @calloc(i64 noundef, i64 noundef) #1
declare i32 @putc(i8 noundef, ptr noundef) #2
declare i8 @getc(ptr noundef) #2

attributes #0 = {{ noinline nounwind optnone sspstrong uwtable "frame-pointer"="all" "min-legal-vector-width"="0" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }}
attributes #1 = {{ nounwind allocsize(0,1) "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }}
attributes #2 = {{ "frame-pointer"="all" "no-trapping-math"="true" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+cmov,+cx8,+fxsr,+mmx,+sse,+sse2,+x87" "tune-cpu"="generic" }}
attributes #3 = {{ nounwind allocsize(0,1) }}
"#,
        *counter,
        *counter + 1
    )?;

    *counter += 2;

    Ok(())
}

fn codegen_ast(
    ast: Ast,
    counter: &mut usize,
    call_stack: &mut usize,
    f: &mut impl Write,
) -> std::io::Result<()> {
    for expr in ast.0 {
        match expr {
            Expression::Plus => {
                write!(
                    f,
                    r#"
  %{0} = load ptr, ptr %arr, align 8
  %{1} = load i8, ptr %{0}, align 1
  %{2} = add nsw i8 %{1}, 1
  store i8 %{2}, ptr %{0}, align 1
"#,
                    *counter,
                    *counter + 1,
                    *counter + 2
                )?;

                *counter += 3;
            }
            Expression::Minus => {
                write!(
                    f,
                    r#"
  %{0} = load ptr, ptr %arr, align 8
  %{1} = load i8, ptr %{0}, align 1
  %{2} = sub nsw i8 %{1}, 1
  store i8 %{2}, ptr %{0}, align 1
"#,
                    *counter,
                    *counter + 1,
                    *counter + 2
                )?;

                *counter += 3;
            }
            Expression::Right => {
                write!(
                    f,
                    r#"
  %{0} = load ptr, ptr %arr, align 8
  %{1} = getelementptr inbounds i32, ptr %{0}, i32 1
  store ptr %{1}, ptr %arr, align 8
"#,
                    *counter,
                    *counter + 1
                )?;

                *counter += 2;
            }
            Expression::Left => {
                write!(
                    f,
                    r#"
  %{0} = load ptr, ptr %arr, align 8
  %{1} = getelementptr inbounds i32, ptr %{0}, i32 -1
  store ptr %{1}, ptr %arr, align 8
"#,
                    *counter,
                    *counter + 1
                )?;

                *counter += 2;
            }
            Expression::Comma => {
                write!(
                    f,
                    r#"
  %{0} = load ptr, ptr @stdin, align 8
  %{1} = call i8 @getc(ptr noundef %{0})

  %{2} = load ptr, ptr %arr, align 8
  store i8 %{1}, ptr %{2}, align 1
                    "#,
                    *counter,
                    *counter + 1,
                    *counter + 2
                )?;

                *counter += 3;
            }
            Expression::Dot => {
                write!(
                    f,
                    r#"
  %{0} = load ptr, ptr %arr, align 8
  %{1} = load i8, ptr %{0}, align 1
  %{2} = load ptr, ptr @stdout, align 8
  %{3} = call i32 @putc(i8 noundef %{1}, ptr noundef %{2})
                    "#,
                    *counter,
                    *counter + 1,
                    *counter + 2,
                    *counter + 3
                )?;

                *counter += 4;
            }
            Expression::Conditional(inner) => {
                let current = *call_stack;
                *call_stack += 1;

                write!(
                    f,
                    r#"
  br label %start_{current}
start_{current}:
  %{0} = load ptr, ptr %arr, align 8
  %{1} = load i8, ptr %{0}, align 1
  %{2} = icmp ne i8 %{1}, 0
  br i1 %{2}, label %loop_{current}, label %end_{current}
loop_{current}:
"#,
                    *counter,
                    *counter + 1,
                    *counter + 2
                )?;

                *counter += 3;

                codegen_ast(inner, counter, call_stack, f)?;

                write!(
                    f,
                    r#"
  br label %start_{current}
end_{current}:
"#,
                )?;
            }
        }
    }

    Ok(())
}

fn codegen(ast: Ast, f: &mut impl Write) -> std::io::Result<()> {
    let mut counter = 0;
    let mut call_stack = 0;

    codegen_header(&mut counter, f)?;
    codegen_ast(ast, &mut counter, &mut call_stack, f)?;
    codegen_end(&mut counter, f)?;

    Ok(())
}

pub fn compile(input: String) -> Result<(), String> {
    let mut opcodes = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .filter_map(parse)
        .collect::<VecDeque<_>>();

    let exprs = generate_ast(&mut opcodes)?;

    let file = File::create("code.ll").unwrap();
    let mut f = BufWriter::new(file);
    codegen(exprs, &mut f).unwrap();
    f.flush().unwrap();
    drop(f);

    Command::new("llc")
        .args(["code.ll"])
        .status()
        .map_err(|e| format!("llc: {e:?}"))?;

    Ok(())
}
