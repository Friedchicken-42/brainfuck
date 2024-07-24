// + - > < , . [ ]
#[derive(Debug)]
pub enum Opcode {
    Plus,
    Minus,
    Rigth,
    Left,
    Comma,
    Dot,
    OpenBracket,
    CloseBracket,
}

pub fn parse(character: char) -> Option<Opcode> {
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
