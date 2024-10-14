use std::{
    fs::read_to_string,
    io::{stdin, BufRead, Write},
    process::{Command, Stdio},
};

#[derive(PartialEq)]
enum Optimization {
    O0,
    O1,
    O2,
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
        let optimizations = args
            .iter()
            .filter(|arg| arg.starts_with("-O"))
            .map(|arg| arg.as_ref())
            .collect::<Vec<_>>();

        match optimizations[..] {
            [] => Ok(Optimization::O0),
            ["-O0"] => Ok(Optimization::O0),
            ["-O1"] => Ok(Optimization::O1),
            ["-O2"] => Ok(Optimization::O2),
            [opt] => Err(format!("unsuppoted optimization: {opt:?}")),
            _ => Err("multiple optimization specified".to_string()),
        }
    }?;

    match optimization {
        Optimization::O0 => base::compile(data),
        Optimization::O1 => ir::compile(data),
        Optimization::O2 => llvm::compile(data),
    }?;

    if optimization == Optimization::O0 || optimization == Optimization::O1 {
        Command::new("nasm")
            .args(["-g", "-f elf64", "code.s"])
            .status()
            .map_err(|e| format!("nasm: {e:?}"))?;

        Command::new("ld")
            .args(["-ocode", "code.o"])
            .status()
            .map_err(|e| format!("ld: {e:?}"))?;
    } else {
        Command::new("gcc")
            .args(["-o", "code", "code.s"])
            .status()
            .map_err(|e| format!("gcc: {e:?}"))?;
    }

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
