use pest::iterators::Pair;

use crate::parser::ast::{SelectColumns, Statement};
use crate::parser::clause::{
    parse_column_definition_list, parse_expression_list, parse_insert_value_list,
    parse_order_by_clause, parse_where_clause,
};
use crate::parser::Rule;

pub fn parse_statement(pair: Pair<Rule>) -> Statement {
    let inner = pair.into_inner().next().expect("statement has one inner rule");
    match inner.as_rule() {
        Rule::select_statement => parse_select_statement(inner),
        Rule::create_statement => parse_create_statement(inner),
        Rule::insert_statement => parse_insert_statement(inner),
        rule => unreachable!("unexpected rule in statement: {:?}", rule),
    }
}

pub fn parse_select_statement(pair: Pair<Rule>) -> Statement {
    let mut columns = SelectColumns::All;
    let mut from = String::new();
    let mut where_clause = None;
    let mut order_by = None;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::select_all => {
                columns = SelectColumns::All;
            }
            Rule::expression_list => {
                columns = parse_expression_list(p);
            }
            Rule::identifier => {
                from = p.as_str().to_string();
            }
            Rule::where_clause => {
                where_clause = Some(parse_where_clause(p));
            }
            Rule::order_by_clause => {
                order_by = Some(parse_order_by_clause(p));
            }
            rule => unreachable!("unexpected rule in select_statement: {:?}", rule),
        }
    }

    Statement::Select {
        columns,
        from,
        where_clause,
        order_by,
    }
}

pub fn parse_create_statement(pair: Pair<Rule>) -> Statement {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let col_defs = parse_column_definition_list(inner.next().unwrap());

    Statement::CreateTable {
        name,
        columns: col_defs,
    }
}

pub fn parse_insert_statement(pair: Pair<Rule>) -> Statement {
    let mut inner = pair.into_inner();
    let table = inner.next().unwrap().as_str().to_string();
    let values = parse_insert_value_list(inner.next().unwrap());

    Statement::Insert { table, values }
}