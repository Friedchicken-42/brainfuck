use crate::parser::{Ast, Expression};

#[derive(Debug)]
pub enum IRExpr {
    Set(i32),
    Update(i32),
    Step(i32),
    Input,
    Output,
    ConditionalStart(u32),
    ConditionalEnd(u32),
}

pub type IR = Vec<IRExpr>;

fn convert(ast: Ast, call_stack: &mut u32) -> IR {
    let mut arr = vec![];

    for expr in ast.0 {
        let ire = match expr {
            Expression::Plus => IRExpr::Update(1),
            Expression::Minus => IRExpr::Update(-1),
            Expression::Rigth => IRExpr::Step(1),
            Expression::Left => IRExpr::Step(-1),
            Expression::Comma => IRExpr::Input,
            Expression::Dot => IRExpr::Output,
            Expression::Conditional(inner) => {
                let current = *call_stack;
                *call_stack += 1;

                arr.push(IRExpr::ConditionalStart(current));
                let inner = convert(inner, call_stack);
                arr.extend(inner);
                IRExpr::ConditionalEnd(current)
            }
        };

        arr.push(ire);
    }

    arr
}

struct Rule {
    replace: fn(&[IRExpr]) -> Option<Vec<IRExpr>>,
    length: usize,
}

fn replace(mut ir: IR, rule: &Rule) -> (IR, bool) {
    let mut i = 0;
    let mut updated = false;

    if ir.len() < rule.length {
        return (ir, false);
    }

    while i < ir.len() - rule.length + 1 {
        let slice = &ir[i..i + rule.length];

        if let Some(new) = (rule.replace)(slice) {
            ir.drain(i..i + rule.length);
            for expr in new.into_iter().rev() {
                ir.insert(i, expr);
            }
            updated = true;
        } else {
            i += 1;
        }

        if ir.len() < rule.length {
            break;
        }
    }

    (ir, updated)
}

pub fn optimize(ast: Ast) -> IR {
    let mut call_stack = 0;
    let mut ir = convert(ast, &mut call_stack);

    let rules = [
        Rule {
            replace: |slice| match slice {
                [IRExpr::Update(0)] => Some(vec![]),
                [IRExpr::Step(0)] => Some(vec![]),
                _ => None,
            },
            length: 1,
        },
        Rule {
            replace: |slice| match slice {
                [IRExpr::Update(a), IRExpr::Update(b)] => Some(vec![IRExpr::Update(a + b)]),
                _ => None,
            },
            length: 2,
        },
        Rule {
            replace: |slice| match slice {
                [IRExpr::Step(a), IRExpr::Step(b)] => Some(vec![IRExpr::Step(a + b)]),
                _ => None,
            },
            length: 2,
        },
        Rule {
            replace: |slice| match slice {
                [IRExpr::Set(a), IRExpr::Update(b)] => Some(vec![IRExpr::Set(a + b)]),
                [IRExpr::Update(_), IRExpr::Set(a)] => Some(vec![IRExpr::Set(*a)]),
                _ => None,
            },
            length: 2,
        },
        Rule {
            replace: |slice| match slice {
                [IRExpr::ConditionalStart(_), IRExpr::Update(-1), IRExpr::ConditionalEnd(_)] => {
                    Some(vec![IRExpr::Set(0)])
                }
                _ => None,
            },
            length: 3,
        },
        Rule {
            replace: |slice| match slice {
                [IRExpr::Update(a), IRExpr::ConditionalStart(_), IRExpr::Step(sl), IRExpr::Update(b), IRExpr::Step(sr), IRExpr::Update(-1), IRExpr::ConditionalEnd(_)]
                    if *sl == -*sr =>
                {
                    Some(vec![
                        IRExpr::Step(*sl),
                        IRExpr::Update(a * b),
                        IRExpr::Step(*sr),
                        IRExpr::Set(0),
                    ])
                }
                _ => None,
            },
            length: 7,
        },
    ];

    loop {
        let mut updated = false;
        for rule in &rules {
            let changed;
            (ir, changed) = replace(ir, rule);
            updated |= changed;
        }

        if !updated {
            break;
        }
    }

    ir
}
