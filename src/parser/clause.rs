use pest::iterators::Pair;

use crate::parser::ast::{
    ColumnDef, DataType, Expr, OrderBy, SelectColumns, SelectItem,
};
use crate::parser::expression::parse_expression;
use crate::parser::Rule;

pub fn parse_where_clause(pair: Pair<Rule>) -> Expr {
    let expr_pair = pair
        .into_inner()
        .next()
        .expect("where_clause must contain an expression");
    parse_expression(expr_pair)
}

pub fn parse_order_by_clause(pair: Pair<Rule>) -> OrderBy {
    let ident = pair
        .into_inner()
        .next()
        .expect("order_by_clause must contain an identifier");
    OrderBy {
        column: ident.as_str().to_string(),
    }
}

pub fn parse_column_definition(pair: Pair<Rule>) -> ColumnDef {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let dt = parse_data_type(inner.next().unwrap());
    ColumnDef {
        name,
        data_type: dt,
    }
}

pub fn parse_column_definition_list(pair: Pair<Rule>) -> Vec<ColumnDef> {
    pair.into_inner().map(parse_column_definition).collect()
}

pub fn parse_data_type(pair: Pair<Rule>) -> DataType {
    let mut inner = pair.into_inner();
    match inner.next() {
        None => {
            DataType::Integer
        }
        Some(first) => {
            let n: u32 = first.as_str().parse().expect("valid varchar length");
            DataType::Varchar(n)
        }
    }
}

pub fn parse_expression_list(pair: Pair<Rule>) -> SelectColumns {
    let mut items = Vec::new();
    let mut inner = pair.into_inner();

    while let Some(p) = inner.next() {
        match p.as_rule() {
            Rule::expression => {
                let expr = parse_expression(p);
                let alias = inner
                    .peek()
                    .filter(|nxt| nxt.as_rule() == Rule::column_alias)
                    .map(|a| {
                        let _ = inner.next();
                        parse_column_alias(a)
                    });
                items.push(SelectItem { expr, alias });
            }
            rule => unreachable!("unexpected rule in expression_list: {:?}", rule),
        }
    }

    SelectColumns::Expressions(items)
}

pub fn parse_column_alias(pair: Pair<Rule>) -> String {
    pair.into_inner()
        .next()
        .expect("column_alias must contain an identifier")
        .as_str()
        .to_string()
}

pub fn parse_insert_value(pair: Pair<Rule>) -> Vec<Expr> {
    pair.into_inner()
        .map(|v| {
            let expr_pair = v.into_inner().next().expect("value wraps expression");
            parse_expression(expr_pair)
        })
        .collect()
}

pub fn parse_insert_value_list(pair: Pair<Rule>) -> Vec<Vec<Expr>> {
    pair.into_inner().map(parse_insert_value).collect()
}
