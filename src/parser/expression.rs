use std::sync::OnceLock;

use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};

use crate::parser::ast::{BinOp, Expr, UnaryOp};
use crate::parser::Rule;

fn pratt() -> &'static PrattParser<Rule> {
    static PRATT: OnceLock<PrattParser<Rule>> = OnceLock::new();
    PRATT.get_or_init(|| {
        PrattParser::new()
            .op(Op::infix(Rule::or_op, Assoc::Left))
            .op(Op::infix(Rule::and_op, Assoc::Left))
            .op(Op::prefix(Rule::not_op))
            .op(Op::infix(Rule::eq, Assoc::Left)
                | Op::infix(Rule::neq, Assoc::Left)
                | Op::infix(Rule::lt, Assoc::Left)
                | Op::infix(Rule::gt, Assoc::Left)
                | Op::infix(Rule::lte, Assoc::Left)
                | Op::infix(Rule::gte, Assoc::Left))
            .op(Op::infix(Rule::add, Assoc::Left)
                | Op::infix(Rule::sub, Assoc::Left))
            .op(Op::infix(Rule::mul, Assoc::Left)
                | Op::infix(Rule::div, Assoc::Left))
            .op(Op::prefix(Rule::neg))
    })
}

pub fn parse_expression(pair: Pair<Rule>) -> Expr {
    assert_eq!(pair.as_rule(), Rule::expression);
    parse_expr_inner(pair.into_inner())
}

fn parse_expr_inner(pairs: Pairs<Rule>) -> Expr {
    pratt()
        .map_primary(|p| match p.as_rule() {
            Rule::primary => parse_primary(p),
            Rule::expression => parse_expr_inner(p.into_inner()),
            rule => unreachable!("unexpected primary rule: {:?}", rule),
        })
        .map_prefix(|op, rhs| {
            let unary = match op.as_rule() {
                Rule::neg => UnaryOp::Neg,
                Rule::not_op => UnaryOp::Not,
                rule => unreachable!("unexpected prefix rule: {:?}", rule),
            };
            Expr::UnaryOp {
                op: unary,
                expr: Box::new(rhs),
            }
        })
        .map_infix(|lhs, op, rhs| {
            let bin = match op.as_rule() {
                Rule::add => BinOp::Add,
                Rule::sub => BinOp::Sub,
                Rule::mul => BinOp::Mul,
                Rule::div => BinOp::Div,
                Rule::eq => BinOp::Eq,
                Rule::neq => BinOp::Neq,
                Rule::lt => BinOp::Lt,
                Rule::gt => BinOp::Gt,
                Rule::lte => BinOp::Lte,
                Rule::gte => BinOp::Gte,
                Rule::and_op => BinOp::And,
                Rule::or_op => BinOp::Or,
                rule => unreachable!("unexpected infix rule: {:?}", rule),
            };
            Expr::BinaryOp {
                left: Box::new(lhs),
                op: bin,
                right: Box::new(rhs),
            }
        })
        .parse(pairs)
}

fn parse_primary(pair: Pair<Rule>) -> Expr {
    let inner = pair.into_inner().next().expect("primary has one inner rule");
    match inner.as_rule() {
        Rule::number => {
            let n: i64 = inner.as_str().parse().expect("valid integer literal");
            Expr::Number(n)
        }
        Rule::string => {
            let raw = inner.as_str();
            let content = &raw[1..raw.len() - 1];
            Expr::StringLit(content.to_string())
        }
        Rule::identifier => Expr::Identifier(inner.as_str().to_string()),
        Rule::expression => parse_expr_inner(inner.into_inner()),
        rule => unreachable!("unexpected rule inside primary: {:?}", rule),
    }
}
