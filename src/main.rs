use std::{
    fs::read_to_string,
    io::{stdin, BufRead, Write},
    process::{Command, Stdio},
};

#[derive(PartialEq)]
enum Optimization {
    O0,
    O1,
}

fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();

    let data = match args.iter().position(|s| s == "-f") {
        Some(index) => match args.get(index + 1) {
            Some(path) => read_to_string(path).map_err(|_| format!("file {path:?} not found")),
            None => Err("missing filename after \"-f\"".into()),
        },
        None => args.last().cloned().ok_or("missing input data".into()),
    }?;

    let optimization = {
        if args.contains(&"-O0".to_owned()) && args.contains(&"-O1".to_owned()) {
            Err("multiple optimization specified".to_string())
        } else if args.contains(&"-O0".to_owned()) {
            Ok(Optimization::O0)
        } else if args.contains(&"-O1".to_owned()) {
            Ok(Optimization::O1)
        } else {
            Ok(Optimization::O0)
        }
    }?;

    match optimization {
        Optimization::O0 => base::compile(data),
        Optimization::O1 => ir::compile(data),
    }?;

    Command::new("nasm")
        .args(["-g", "-f elf64", "code.s"])
        .status()
        .map_err(|e| format!("nasm: {e:?}"))?;

    Command::new("ld")
        .args(["-ocode", "code.o"])
        .status()
        .map_err(|e| format!("ld: {e:?}"))?;

    let mut input = String::new();

    stdin()
        .lock()
        .read_line(&mut input)
        .map_err(|_| "Error reading string".to_string())?;

    let input = if input.ends_with('\n') {
        format!("{}\0", &input[0..input.len() - 1])
    } else {
        input
    };

    let mut cmd = Command::new("./code")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn: {e:?}"))?;

    if let Some(mut stdin) = cmd.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|_| "Error while passing stdin".to_string())?;
    }

    cmd.wait().map_err(|e| format!("exec: {e:?}"))?;

    Ok(())
}
