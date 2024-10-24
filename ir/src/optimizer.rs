use std::{collections::HashMap, fmt::Debug};

use crate::parser::{Ast, Expression};

#[derive(PartialEq)]
pub enum IRExpr {
    Set(i32),
    Update(i32),
    Step(i32),
    Input,
    Output,
    ConditionalStart(u32),
    ConditionalEnd(u32),
}

impl Debug for IRExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Set(arg0) => write!(f, "={arg0}"),
            Self::Update(arg0) => match *arg0 {
                0 => write!(f, "+0"),
                x if x > 0 => write!(f, "+{x}"),
                x if x < 0 => write!(f, "{x}"),
                _ => unreachable!(),
            },
            Self::Step(arg0) => match *arg0 {
                x if x >= 0 => write!(f, ">{x}"),
                x => write!(f, "<{}", x.abs()),
            },
            Self::Input => write!(f, ","),
            Self::Output => write!(f, "."),
            Self::ConditionalStart(arg0) => write!(f, "[({arg0})"),
            Self::ConditionalEnd(arg0) => write!(f, "]({arg0})"),
        }
    }
}

pub type IR = Vec<IRExpr>;

pub fn convert(ast: Ast, call_stack: &mut u32) -> IR {
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

fn access_analysis(ir: IR) -> (IR, bool) {
    let mut updated = false;
    let mut accesses = HashMap::new();
    let mut new_ir = Vec::with_capacity(ir.len());

    let mut step = 0;

    for (index, expr) in ir.into_iter().enumerate() {
        let (index, expr) = match expr {
            IRExpr::Input
            | IRExpr::Output
            | IRExpr::ConditionalStart(_)
            | IRExpr::ConditionalEnd(_) => {
                step = 0;
                accesses.clear();
                (index, expr)
            }
            IRExpr::Step(s) => {
                step += s;
                (index, expr)
            }
            IRExpr::Set(_) | IRExpr::Update(_) => {
                if let Some(old_index) = accesses.get_mut(&step) {
                    updated = true;
                    let current = *old_index;
                    *old_index += 1;
                    (current + 1, expr)
                } else {
                    accesses.insert(step, index);
                    (index, expr)
                }
            }
        };
        new_ir.insert(index, expr);
    }

    (new_ir, updated)
}

fn unreachable_branch(ir: IR) -> (IR, bool) {
    let mut updated = false;
    let mut new_ir = Vec::with_capacity(ir.len());
    let mut remove = None;

    for expr in ir {
        match (new_ir.last(), &expr) {
            (Some(IRExpr::ConditionalEnd(_)), IRExpr::ConditionalStart(id))
            | (Some(IRExpr::Set(0)), IRExpr::ConditionalStart(id)) => {
                updated = true;
                remove = Some(*id);
            }
            _ => {}
        }

        if let Some(r) = remove {
            if expr == IRExpr::ConditionalEnd(r) {
                remove = None;
            }
        } else {
            new_ir.push(expr);
        }
    }

    (new_ir, updated)
}

fn simple_rules(mut ir: IR) -> (IR, bool) {
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

    let mut updated = false;

    for rule in &rules {
        let changed;
        (ir, changed) = replace(ir, rule);
        updated |= changed;
    }

    (ir, updated)
}

pub fn optimize(mut ir: IR) -> IR {
    loop {
        let mut updated = false;
        let mut changed;

        (ir, changed) = simple_rules(ir);
        updated |= changed;
        (ir, changed) = access_analysis(ir);
        updated |= changed;
        (ir, changed) = unreachable_branch(ir);
        updated |= changed;

        println!("{ir:?}");

        if !updated {
            break;
        }
    }

    ir
}

#[cfg(test)]
mod tests {
    use super::{access_analysis, simple_rules, IRExpr};

    #[test]
    fn access_analysis_test() {
        let ir = vec![
            IRExpr::Step(1),
            IRExpr::Update(2),
            IRExpr::Step(-1),
            IRExpr::Update(2),
            IRExpr::Step(1),
            IRExpr::Update(2),
            IRExpr::Step(-1),
            IRExpr::Update(2),
            IRExpr::Step(1),
        ];

        // update(4), step(1), update(4)
        let (out, _) = access_analysis(ir);
        let (out, _) = simple_rules(out);

        println!("{out:?}");
    }
}
